use rnet::{connection::Connection, packet::PacketFlags, server::Server};
use std::{ops::Range, time::Duration};
use tokio::{runtime::Runtime, time::sleep};

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
                .send(format!("{i}").into(), PacketFlags::IMPORTANT_UNORDERED)
                .await;
        }

        sleep(Duration::from_secs(1)).await;

        // ordered test
        println!("sending ordered");
        for i in RANGE {
            handler
                .send(format!("{i}").into(), PacketFlags::IMPORTANT_ORDERED)
                .await;
        }

        // unreliable test
        println!("sending unreliable");
        for i in RANGE {
            handler
                .send(format!("{i}").into(), PacketFlags::OPTIONAL_UNORDERED)
                .await;
        }

        server.wait_idle().await;
    });
}
