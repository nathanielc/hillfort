use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::models::*;

pub fn get_warrior<'a>(conn: &SqliteConnection, wname: &'a str) -> Option<Warrior> {
    use crate::schema::warriors::dsl::*;
    let mut ws = warriors
        .filter(name.eq(wname))
        .load::<Warrior>(conn)
        .expect("Error loading warrior");
    ws.pop()
}
pub fn get_warrior_by_id<'a>(conn: &SqliteConnection, wid: i32) -> Option<Warrior> {
    use crate::schema::warriors::dsl::*;
    let mut ws = warriors
        .filter(id.eq(wid))
        .load::<Warrior>(conn)
        .expect("Error loading warrior by id");
    ws.pop()
}
pub fn get_warriors(conn: &SqliteConnection) -> Vec<Warrior> {
    use crate::schema::warriors::dsl::*;
    warriors
        .load::<Warrior>(conn)
        .expect("Error loading warriors")
}

pub fn create_warrior(conn: &SqliteConnection, w: &NewWarrior) -> usize {
    use crate::schema::warriors;
    match get_warrior(conn, w.name.as_str()) {
        None => diesel::insert_into(warriors::table)
            .values(w)
            .execute(conn)
            .expect("Error saving warrior"),
        Some(existing) => diesel::update(&existing)
            .set(w)
            .execute(conn)
            .expect("Error could not replace warrior"),
    }
}
pub fn delete_warrior<'a>(conn: &SqliteConnection, wname: &'a str) -> usize {
    use crate::schema::warriors::dsl::*;
    diesel::delete(warriors.filter(name.eq(wname)))
        .execute(conn)
        .expect("Error loading warrior")
}

pub fn get_hill<'a>(conn: &SqliteConnection, hname: &'a str) -> Option<Hill> {
    use crate::schema::hills::dsl::*;
    let mut hs = hills
        .filter(name.eq(hname))
        .load::<Hill>(conn)
        .expect("Error loading hill");
    hs.pop()
}
pub fn get_hill_by_id<'a>(conn: &SqliteConnection, hid: i32) -> Option<Hill> {
    use crate::schema::hills::dsl::*;
    let mut hs = hills
        .filter(id.eq(hid))
        .load::<Hill>(conn)
        .expect("Error loading hill by id");
    hs.pop()
}
pub fn get_hills(conn: &SqliteConnection) -> Vec<Hill> {
    use crate::schema::hills::dsl::*;
    hills.load::<Hill>(conn).expect("Error loading hills")
}
pub fn create_hill(conn: &SqliteConnection, h: &NewHill) -> usize {
    println!("Hill {:?}", h);
    use crate::schema::hills;
    match get_hill(conn, h.name.as_str()) {
        None => diesel::insert_into(hills::table)
            .values(h)
            .execute(conn)
            .expect("Error saving hill"),
        Some(existing) => diesel::update(&existing)
            .set(h)
            .execute(conn)
            .expect("Error could not replace hill"),
    }
}
pub fn get_warriors_on_hill<'a>(
    conn: &SqliteConnection,
    hname: &'a str,
) -> Option<(Hill, Vec<HillWarrior>)> {
    use crate::schema::hill_warriors::dsl::*;
    match get_hill(conn, hname) {
        None => None,
        Some(h) => {
            let hs = hill_warriors
                .filter(hill.eq(h.id))
                .load::<HillWarrior>(conn)
                .expect("Error loading hill_warrior");
            Some((h, hs))
        }
    }
}

pub fn get_climb_by_id<'a>(conn: &SqliteConnection, wid: i32) -> Option<Climb> {
    use crate::schema::climbs::dsl::*;
    let mut ws = climbs
        .filter(id.eq(wid))
        .load::<Climb>(conn)
        .expect("Error loading climb by id");
    ws.pop()
}
pub fn get_climbs(conn: &SqliteConnection) -> Vec<Climb> {
    use crate::schema::climbs::dsl::*;
    climbs.load::<Climb>(conn).expect("Error loading climbs")
}
pub fn create_climb(conn: &SqliteConnection, c: &NewClimb) -> usize {
    use crate::schema::climbs;
    diesel::insert_into(climbs::table)
        .values(c)
        .execute(conn)
        .expect("Error saving climb")
}
pub fn update_climb_status<'a>(conn: &SqliteConnection, cid: i32, newstatus: i32) -> usize {
    use crate::schema::climbs::dsl::*;
    diesel::update(climbs.filter(id.eq(cid)))
        .set(status.eq(newstatus))
        .execute(conn)
        .expect("Error updating climb status")
}
