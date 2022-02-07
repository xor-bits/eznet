use super::{Framed, FramedRead, FramedWrite};
use crate::packet::Packet;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use quinn::{Connection, ConnectionError, IncomingUniStreams};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::{
    join, spawn,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use tokio_serde::formats::Bincode;
use tokio_util::codec::LengthDelimitedCodec;

//

pub struct Unordered {
    send: UnboundedSender<Packet>,
    recv: UnboundedReceiver<Packet>,
}

//

impl Unordered {
    pub async fn new(connection: Arc<Connection>, incoming: IncomingUniStreams) -> Self {
        let (send, t_recv) = unbounded_channel();
        let (t_send, recv) = unbounded_channel();

        spawn(async move {
            if let Err(err) = Self::writer_loop(connection, t_recv).await {
                log::debug!("Error: {err}")
            }
        });
        spawn(async move {
            if let Err(err) = Self::reader_loop(incoming, t_send).await {
                log::debug!("Error: {err}")
            }
        });

        Self { send, recv }
    }

    async fn writer_loop(
        connection: Arc<Connection>,
        mut recv: UnboundedReceiver<Packet>,
    ) -> Result<(), ConnectionError> {
        let mut seq = 0;

        // TODO: feed packets and send them every 2ms

        loop {
            let stream = connection.open_uni();
            let recv = recv.recv();

            let (stream, mut packet) = match join! { stream, recv } {
                (Ok(stream), Some(packet)) => (stream, packet),
                (Err(err), _) => {
                    break Err(err);
                }
                (_, None) => {
                    break Ok(());
                }
            };
            if let Some(s) = packet.header.seq.as_mut() {
                *s = seq;
                seq = seq.wrapping_add(1);
            }

            let stream = FramedWrite::new(stream, LengthDelimitedCodec::default());
            let mut stream = Framed::new(stream, Bincode::default());

            match stream.send(packet).await {
                Ok(_) => {}
                Err(_) => break Ok(()),
            };
        }
    }

    async fn reader_loop(
        mut incoming: IncomingUniStreams,
        send: UnboundedSender<Packet>,
    ) -> Result<(), ConnectionError> {
        let keep_running = Arc::new(AtomicBool::new(true));

        loop {
            if !keep_running.load(Ordering::SeqCst) {
                break Ok(());
            }
            let stream = match incoming.next().await {
                Some(Ok(stream)) => stream,
                Some(Err(err)) => break Err(err),
                None => break Ok(()),
            };

            // let keep_running = keep_running.clone();
            // let send = send.clone();
            // spawn(async move {
            let stream = FramedRead::new(stream, LengthDelimitedCodec::default());
            let mut stream = Framed::new(stream, Bincode::default());

            let packet = match stream.next().await {
                Some(Ok(packet)) => packet,
                _ => {
                    keep_running.store(false, Ordering::SeqCst);
                    continue;
                }
            };

            match send.send(packet) {
                Ok(_) => {}
                Err(_) => {
                    keep_running.store(false, Ordering::SeqCst);
                }
            }
            // });
        }
    }

    pub async fn write(&mut self, message: Bytes, seq: bool) -> Option<()> {
        self.send
            .send(Packet::new(if seq { Some(0) } else { None }, message))
            .ok()?;
        Some(())
    }

    pub async fn read(&mut self) -> Option<Bytes> {
        let packet = self.recv.recv().await?;
        Some(packet.payload)
    }
}
