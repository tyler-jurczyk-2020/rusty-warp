use super::gamedata::{State, Player};

pub enum PageType {
    DraftIP(DraftResp) 
}

pub struct DraftResp {
    player_pool : Vec<Player> 
}

impl DraftResp {
    pub fn new() -> DraftResp {
        DraftResp { player_pool: Vec::new() }
    }
}


pub struct BrowserData {
    
}

impl BrowserData {
    pub fn new() -> BrowserData {
        BrowserData {  }
    }
}
