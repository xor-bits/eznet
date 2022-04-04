use crate::packet::Packet;
use anyhow::{format_err, Result};
use bytes::Bytes;
use futures::StreamExt;
use quinn::{Connection, Datagrams, SendDatagramError};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::{
    spawn,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

//

pub struct Unreliable<S, R>
where
    S: Send + Serialize + Unpin + 'static,
    R: Send + DeserializeOwned + Unpin + 'static,
{
    send: UnboundedSender<Packet<S>>,
    recv: UnboundedReceiver<Packet<R>>,
}

//

impl<S, R> Unreliable<S, R>
where
    S: Send + Serialize + Unpin + 'static,
    R: Send + DeserializeOwned + Unpin + 'static,
{
    pub async fn new(connection: Arc<Connection>, datagrams: Datagrams) -> Self {
        let (send, t_recv) = unbounded_channel();
        let (t_send, recv) = unbounded_channel();

        spawn(async move {
            if let Err(err) = Self::writer_loop(connection, t_recv).await {
                log::debug!("Error: {err}")
            }
        });
        spawn(async move {
            if let Err(err) = Self::reader_loop(datagrams, t_send).await {
                log::debug!("Error: {err}")
            }
        });

        Self { send, recv }
    }

    async fn writer_loop(
        connection: Arc<Connection>,
        mut recv: UnboundedReceiver<Packet<S>>,
    ) -> Result<()> {
        let config = bincode::config::standard();
        let mut seq = 0;

        // TODO: feed packets and send them every 1ms

        loop {
            // TODO: let..else
            let mut packet = recv.recv().await.ok_or_else(|| format_err!("Stopped"))?;
            if let Some(s) = packet.header.seq.as_mut() {
                *s = seq;
                seq = seq.wrapping_add(1);
            }

            let bytes = bincode::serde::encode_to_vec(&packet, config)?;

            match connection.send_datagram(Bytes::from(bytes)) {
                Ok(_) => {}
                Err(SendDatagramError::TooLarge) => {
                    log::debug!("Datagram was too large, dropping.");
                    continue;
                }
                Err(err) => break Err(err.into()),
            }
        }
    }

    async fn reader_loop(mut datagrams: Datagrams, send: UnboundedSender<Packet<R>>) -> Result<()> {
        let config = bincode::config::standard();
        loop {
            let bytes = datagrams
                .next()
                .await
                .ok_or_else(|| format_err!("Stopped"))??;

            let (packet, _) = bincode::serde::decode_from_slice(&bytes, config)?;

            send.send(packet).map_err(|_| format_err!("Stopped"))?;
        }
    }

    pub async fn write(&mut self, message: S, seq: bool) -> Option<()> {
        self.send
            .send(Packet::new(if seq { Some(0) } else { None }, message))
            .ok()?;
        Some(())
    }

    pub async fn read(&mut self) -> Option<R> {
        let packet = self.recv.recv().await?;
        Some(packet.payload)
    }
}
