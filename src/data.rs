use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Batch {
    play_action : Vec<Vec<f64>>
} 

impl Batch {
    pub fn new() -> Batch {
        Batch { play_action: Vec::new() }
    }
}
