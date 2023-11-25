use std::{time::Duration, sync::{Mutex, Arc}};

use tokio::sync::watch::Sender;

use crate::{data::Batch, Data};

pub async fn run_match( sender : &Sender<usize>, data : Arc<Mutex<Data>>) {
    let size;
    {
        let d = data.lock().unwrap();
        size = d.batch_size;
    }
    for i in 0..size { // Should NOT be hardcoded
        {
            let mut d = data.lock().unwrap();
            println!("Updating displays");
            d.display1 += d.batch.play_action[0][i];
            d.display2 += d.batch.play_action[1][i];
        }
        sender.send(2);
        tokio::time::sleep(Duration::new(3, 0)).await;
    }
}
