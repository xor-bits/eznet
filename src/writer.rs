use crate::packet::{Packet, PacketType};
use futures::{
    future::{join_all, pending},
    select_biased, FutureExt, SinkExt,
};
use quinn::{Connection, SendStream};
use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};
use tokio::{
    sync::{broadcast, mpsc},
    time::sleep,
};
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

//

macro_rules! unwrap_or {
    ($e:expr, $or:expr, $err:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(err) => {
                log::debug!("Disconnected, reason: {err} ({})", $err);
                $or;
            }
        }
    };
}

//

pub async fn writer_worker_job(
    connection: Connection,
    mut recv: mpsc::Receiver<Packet>,
    mut should_stop: broadcast::Receiver<()>,
) {
    let mut ordered: HashMap<Option<u16>, FWrite> = Default::default();
    let mut can_flush = false;

    while let Some(job) = next_job(&mut recv, &mut should_stop, can_flush).await {
        match job {
            WriterJob::Feed(Packet {
                bytes,
                ty: PacketType::ReliableOrdered(stream),
            }) => {
                let stream = match ordered.entry(stream) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => entry.insert(FramedWrite::new(
                        unwrap_or!(connection.open_uni().await, break, "d"),
                        LengthDelimitedCodec::default(),
                    )),
                };
                unwrap_or!(stream.feed(bytes).await, break, "e");
                can_flush = true;
            }
            WriterJob::Feed(Packet {
                bytes,
                ty: PacketType::ReliableUnordered,
            }) => {
                let open_uni = connection.open_uni();
                tokio::spawn(async move {
                    let mut stream = FramedWrite::new(
                        unwrap_or!(open_uni.await, return, "f"),
                        LengthDelimitedCodec::default(),
                    );
                    unwrap_or!(stream.send(bytes).await, return, "g");
                    unwrap_or!(stream.get_mut().finish().await, return, "h");
                });
            }
            WriterJob::Feed(Packet {
                bytes,
                ty: PacketType::Unreliable,
            }) => {
                unwrap_or!(connection.send_datagram(bytes), break, "i");
            }
            WriterJob::Flush => flush(&mut ordered, &mut can_flush).await,
        }
    }

    flush(&mut ordered, &mut can_flush).await;

    log::debug!("Writer worker stopped");
}

async fn next_job(
    recv: &mut mpsc::Receiver<Packet>,
    should_stop: &mut broadcast::Receiver<()>,
    can_flush: bool,
) -> Option<WriterJob> {
    if let Ok(packet) = recv.try_recv() {
        return Some(WriterJob::Feed(packet));
    }

    let wait_until_flush = async {
        if can_flush {
            sleep(Duration::from_millis(1)).await;
        } else {
            pending::<()>().await;
        }
    };

    select_biased! {
        p = recv.recv().fuse() => p.map(WriterJob::Feed),
        _ = should_stop.recv().fuse() => None,
        _ = wait_until_flush.fuse() => Some(WriterJob::Flush)
    }
}

async fn flush(ordered: &mut HashMap<Option<u16>, FWrite>, can_flush: &mut bool) {
    *can_flush = false;

    // finish all pending streams

    join_all(ordered.drain().map(|(_, mut stream)| async move {
        unwrap_or!(stream.flush().await, return, "j");
        unwrap_or!(stream.get_mut().finish().await, return, "k");
    }))
    .await;
}

//

enum WriterJob {
    Feed(Packet),
    Flush,
}

type FWrite = FramedWrite<SendStream, LengthDelimitedCodec>;
