use serde::{Serialize, Deserialize};

pub mod messaging;
pub mod gamedata;

pub const GENERATE : &str = "A3A3";

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub draft : bool,
    pub gen_players : bool
}

impl Settings {
    pub fn new() -> Settings {
        Settings { draft: true, gen_players : true }
    } 
}

