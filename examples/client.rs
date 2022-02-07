use rnet::client::Client;
use std::{ops::Range, time::Duration};
use tokio::{
    runtime::Runtime,
    time::{sleep, Instant},
};

//

pub fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut client = Client::new("10.0.0.2:13331".parse().unwrap()).await;

        const RANGE: Range<i32> = 0..20000;

        println!("receiving unordered");
        let timer = Instant::now();
        for i in RANGE {
            let message = client.read().await.unwrap();
            let message = std::str::from_utf8(&message[..]).unwrap();
            let expect = format!("a {i}");
            if message != expect {
                println!("out of order")
            }
        }
        println!("receiving done: {:?}", timer.elapsed());

        sleep(Duration::from_secs(1)).await;

        println!("receiving ordered");
        let timer = Instant::now();
        for i in RANGE {
            let message = client.read().await.unwrap();
            let message = std::str::from_utf8(&message[..]).unwrap();
            let expect = format!("b {i}");
            if message != expect {
                println!("out of order")
            }
        }
        println!("receiving done: {:?}", timer.elapsed());

        sleep(Duration::from_secs(1)).await;

        println!("receiving unreliable");
        let timer = Instant::now();
        for i in RANGE {
            let message = client.read().await.unwrap();
            let message = std::str::from_utf8(&message[..]).unwrap();
            let expect = format!("c {i}");
            if message != expect {
                println!("out of order")
            }
        }
        println!("receiving done: {:?}", timer.elapsed());
    });
}
