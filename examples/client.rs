use rnet::{client::Client, connection::Connection};
use std::{ops::Range, time::Duration};
use tokio::{runtime::Runtime, time::sleep};

//

pub fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut client = Client::new("10.0.0.2:13331".parse().unwrap()).await;

        const RANGE: Range<i32> = 0..20000;

        println!("receiving unordered");
        for i in RANGE {
            let message = client.read().await;
            let message = std::str::from_utf8(&message[..]).unwrap();
            if Ok(i) != message.parse::<i32>() {
                println!("  - got out of order");
            }
        }

        sleep(Duration::from_secs(1)).await;

        println!("receiving ordered");
        for _ in RANGE {
            let message = client.read().await;
            let _ = std::str::from_utf8(&message[..]).unwrap();
        }

        sleep(Duration::from_secs(1)).await;

        println!("receiving unreliable");
        for _ in RANGE {
            let message = client.read().await;
            let _ = std::str::from_utf8(&message[..]).unwrap();
        }
    });
}
