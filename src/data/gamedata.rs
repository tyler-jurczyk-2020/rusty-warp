use std::collections::HashMap;

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub name : String,
    mean : f64,
    std_dev : f64,
    pub photo : String 
}

impl Player {
    fn new(name : String, mean : f64, std_dev : f64, photo : String) -> Player {
        Player { name, mean, std_dev, photo}
    }
}

pub struct Pool {
    pool : Vec<Player>    
}


// structure to hold data the should be available to all threads
#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
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
            batch : Batch::new(),
            player_count : 2,
            batch_size : 40,
            display1 : 0.0,
            display2 : 0.0,
            players : vec![Player::new("none1".to_string(), 0.2, 0.1, "a4.png".to_string()), Player::new("none2,".to_string(), 0.1, 0.1, "a2.png".to_string())]
        }
    }
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
