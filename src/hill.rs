use crossbeam_channel::{select, tick};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tempdir::TempDir;

use crate::db;
use crate::error::{Code, Error};
use crate::models::*;

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
pub fn run() {
    let conn = establish_connection();
    let ticker = tick(Duration::from_millis(2000));

    match db::update_inprogress_climbs_to_pending(&conn) {
        Err(e) if !e.is_not_found() => error!("Error restoring existing climbs: {}", e),
        _ => {}
    };

    loop {
        select! {
            recv(ticker) -> _ => {
                 match db::get_pending_climbs(&conn){
                    Ok(climbs) => {
                        for c in climbs {
                            let id = c.id;
                            match climb(&conn,c) {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Error processing climb {}", e);
                                    match db::update_climb_status(&conn, id, ClimbStatus::Failed as i32) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            error!("Error updating failed climb {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => error!("Error retrieving pending climbs {}", e),
                }
            }
        }
    }
}

fn climb(conn: &SqliteConnection, c: Climb) -> Result<(), Error> {
    db::update_climb_status(&conn, c.id, ClimbStatus::InProgress as i32)?;
    let hill = db::get_hill_by_id(conn, c.hill)?;
    let warrior = db::get_warrior_by_id(conn, c.warrior)?;
    let wid = warrior.id;
    let ws = db::get_warriors_on_hill(conn, hill.id)?;

    let mut warriors: Vec<Warrior> = ws
        .iter()
        .map(|hw| -> Result<Warrior, Error> { Ok(db::get_warrior_by_id(conn, hw.warrior)?) })
        .collect::<Result<Vec<Warrior>, Error>>()?
        .drain(..)
        // if this is a new submission of the an existing warrior remove it
        .filter(|w| w.id != wid)
        .collect();
    warriors.push(warrior);

    let tmp_dir = TempDir::new("hillfort-climb")?;
    let opt_path = tmp_dir.path().join("hill.opt");
    let mut opt_f = File::create(&opt_path)?;
    let opt_str = opt_string(&hill);
    opt_f.write_all(opt_str.clone().as_bytes())?;

    let mut w_paths: Vec<PathBuf> = Vec::with_capacity(warriors.len());
    let mut hws: Vec<NewHillWarrior> = Vec::with_capacity(warriors.len());
    for w in &warriors {
        let p = tmp_dir.path().join(&w.name);
        let mut f = File::create(&p)?;
        f.write_all(w.redcode.as_bytes())?;
        w_paths.push(p);
        let hw = ws.iter().find_map(|hw| if hw.warrior == w.id {
            Some(hw)
        } else {
            None
        });
        let age = match hw {
            Some(hw) => if hw.warrior == wid {
                0
            } else {
                hw.age+1
            },
            None => 0,
        };
        hws.push(NewHillWarrior {
            hill: hill.id,
            warrior: w.id,
            rank: 0,
            win: 0.0,
            loss: 0.0,
            tie: 0.0,
            score: 0.0,
            age: age,
        });
    }

    db::delete_battles_with_warrior(conn, hill.id, wid)?;
    let n = warriors.len();
    for i in 0..n {
        for j in i..n {
            match compute_battle(conn, &hill, wid, &warriors, &w_paths, i, j, &opt_path) {
                Ok(battle) => {
                    if battle.warrior_a == warriors[i].id {
                        hws[i].win += battle.a_win as f32;
                        hws[i].tie += battle.a_tie as f32;
                        hws[i].loss += battle.a_loss as f32;

                        hws[j].win += battle.b_win as f32;
                        hws[j].tie += battle.b_tie as f32;
                        hws[j].loss += battle.b_loss as f32;
                    } else {
                        hws[i].win += battle.b_win as f32;
                        hws[i].tie += battle.b_tie as f32;
                        hws[i].loss += battle.b_loss as f32;

                        hws[j].win += battle.a_win as f32;
                        hws[j].tie += battle.a_tie as f32;
                        hws[j].loss += battle.a_loss as f32;
                    }
                }
                Err(e) => return Err(e),
            };
        }
    }
    for hw in &mut hws {
        let total_rounds = hw.win + hw.loss + hw.tie;
        hw.win = hw.win / total_rounds * 100.0;
        hw.loss = hw.loss / total_rounds * 100.0;
        hw.tie = hw.tie / total_rounds * 100.0;
        hw.score = hw.win * 3.0 + hw.tie;
    }
    db::delete_warriors_on_hill(conn, hill.id)?;
    hws.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    for (rank, hw) in (&mut hws).iter_mut().enumerate() {
        hw.rank = rank as i32 + 1;
        if hw.rank > hill.slots {
            db::delete_battles_with_warrior(conn, hill.id, hw.warrior)?;
            continue;
        }
        db::create_hill_warrior(conn, &hw)?;
    }
    db::update_climb_status(&conn, c.id, ClimbStatus::Finished as i32)?;
    Ok(())
}
fn opt_string(h: &Hill) -> String {
    let mut s = String::new();
    s.push_str(format!(";redcode-{}\n", h.key).as_str());
    if h.instruction_set == 88 {
        s.push_str(format!("-8\n").as_str());
    }
    s.push_str(format!("-s {}\n", h.core_size).as_str());
    s.push_str(format!("-c {}\n", h.max_cycles).as_str());
    s.push_str(format!("-p {}\n", h.max_processes).as_str());
    s.push_str(format!("-l {}\n", h.max_warrior_length).as_str());
    s.push_str(format!("-d {}\n", h.min_distance).as_str());
    s.push_str(format!("-r {}\n", h.rounds).as_str());
    s
}

fn compute_battle<'a>(
    conn: &SqliteConnection,
    hill: &Hill,
    wid: i32,
    warriors: &Vec<Warrior>,
    w_paths: &Vec<PathBuf>,
    i: usize,
    j: usize,
    opt_path: &PathBuf,
) -> Result<Battle, Error> {
    if warriors[i].id != wid && warriors[j].id != wid {
        match db::get_battle_by_ids(conn, hill.id, warriors[i].id, warriors[j].id) {
            Ok(b) => return Ok(b),
            Err(e) if e.is_not_found() => {
                // do nothing and perform battle
            }
            Err(e) => return Err(e),
        }
    }

    let output = Command::new("pmars-server")
        .args(&[
            "-k",
            "-b",
            "-@",
            opt_path.to_str().unwrap(),
            w_paths[i].to_str().unwrap(),
            w_paths[j].to_str().unwrap(),
        ])
        .output()?;
    if !output.status.success() {
        error!("STDERR: {}", String::from_utf8(output.stderr)?);
        return Err(Error {
            code: Code::Internal,
        });
    }
    let mut newbattle = NewBattle {
        hill: hill.id,
        warrior_a: warriors[i].id,
        warrior_b: warriors[j].id,
        a_win: 0,
        a_loss: 0,
        a_tie: 0,
        b_win: 0,
        b_loss: 0,
        b_tie: 0,
    };
    lazy_static! {
        static ref SCORE_RE: Regex = Regex::new("([0-9]+) ([0-9]+)").unwrap();
    }
    let lines = output
        .stdout
        .lines()
        .collect::<Result<Vec<String>, io::Error>>()?;
    let (a_win, a_tie) = parse_score_line(lines[0].as_str());
    let (b_win, b_tie) = parse_score_line(lines[1].as_str());
    newbattle.a_win = a_win;
    newbattle.a_tie = a_tie;
    newbattle.a_loss = hill.rounds - a_win - a_tie;
    newbattle.b_win = b_win;
    newbattle.b_tie = b_tie;
    newbattle.b_loss = hill.rounds - b_win - b_tie;
    db::create_battle(conn, &newbattle)
}

fn parse_score_line(line: &str) -> (i32, i32) {
    lazy_static! {
        static ref SCORE_RE: Regex = Regex::new("([0-9]+) ([0-9]+)").unwrap();
    }
    for n in SCORE_RE.captures_iter(line) {
        let wins = n[1].parse::<i32>().unwrap();
        let ties = n[2].parse::<i32>().unwrap();
        return (wins, ties);
    }
    (0, 0)
}
