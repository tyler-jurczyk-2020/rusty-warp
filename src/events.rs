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
        // Bits on indicator in format Player2 | Player1
        let mut indicator = 0b00;
        {
            let mut d = data.lock().unwrap();
            println!("Updating displays");
            d.display1 += d.batch.play_action[0][i];
            d.display2 += d.batch.play_action[1][i];
            indicator |= (d.batch.play_action[0][i] > 0.0) as usize;
            indicator |= ((d.batch.play_action[1][i] > 0.0) as usize)*2;
        }
        sender.send(indicator);
        tokio::time::sleep(Duration::new(3, 0)).await;
    }
}
