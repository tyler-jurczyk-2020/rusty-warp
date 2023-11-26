use std::{net::SocketAddr, path::PathBuf, collections::HashMap, cell::RefCell, convert::Infallible, sync::{Arc, Mutex}, task::Poll, time::Duration, vec::IntoIter};
use warp::{Filter, filters::{ws::{Message, Ws, WebSocket}, body}, reply::Reply, reject::Rejection, Error};
use futures::{StreamExt, FutureExt, SinkExt, TryFutureExt, Future, future::Map, stream::{SplitSink, Iter}, Stream, TryStream};
use warp::hyper::body::Bytes;
use tokio::{sync::{mpsc::{self, UnboundedSender, UnboundedReceiver}, watch::{self, Sender, Receiver}}, stream};
use serde::{Serialize, Deserialize};

use crate::events::run_match;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Batch {
    pub play_action : Vec<Vec<f64>>
} 

impl Batch {
    pub fn new() -> Batch {
        Batch { play_action: Vec::new() }
    }
}

pub struct GlobalComms {
    pub send_to_py : Option<UnboundedSender<()>>,
    pub recv_from_py : Option<Receiver<usize>>
}

impl GlobalComms {
    pub fn new() -> GlobalComms {
        GlobalComms { send_to_py : None, recv_from_py : None}
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExternMessage {
    preflight : String,
    contents : String
}

impl ExternMessage {
    pub fn new(preflight : String, contents : impl serde::Serialize) -> ExternMessage{
            let p = preflight;
            let c = serde_json::to_string(&contents).unwrap();
            ExternMessage{preflight : p, contents : c}    
    }
    pub async fn send_message(&self, sender : &mut SplitSink<WebSocket, Message>) {
        let serialized = serde_json::to_string(self).unwrap();
        sender.send(Message::text(serialized)).await;
    }

}

#[derive(Clone, Serialize, Deserialize)]
pub struct InternMessage {
    pub msg : String
}

impl InternMessage {
    pub fn new(msg : String) -> InternMessage {
        InternMessage { msg }
    }
    pub async fn send_message(&self, sender : &mut UnboundedSender<InternMessage>) {
        let to_send = self.clone();
        sender.send(to_send);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub batch : Batch,
    pub player_count : usize,    
    pub batch_size : usize,
    pub players : Vec<Player>,
    pub display1 : f64,
    pub display2 : f64 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    mean : f64,
    std_dev : f64,
    pub photo : String 
}

impl Player {
    fn new(mean : f64, std_dev : f64, photo : String) -> Player {
        Player { mean, std_dev, photo}
    }
}

impl Data {
    pub fn new() -> Data {
        Data { 
            batch : Batch::new(),
            player_count : 2,
            batch_size : 40,
            display1 : 0.0,
            display2 : 0.0,
            players : vec![Player::new(0.2, 0.1, "a4.png".to_string()), Player::new(0.1, 0.1, "a2.png".to_string())]
        }
    }
}


pub const GENERATE : &str = "A3A3";

