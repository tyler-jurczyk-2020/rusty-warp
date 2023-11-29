use std::collections::HashMap;

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name : String,
    mean : f64,
    std_dev : f64,
    pub photo : String, 
    pub draftability : f64,
    pick : u32
}

impl Player {
    fn new(name : String, mean : f64, std_dev : f64, photo : String, draftability : f64, pick : u32) -> Player {
        Player { name, mean, std_dev, photo, draftability, pick}
    }
}

pub struct Pool {
    pool : Vec<Player>    
}

#[derive(Debug)]
pub struct Team {
    pub members : Vec<Player>
}

impl Team {
    pub fn new() -> Team {
        Team { members: Vec::new() }
    }
}


// structure to hold data the should be available to all threads
#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub player_pool : HashMap<String, Player>,
    pub state : State,
    pub batch : Batch,
    pub player_count : usize,    
    pub batch_size : usize,
    pub players : Vec<Player>,
    pub display1 : f64,
    pub display2 : f64 
}

impl Data {
    pub fn new() -> Data {
        Data { 
            player_pool : HashMap::new(),
            state : State::Undefined,
            batch : Batch::new(),
            player_count : 2,
            batch_size : 40,
            display1 : 0.0,
            display2 : 0.0,
            players : vec![Player::new("none1".to_string(), 0.2, 0.1, "a4.png".to_string(), 0.0, 1), Player::new("none2,".to_string(), 0.1, 0.1, "a2.png".to_string(), 0.0, 2)]
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    Draft,
    Season,
    Undefined
}




#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Batch {
    pub play_action : Vec<Vec<f64>>
} 

impl Batch {
    pub fn new() -> Batch {
        Batch { play_action: Vec::new() }
    }
}
