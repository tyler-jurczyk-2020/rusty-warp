use std::sync::{Mutex, Arc};

use serde::{Serialize, Deserialize};

use self::{gamedata::Data, messaging::GlobalComms, browser_comms::BrowserData};

pub mod messaging;
pub mod gamedata;
pub mod browser_comms;

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

pub struct SharedData {
    pub game : Arc<Mutex<Data>>,
    pub comms : Arc<Mutex<GlobalComms>>,
    pub browser : Arc<Mutex<BrowserData>>
}

impl SharedData {
    pub fn new() -> SharedData {
        SharedData { game: Arc::new(Mutex::new(Data::new())), comms: Arc::new(Mutex::new(GlobalComms::new())), browser: Arc::new(Mutex::new(BrowserData::new())) }
    }
}

