#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate base64;
extern crate regex;

use regex::Regex;
use rocket::fairing::AdHoc;
use rocket::http::{RawStr, Status};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::Rocket;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::thread;

mod db;
mod error;
mod hill;
mod models;
mod schema;

use crate::error::Error;

#[database("sqlite_db")]
struct DbConn(diesel::SqliteConnection);

#[derive(Debug, FromForm, serde::Serialize)]
struct WarriorFormInput {
    pub redcode: String,
}
#[derive(Debug, serde::Serialize)]
struct WarriorList {
    warriors: Vec<Warrior>,
}
#[derive(Debug, serde::Serialize)]
struct Warrior {
    pub id: i32,
    pub name: String,
    pub hill: String,
    pub hill_id: i32,
    pub author: String,
    pub author_id: i32,
    pub redcode: String,
}

fn to_id(s: &RawStr) -> Result<i32, Error> {
    Ok(s.url_decode()?.parse::<i32>()?)
}

#[catch(404)]
fn not_found() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("notfound", &context)
}
#[get("/")]
fn home() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("home", &context)
}
#[get("/warrior/<id>")]
fn get_warrior(conn: DbConn, id: &RawStr) -> Result<Template, Status> {
    let id = to_id(id)?;
    let w = db::get_warrior_by_id(&conn.0, id)?;
    let h = db::get_hill_by_id(&conn.0, w.hill)?;
    let a = db::get_author_by_id(&conn.0, w.author)?;
    Ok(Template::render(
        "warrior",
        &Warrior {
            id,
            name: w.name,
            hill: h.name,
            hill_id: h.id,
            author: a.name,
            author_id: a.id,
            redcode: w.redcode,
        },
    ))
}
#[get("/warrior")]
fn create_warrior_form() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("warrior-create", &context)
}
#[post("/warrior", data = "<warrior>")]
fn create_warrior(conn: DbConn, warrior: Form<WarriorFormInput>) -> Result<Redirect, Status> {
    let data = parse_warrior_code(warrior.redcode.as_str());
    let h = db::get_hill_by_key(&conn.0, data.hill_key.as_str())?;
    let a = db::create_author(
        &conn.0,
        &models::NewAuthor {
            name: data.author.as_str(),
        },
    )?;
    let w = db::create_warrior(
        &conn.0,
        &models::NewWarrior {
            name: data.warrior.as_str(),
            hill: h.id,
            author: a.id,
            redcode: warrior.redcode.as_str(),
        },
    )?;
    let climb = models::NewClimb {
        warrior: w.id,
        hill: h.id,
        status: models::ClimbStatus::Pending as i32,
    };
    db::create_climb(&conn.0, &climb)?;
    Ok(Redirect::to(uri!(get_warrior: w.id.to_string())))
}
#[get("/warriors")]
fn list_warriors(conn: DbConn) -> Result<Template, Status> {
    let context = WarriorList {
        warriors: db::get_warriors(&conn.0)?
            .drain(..)
            .map(|w| -> Result<Warrior, Error> {
                let h = db::get_hill_by_id(&conn.0, w.hill)?;
                let a = db::get_author_by_id(&conn.0, w.author)?;
                Ok(Warrior {
                    id: w.id,
                    name: w.name,
                    hill: h.name,
                    hill_id: h.id,
                    author: a.name,
                    author_id: a.id,
                    redcode: w.redcode,
                })
            })
            .collect::<Result<Vec<Warrior>, Error>>()?,
    };
    Ok(Template::render("warrior-index", &context))
}

#[derive(Debug, FromForm, serde::Serialize)]
struct HillFormInput {
    pub name: String,
    pub key: String,
    pub instruction_set: i32,
    pub core_size: i32,
    pub max_cycles: i32,
    pub max_processes: i32,
    pub max_warrior_length: i32,
    pub min_distance: i32,
    pub rounds: i32,
    pub slots: i32,
}
#[derive(Debug, serde::Serialize)]
struct HillList {
    hills: Vec<models::Hill>,
}

#[derive(Debug, serde::Serialize)]
struct HillWarriors {
    pub hill: models::Hill,
    pub warriors: Vec<HillWarrior>,
}
#[derive(Debug, serde::Serialize)]
struct HillWarrior {
    pub id: i32,
    pub name: String,
    pub author: String,
    pub author_id: i32,
    pub rank: i32,
    pub win: f32,
    pub loss: f32,
    pub tie: f32,
    pub score: f32,
}

#[get("/hill/<id>")]
fn get_hill(conn: DbConn, id: &RawStr) -> Result<Template, Status> {
    let id = to_id(id)?;
    let h = db::get_hill_by_id(&conn.0, id)?;
    let mut ws = db::get_warriors_on_hill(&conn.0, id)?;
    let mut hw = HillWarriors {
        hill: h,
        warriors: ws
            .drain(..)
            .map(|hw| -> Result<HillWarrior, Error> {
                let w = db::get_warrior_by_id(&conn.0, hw.warrior)?;
                let a = db::get_author_by_id(&conn.0, w.author)?;
                Ok(HillWarrior {
                    id: w.id,
                    name: w.name,
                    author: a.name,
                    author_id: a.id,
                    rank: hw.rank,
                    win: hw.win,
                    loss: hw.loss,
                    tie: hw.tie,
                    score: hw.score,
                })
            })
            .collect::<Result<Vec<HillWarrior>, Error>>()?,
    };
    hw.warriors
        .sort_by(|a, b| a.rank.partial_cmp(&b.rank).unwrap());
    Ok(Template::render("hill", &hw))
}
#[get("/hill")]
fn create_hill_form() -> Result<Template, Status> {
    let context: HashMap<&str, &str> = HashMap::new();
    Ok(Template::render("hill-create", &context))
}
#[post("/hill", data = "<hill>")]
fn create_hill(conn: DbConn, hill: Form<HillFormInput>) -> Result<Redirect, Status> {
    let h = models::NewHill {
        name: hill.name.as_str(),
        key: hill.key.as_str(),
        instruction_set: hill.instruction_set,
        core_size: hill.core_size,
        max_cycles: hill.max_cycles,
        max_processes: hill.max_processes,
        max_warrior_length: hill.max_warrior_length,
        min_distance: hill.min_distance,
        rounds: hill.rounds,
        slots: hill.slots,
    };
    let hill = db::create_hill(&conn.0, &h)?;
    Ok(Redirect::to(uri!(get_hill: hill.id.to_string())))
}
#[get("/hills")]
fn list_hills(conn: DbConn) -> Result<Template, Status> {
    let context = HillList {
        hills: db::get_hills(&conn.0)?,
    };
    Ok(Template::render("hill-index", &context))
}
#[derive(Debug, FromForm, serde::Serialize)]
struct ClimbFormInput {
    pub hill: String,
    pub warrior: String,
}
#[derive(Debug, serde::Serialize)]
struct ClimbList {
    unfinished: bool,
    climbs: Vec<Climb>,
}
#[derive(Debug, serde::Serialize)]
struct Climb {
    pub id: i32,
    pub hill: String,
    pub hill_id: i32,
    pub warrior: String,
    pub warrior_id: i32,
    pub status: String,
}
#[get("/climbs?<unfinished>")]
fn list_climbs(conn: DbConn, unfinished: bool) -> Result<Template, Status> {
    let mut climbs = match unfinished {
        true => db::get_unfinished_climbs(&conn.0)?,
        false => db::get_climbs(&conn.0)?,
    };
    let context = ClimbList {
        unfinished: unfinished,
        climbs: climbs
            .drain(..)
            .map(|c| -> Result<Climb, Error> {
                let h = db::get_hill_by_id(&conn.0, c.hill)?;
                let w = db::get_warrior_by_id(&conn.0, c.warrior)?;
                Ok(Climb {
                    id: c.id,
                    hill: h.name,
                    hill_id: h.id,
                    warrior: w.name,
                    warrior_id: w.id,
                    status: match c.status {
                        0 => "Pending".to_string(),
                        1 => "In Progress".to_string(),
                        2 => "Finished".to_string(),
                        3 => "Failed".to_string(),
                        _ => "Unknown".to_string(),
                    },
                })
            })
            .collect::<Result<Vec<Climb>, Error>>()?,
    };
    Ok(Template::render("climb-index", &context))
}
#[derive(Debug, serde::Serialize)]
struct AuthorList {
    authors: Vec<models::Author>,
}
#[derive(Debug, serde::Serialize)]
struct Author {
    author: models::Author,
    warriors: Vec<AuthorWarrior>,
}
#[derive(Debug, serde::Serialize)]
struct AuthorWarrior {
    id: i32,
    name: String,
    hill: String,
    hill_id: i32,
    rank: i32,
}

#[get("/author/<id>")]
fn get_author(conn: DbConn, id: &RawStr) -> Result<Template, Status> {
    let id = to_id(id)?;
    let a = db::get_author_by_id(&conn.0, id)?;
    let context = Author {
        author: a,
        warriors: db::get_warriors_from_author(&conn.0, id)?
            .drain(..)
            .map(|w| -> Result<AuthorWarrior, Error> {
                let h = db::get_hill_by_id(&conn.0, w.hill)?;
                let rank = match db::get_warrior_from_hill(&conn.0, w.hill, w.id) {
                    Ok(hw) => hw.rank,
                    Err(e) if e.is_not_found() => -1,
                    Err(e) => return Err(e),
                };
                Ok(AuthorWarrior {
                    id: w.id,
                    name: w.name,
                    hill: h.name,
                    hill_id: h.id,
                    rank: rank,
                })
            })
            .collect::<Result<Vec<AuthorWarrior>, Error>>()?,
    };
    Ok(Template::render("author", &context))
}
#[get("/authors")]
fn list_authors(conn: DbConn) -> Result<Template, Status> {
    let context = AuthorList {
        authors: db::get_authors(&conn.0)?,
    };
    Ok(Template::render("author-index", &context))
}

#[derive(Debug)]
struct ParsedData {
    pub warrior: String,
    pub hill_key: String,
    pub author: String,
}

fn parse_warrior_code(redcode: &str) -> ParsedData {
    lazy_static! {
        // [ -~] is the rang of all printable ascii chars
        static ref NAME_RE: Regex = Regex::new(";name ([ -~]+)").unwrap();
        static ref AUTHOR_RE: Regex = Regex::new(";author ([ -~]+)").unwrap();
        static ref HILL_RE: Regex = Regex::new(";redcode-([ -~]+)").unwrap();
    }
    let mut data = ParsedData {
        warrior: String::new(),
        hill_key: String::new(),
        author: String::new(),
    };
    for n in NAME_RE.captures_iter(redcode) {
        data.warrior = n[1].to_string();
    }
    for h in HILL_RE.captures_iter(redcode) {
        data.hill_key = h[1].to_string();
    }
    for a in AUTHOR_RE.captures_iter(redcode) {
        data.author = a[1].to_string();
    }
    data
}

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
embed_migrations!();

fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn = DbConn::get_one(&rocket).expect("database connection");
    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            error!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    }
}

fn main() {
    let _ = thread::spawn(|| {
        hill::run();
    });

    rocket::ignite()
        .attach(Template::fairing())
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
        .register(catchers![not_found])
        .mount(
            "/",
            routes![
                home,
                get_warrior,
                create_warrior_form,
                create_warrior,
                list_warriors,
                get_hill,
                create_hill_form,
                create_hill,
                list_hills,
                list_climbs,
                get_author,
                list_authors,
            ],
        )
        .launch();
}
