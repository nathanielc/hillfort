use crossbeam_channel::{select, tick};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tempdir::TempDir;

use crate::db;
use crate::error::Error;
use crate::models::*;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
pub fn run() {
    let conn = establish_connection();
    let ticker = tick(Duration::from_millis(2000));

    loop {
        select! {
            recv(ticker) -> _ => {
                 match db::get_pending_climbs(&conn){
                        Ok(climbs) => {
                            for c in climbs {
                                match climb(&conn,c) {
                                    Ok(_) => {},
                                    Err(e) => println!("Error processing climb {}", e),
                                }
                            }
                        }
                        Err(e) => println!("Error retrieving pending climbs {}", e),
                }
            }
        }
    }
}

fn climb(conn: &SqliteConnection, c: Climb) -> Result<(), Error> {
    db::update_climb_status(&conn, c.id, 1)?;
    let hill = db::get_hill_by_id(conn, c.hill)?;
    let warrior = db::get_warrior_by_id(conn, c.warrior)?;
    let ws = db::get_warriors_on_hill(conn, hill.id)?;

    let mut warriors: Vec<Warrior> = ws
        .iter()
        .map(|hw| -> Result<Warrior, Error> { Ok(db::get_warrior_by_id(conn, hw.warrior)?) })
        .collect::<Result<Vec<Warrior>, Error>>()?;
    warriors.push(warrior);

    let tmp_dir = TempDir::new("hillfort-climb")?;
    let opt = tmp_dir.path().join("hill.opt");
    let mut opt_f = File::create(&opt)?;
    write_opt(&mut opt_f, &hill);

    let mut w_paths: Vec<PathBuf> = Vec::with_capacity(warriors.len());
    let mut hws: Vec<NewHillWarrior> = Vec::with_capacity(warriors.len());
    for w in &warriors {
        let p = tmp_dir.path().join(&w.name);
        let mut f = File::create(&p)?;
        f.write_all(w.redcode.as_bytes())?;
        w_paths.push(p);
        hws.push(NewHillWarrior {
            hill: hill.id,
            warrior: w.id,
            rank: 0,
            win: 0.0,
            loss: 0.0,
            tie: 0.0,
            score: 0.0,
        });
    }
    lazy_static! {
        static ref SCORE_RE: Regex = Regex::new("([0-9]+) ([0-9]+)").unwrap();
    }
    for (i, _) in (&warriors).iter().enumerate() {
        for (j, _) in (&warriors).iter().enumerate() {
            let output = Command::new("pmars-server")
                .args(&[
                    "-k",
                    "-b",
                    "-@",
                    opt.to_str().unwrap(),
                    w_paths[i].to_str().unwrap(),
                    w_paths[j].to_str().unwrap(),
                ])
                .output()?;
            if !output.status.success() {
                println!("STDERR: {}", String::from_utf8(output.stderr)?);
                break;
            }
            for (index, line) in output.stdout.lines().enumerate() {
                let widx = match index {
                    0 => i,
                    1 => j,
                    _ => panic!("got bad stdout"),
                };
                for n in SCORE_RE.captures_iter(line.unwrap().as_str()) {
                    let wins = n[1].parse::<f32>().unwrap();
                    let ties = n[2].parse::<f32>().unwrap();
                    hws[widx].win += wins;
                    hws[widx].tie += ties;
                    hws[widx].loss += (hill.rounds as f32) - wins - ties;
                }
                if i == j {
                    // do not count self play twice
                    break;
                }
            }
        }
    }
    let total_rounds = hill.rounds as f32 * warriors.len() as f32;
    for hw in &mut hws {
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
            break;
        }
        println!("HW, {:?}", hw);
        db::create_hill_warrior(conn, &hw)?;
    }
    db::update_climb_status(&conn, c.id, 2)?;
    Ok(())
}
fn write_opt(f: &mut File, h: &Hill) {
    write!(f, ";redcode-{}\n", h.key).unwrap();
    if h.instruction_set == 88 {
        write!(f, "-8\n").unwrap();
    }
    write!(f, "-s {}\n", h.core_size).unwrap();
    write!(f, "-c {}\n", h.max_cycles).unwrap();
    write!(f, "-p {}\n", h.max_processes).unwrap();
    write!(f, "-l {}\n", h.max_warrior_length).unwrap();
    write!(f, "-d {}\n", h.min_distance).unwrap();
    write!(f, "-r {}\n", h.rounds).unwrap();
}
