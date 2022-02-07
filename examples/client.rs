use rnet::{client::Client, packet::PacketFlags};
use std::ops::Range;
use tokio::{runtime::Runtime, time::Instant};

//

pub fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut client = Client::new("10.0.0.2:13331".parse().unwrap()).await;

        const RANGE: Range<i32> = 0..20000;

        // unordered test
        println!("receiving unordered");
        let timer = Instant::now();
        for i in RANGE {
            let message: String = client.read().await.unwrap();
            let expect = format!("a {i}");
            if message != expect {
                // println!("out of order")
            }
        }
        println!("receiving done: {:?}", timer.elapsed());

        // ordered test
        println!("receiving ordered");
        let timer = Instant::now();
        for i in RANGE {
            let message: String = client.read().await.unwrap();
            let expect = format!("b {i}");
            if message != expect {
                // println!("out of order")
            }
        }
        println!("receiving done: {:?}", timer.elapsed());

        // unreliable test
        println!("receiving unreliable");
        let timer = Instant::now();
        for i in RANGE {
            let message: String = client.read().await.unwrap();
            let expect = format!("c {i}");
            if message != expect {
                // println!("out of order")
            }
        }
        println!("receiving done: {:?}", timer.elapsed());

        // custom test
        println!("sending custom");
        for i in RANGE {
            client
                .send(i * 8, PacketFlags::PRESET_IMPORTANT)
                .await
                .unwrap();
        }

        client.wait_idle().await;
    });
}
