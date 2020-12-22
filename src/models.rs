use crate::schema::warriors;

#[derive(Debug, serde::Serialize, Identifiable, Queryable)]
pub struct Warrior {
    pub id: i32,
    pub name: String,
    pub hill: i32,
    pub author: i32,
    pub redcode: String,
}
#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "warriors"]
pub struct NewWarrior<'a> {
    pub name: &'a str,
    pub hill: i32,
    pub author: i32,
    pub redcode: &'a str,
}

use crate::schema::hills;

#[derive(Debug, serde::Serialize, Identifiable, Queryable)]
pub struct Hill {
    pub id: i32,
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
#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "hills"]
pub struct NewHill {
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

use crate::schema::hill_warriors;

#[derive(Debug, serde::Serialize, Identifiable, Queryable)]
pub struct HillWarrior {
    pub id: i32,
    pub hill: i32,
    pub warrior: i32,
    pub rank: i32,
    pub win: f32,
    pub loss: f32,
    pub tie: f32,
    pub score: f32,
}
#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "hill_warriors"]
pub struct NewHillWarrior {
    pub hill: i32,
    pub warrior: i32,
    pub rank: i32,
    pub win: f32,
    pub loss: f32,
    pub tie: f32,
    pub score: f32,
}

use crate::schema::climbs;

#[derive(Debug, serde::Serialize, Identifiable, Queryable)]
pub struct Climb {
    pub id: i32,
    pub hill: i32,
    pub warrior: i32,
    pub status: i32,
}
#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "climbs"]
pub struct NewClimb {
    pub hill: i32,
    pub warrior: i32,
    pub status: i32,
}

pub enum ClimbStatus {
    Pending,
    InProgress,
    Finished,
    Failed,
}


use crate::schema::authors;

#[derive(Debug, serde::Serialize, Identifiable, Queryable)]
pub struct Author {
    pub id: i32,
    pub name: String,
}
#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "authors"]
pub struct NewAuthor<'a> {
    pub name: &'a str,
}
