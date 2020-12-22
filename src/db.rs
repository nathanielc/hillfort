use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::error::{Code, Error};
use crate::models::*;

pub fn get_warrior<'a>(conn: &SqliteConnection, wname: &'a str) -> Result<Warrior, Error> {
    use crate::schema::warriors::dsl::*;
    let mut list = warriors.filter(name.eq(wname)).load::<Warrior>(conn)?;
    match list.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn get_warrior_by_id<'a>(conn: &SqliteConnection, wid: i32) -> Result<Warrior, Error> {
    use crate::schema::warriors::dsl::*;
    let mut ws = warriors.filter(id.eq(wid)).load::<Warrior>(conn)?;
    match ws.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn get_warriors(conn: &SqliteConnection) -> Result<Vec<Warrior>, Error> {
    use crate::schema::warriors::dsl::*;
    Ok(warriors.load::<Warrior>(conn)?)
}

pub fn create_warrior(conn: &SqliteConnection, w: &NewWarrior) -> Result<Warrior, Error> {
    use crate::schema::warriors;
    let _ = match get_warrior(conn, w.name) {
        Ok(existing) => diesel::update(&existing).set(w).execute(conn)?,
        Err(e) if e.is_not_found() => diesel::insert_into(warriors::table)
            .values(w)
            .execute(conn)?,
        Err(e) => return Err(e),
    };
    get_warrior(conn, w.name)
}
pub fn delete_warrior<'a>(conn: &SqliteConnection, wname: &'a str) -> Result<(), Error> {
    use crate::schema::warriors::dsl::*;
    let _ = diesel::delete(warriors.filter(name.eq(wname))).execute(conn)?;
    Ok(())
}

pub fn get_hill<'a>(conn: &SqliteConnection, hname: &'a str) -> Result<Hill, Error> {
    use crate::schema::hills::dsl::*;
    let mut list = hills.filter(name.eq(hname)).load::<Hill>(conn)?;
    match list.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn get_hill_by_id<'a>(conn: &SqliteConnection, hid: i32) -> Result<Hill, Error> {
    use crate::schema::hills::dsl::*;
    let mut list = hills.filter(id.eq(hid)).load::<Hill>(conn)?;
    match list.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn get_hill_by_key<'a>(conn: &SqliteConnection, hkey: &'a str) -> Result<Hill, Error> {
    use crate::schema::hills::dsl::*;
    let mut list = hills.filter(key.eq(hkey)).load::<Hill>(conn)?;
    match list.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn get_hills(conn: &SqliteConnection) -> Result<Vec<Hill>, Error> {
    use crate::schema::hills::dsl::*;
    Ok(hills.load::<Hill>(conn)?)
}
pub fn create_hill(conn: &SqliteConnection, h: &NewHill) -> Result<Hill, Error> {
    use crate::schema::hills;
    let _ = match get_hill(conn, h.name.as_str()) {
        Ok(existing) => diesel::update(&existing).set(h).execute(conn)?,
        Err(e) if e.is_not_found() => diesel::insert_into(hills::table).values(h).execute(conn)?,
        Err(e) => return Err(e),
    };
    get_hill(conn, h.name.as_str())
}
pub fn get_warriors_on_hill<'a>(
    conn: &SqliteConnection,
    hid: i32,
) -> Result<Vec<HillWarrior>, Error> {
    use crate::schema::hill_warriors::dsl::*;
    match get_hill_by_id(conn, hid) {
        Ok(h) => Ok(hill_warriors
            .filter(hill.eq(h.id))
            .load::<HillWarrior>(conn)?),
        Err(e) => return Err(e),
    }
}
pub fn get_warrior_from_hill(
    conn: &SqliteConnection,
    hid: i32,
    wid: i32,
) -> Result<HillWarrior, Error> {
    use crate::schema::hill_warriors::dsl::*;
    let mut list = hill_warriors
        .filter(warrior.eq(wid))
        .filter(hill.eq(hid))
        .load::<HillWarrior>(conn)?;
    match list.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn delete_warriors_on_hill<'a>(conn: &SqliteConnection, hid: i32) -> Result<(), Error> {
    use crate::schema::hill_warriors::dsl::*;
    diesel::delete(hill_warriors.filter(hill.eq(hid))).execute(conn)?;
    Ok(())
}
pub fn create_hill_warrior(conn: &SqliteConnection, hw: &NewHillWarrior) -> Result<(), Error> {
    use crate::schema::hill_warriors;
    diesel::insert_into(hill_warriors::table)
        .values(hw)
        .execute(conn)?;
    Ok(())
}

pub fn get_climb_by_id<'a>(conn: &SqliteConnection, wid: i32) -> Result<Climb, Error> {
    use crate::schema::climbs::dsl::*;
    let mut list = climbs.filter(id.eq(wid)).load::<Climb>(conn)?;
    match list.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn get_climbs(conn: &SqliteConnection) -> Result<Vec<Climb>, Error> {
    use crate::schema::climbs::dsl::*;
    Ok(climbs.load::<Climb>(conn)?)
}
pub fn get_unfinished_climbs(conn: &SqliteConnection) -> Result<Vec<Climb>, Error> {
    use crate::schema::climbs::dsl::*;
    Ok(climbs
        .filter(
            status
                .eq(ClimbStatus::Pending as i32)
                .or(status.eq(ClimbStatus::InProgress as i32)),
        )
        .load::<Climb>(conn)?)
}
pub fn get_pending_climbs(conn: &SqliteConnection) -> Result<Vec<Climb>, Error> {
    use crate::schema::climbs::dsl::*;
    Ok(climbs
        .filter(status.eq(ClimbStatus::Pending as i32))
        .load::<Climb>(conn)?)
}
pub fn update_inprogress_climbs_to_pending(conn: &SqliteConnection) -> Result<(), Error> {
    use crate::schema::climbs::dsl::*;
    diesel::update(climbs.filter(status.eq(ClimbStatus::InProgress as i32)))
        .set(status.eq(0))
        .execute(conn)?;
    Ok(())
}
pub fn create_climb(conn: &SqliteConnection, c: &NewClimb) -> Result<(), Error> {
    use crate::schema::climbs;
    diesel::insert_into(climbs::table).values(c).execute(conn)?;
    Ok(())
}
pub fn update_climb_status<'a>(
    conn: &SqliteConnection,
    cid: i32,
    newstatus: i32,
) -> Result<(), Error> {
    use crate::schema::climbs::dsl::*;
    diesel::update(climbs.filter(id.eq(cid)))
        .set(status.eq(newstatus))
        .execute(conn)?;
    Ok(())
}

pub fn get_author<'a>(conn: &SqliteConnection, aname: &'a str) -> Result<Author, Error> {
    use crate::schema::authors::dsl::*;
    let mut list = authors.filter(name.eq(aname)).load::<Author>(conn)?;
    match list.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn get_author_by_id<'a>(conn: &SqliteConnection, aid: i32) -> Result<Author, Error> {
    use crate::schema::authors::dsl::*;
    let mut list = authors.filter(id.eq(aid)).load::<Author>(conn)?;
    match list.pop() {
        Some(w) => Ok(w),
        None => Err(Error {
            code: Code::NotFound,
        }),
    }
}
pub fn get_authors(conn: &SqliteConnection) -> Result<Vec<Author>, Error> {
    use crate::schema::authors::dsl::*;
    Ok(authors.load::<Author>(conn)?)
}
pub fn create_author(conn: &SqliteConnection, a: &NewAuthor) -> Result<Author, Error> {
    use crate::schema::authors;
    match get_author(conn, a.name) {
        Ok(existing) => {
            // do nothing authors only have a single unique column
            Ok(existing)
        }
        Err(e) if e.is_not_found() => {
            diesel::insert_into(authors::table)
                .values(a)
                .execute(conn)?;
            Ok(get_author(conn, a.name)?)
        }
        Err(e) => return Err(e),
    }
}
pub fn get_warriors_from_author(conn: &SqliteConnection, aid: i32) -> Result<Vec<Warrior>, Error> {
    use crate::schema::warriors::dsl::*;
    Ok(warriors.filter(author.eq(aid)).load::<Warrior>(conn)?)
}
