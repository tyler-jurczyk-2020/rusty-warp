use warp::filters::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use serde::{Serialize, Deserialize};


pub struct GlobalComms {
    pub send_to_py : Option<UnboundedSender<InternMessage>>,
    pub recv_from_py : Option<UnboundedReceiver<InternMessage>>,
    pub send_to_brow : Option<UnboundedSender<InternMessage>>,
    pub send_to_main : Option<UnboundedSender<InternMessage>>
}

impl GlobalComms {
    pub fn new() -> GlobalComms {
        GlobalComms { send_to_py : None, send_to_brow : None, recv_from_py : None, send_to_main : None}
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ExternMessage {
    pub preflight : String,
    pub contents : Option<String>
}

impl ExternMessage {
    pub fn new(preflight : String, contents : impl serde::Serialize) -> ExternMessage{
            let p = preflight;
            let c = serde_json::to_string(&contents).unwrap();
            ExternMessage{preflight : p, contents : Some(c)}    
    }
    pub async fn send_message(&self, sender : &mut SplitSink<WebSocket, Message>) {
        let serialized = serde_json::to_string(self).unwrap();
        sender.send(Message::text(serialized)).await;
    }
}

impl From<InternMessage> for ExternMessage {
    fn from(value: InternMessage) -> Self {
        ExternMessage { preflight: value.code.unwrap(), contents: value.msg } 
    }
}

impl From<ExternMessage> for InternMessage {
    fn from(value: ExternMessage) -> Self {
        InternMessage { code: Some(value.preflight), msg: value.contents}
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InternMessage {
    pub code : Option<String>,
    pub msg : Option<String> 
}

impl InternMessage {
    pub fn new(code : Option<String>, msg : Option<String>) -> InternMessage {
        InternMessage { code, msg }
    }
    pub async fn send_message(&self, sender : &mut UnboundedSender<InternMessage>) {
        let to_send = self.clone();
        sender.send(to_send);
    }
}
