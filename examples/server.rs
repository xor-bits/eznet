use rnet::{packet::PacketFlags, server::Server};
use std::ops::Range;
use tokio::{runtime::Runtime, time::Instant};

//

pub fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut server = Server::new("0.0.0.0:13331".parse().unwrap());
        let mut handler = server.accept().await;

        const RANGE: Range<i32> = 0..20000;

        // unordered test
        println!("sending unordered");
        for i in RANGE {
            handler
                .send(format!("a {i}"), PacketFlags::PRESET_IMPORTANT)
                .await
                .unwrap();
        }

        // ordered test
        println!("sending ordered");
        for i in RANGE {
            handler
                .send(format!("b {i}"), PacketFlags::PRESET_ASSERTIVE)
                .await
                .unwrap();
        }

        // unreliable test
        println!("sending unreliable");
        for i in RANGE {
            handler
                .send(format!("c {i}"), PacketFlags::PRESET_DEFAULT)
                .await
                .unwrap();
        }

        // custom test
        println!("receiving custom");
        let timer = Instant::now();
        for i in RANGE {
            let message: i32 = handler.read().await.unwrap();
            let expect = i * 8;
            if message != expect {
                println!("out of order")
            }
        }
        println!("receiving done: {:?}", timer.elapsed());

        server.wait_idle().await;
    });
}
