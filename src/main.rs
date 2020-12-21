#![feature(proc_macro_hygiene, decl_macro)]

mod db;
mod models;
mod schema;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;

#[macro_use]
extern crate rocket;
use rocket::http::RawStr;
use rocket::request::Form;
use rocket::response::Redirect;

#[macro_use]
extern crate rocket_contrib;
use rocket_contrib::templates::Template;

use std::collections::HashMap;

#[database("sqlite_db")]
struct DbConn(diesel::SqliteConnection);

#[derive(Debug, FromForm, serde::Serialize)]
struct WarriorFormInput {
    pub redcode: String,
}
#[derive(Debug, serde::Serialize)]
struct WarriorList {
    warriors: Vec<models::Warrior>,
}

#[get("/warrior/<name>")]
fn get_warrior(conn: DbConn, name: &RawStr) -> Template {
    let w = db::get_warrior(&conn.0, name);
    Template::render("warrior", &w)
}
#[get("/warrior")]
fn create_warrior_form() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("warrior-create", &context)
}
#[post("/warrior", data = "<warrior>")]
fn create_warrior(conn: DbConn, warrior: Form<WarriorFormInput>) -> Redirect {
    let w = parse_warrior_code(warrior.redcode.as_str());
    db::create_warrior(&conn.0, &w);
    Redirect::to(uri!(get_warrior: w.name))
}
#[get("/warriors")]
fn list_warriors(conn: DbConn) -> Template {
    let context = WarriorList {
        warriors: db::get_warriors(&conn.0),
    };
    Template::render("warrior-index", &context)
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
    pub warriors: Vec<Option<HillWarrior>>,
}
#[derive(Debug, serde::Serialize)]
struct HillWarrior {
    pub warrior: String,
    pub author: String,
    pub rank: i32,
    pub win: f32,
    pub loss: f32,
    pub tie: f32,
    pub score: f32,
}

#[get("/hill/<name>")]
fn get_hill(conn: DbConn, name: &RawStr) -> Template {
    let (h, ws) = db::get_warriors_on_hill(&conn.0, name).expect("No hill found");
    let hw = HillWarriors {
        hill: h,
        warriors: ws
            .iter()
            .map(|hw| match db::get_warrior_by_id(&conn.0, hw.id) {
                None => None,
                Some(w) => Some(HillWarrior {
                    warrior: w.name,
                    author: w.author,
                    rank: hw.rank,
                    win: hw.win,
                    loss: hw.loss,
                    tie: hw.tie,
                    score: hw.score,
                }),
            })
            .collect(),
    };
    Template::render("hill", &hw)
}
#[get("/hill")]
fn create_hill_form() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("hill-create", &context)
}
#[post("/hill", data = "<hill>")]
fn create_hill(conn: DbConn, hill: Form<HillFormInput>) -> Redirect {
    let h = models::NewHill {
        name: hill.name.clone(),
        key: hill.key.clone(),
        instruction_set: hill.instruction_set,
        core_size: hill.core_size,
        max_cycles: hill.max_cycles,
        max_processes: hill.max_processes,
        max_warrior_length: hill.max_warrior_length,
        min_distance: hill.min_distance,
        rounds: hill.rounds,
        slots: hill.slots,
    };
    db::create_hill(&conn.0, &h);
    Redirect::to(uri!(get_hill: h.name))
}
#[get("/hills")]
fn list_hills(conn: DbConn) -> Template {
    let context = HillList {
        hills: db::get_hills(&conn.0),
    };
    Template::render("hill-index", &context)
}
#[derive(Debug, FromForm, serde::Serialize)]
struct ClimbFormInput {
    pub hill: String,
    pub warrior: String,
}
#[derive(Debug, serde::Serialize)]
struct ClimbList {
    climbs: Vec<Climb>,
}
#[derive(Debug, serde::Serialize)]
struct Climb {
    pub id: i32,
    pub hill: String,
    pub warrior: String,
    pub status: String,
}
#[derive(Debug, serde::Serialize)]
struct CreateClimb {
    pub hills: Vec<models::Hill>,
    pub warriors: Vec<models::Warrior>,
}
#[get("/climb")]
fn create_climb_form(conn: DbConn) -> Template {
    let context = CreateClimb {
        hills: db::get_hills(&conn.0),
        warriors: db::get_warriors(&conn.0),
    };
    Template::render("climb-create", &context)
}
#[post("/climb", data = "<climb>")]
fn create_climb(conn: DbConn, climb: Form<ClimbFormInput>) -> Redirect {
    let h = db::get_hill(&conn.0, climb.hill.as_str()).expect("Hill not found");
    let w = db::get_warrior(&conn.0, climb.warrior.as_str()).expect("Warrior not found");
    let c = models::NewClimb {
        hill: h.id,
        warrior: w.id,
        status: 0,
    };
    db::create_climb(&conn.0, &c);
    Redirect::to(uri!(list_climbs))
}
#[get("/climbs")]
fn list_climbs(conn: DbConn) -> Template {
    let context = ClimbList {
        climbs: db::get_climbs(&conn.0)
            .iter()
            .map(|c| {
                let h = db::get_hill_by_id(&conn.0, c.hill).expect("Hill not found");
                let w = db::get_warrior_by_id(&conn.0, c.warrior).expect("Warrior not found");
                Climb {
                    id: c.id,
                    hill: h.name,
                    warrior: w.name,
                    status: match c.status {
                        0 => "Pending".to_string(),
                        1 => "Inprogress".to_string(),
                        2 => "Finished".to_string(),
                        _ => "Unknown".to_string(),
                    },
                }
            })
            .collect(),
    };
    Template::render("climb-index", &context)
}
fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .attach(DbConn::fairing())
        .mount(
            "/",
            routes![
                get_warrior,
                create_warrior_form,
                create_warrior,
                list_warriors,
                get_hill,
                create_hill_form,
                create_hill,
                list_hills,
                create_climb_form,
                create_climb,
                list_climbs,
            ],
        )
        .launch();
}

fn parse_warrior_code(redcode: &str) -> models::NewWarrior {
    lazy_static! {
        static ref NAME_RE: Regex = Regex::new(";name ([a-zA-Z0-9 ]+)").unwrap();
        static ref AUTHOR_RE: Regex = Regex::new(";author ([a-zA-Z0-9 ]+)").unwrap();
    }
    let mut name = String::new();
    for n in NAME_RE.captures_iter(redcode) {
        name = n[1].to_string();
    }
    let mut author = String::new();
    for a in AUTHOR_RE.captures_iter(redcode) {
        author = a[1].to_string();
    }
    models::NewWarrior {
        name,
        author,
        redcode: redcode.to_string(),
    }
}
