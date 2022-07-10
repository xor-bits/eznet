use crate::{
    packet::{Packet, PacketHeader},
    unwrap_or,
};
use bytes::{BufMut, Bytes, BytesMut};
use futures::{
    future::{join_all, pending},
    select_biased, FutureExt, SinkExt,
};
use quinn::{Connection, SendStream};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{broadcast, mpsc},
    time::sleep,
};
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

//

pub async fn writer_worker_job(
    connection: Connection,
    mut recv: mpsc::Receiver<Packet>,
    mut should_stop: broadcast::Receiver<()>,
) {
    let mut ordered: HashMap<Option<u8>, FWrite> = Default::default();
    let mut can_flush = false;

    let mut reliable_seq: HashMap<Option<u8>, u16> = Default::default();
    let mut unreliable_seq: HashMap<Option<u8>, u16> = Default::default();

    let stop = Arc::new(AtomicBool::new(false));

    // TODO: 5, see README.md

    while let Some(job) = next_job(&mut recv, &mut should_stop, can_flush, stop.clone()).await {
        match job {
            // reliable ordered packets
            WriterJob::Feed(Packet {
                bytes,
                header: PacketHeader::Ordered { stream_id },
            }) => {
                // get old/new stream asynchronously
                let stream = get_stream(&mut ordered, &connection, stream_id);

                // encode the packet
                let bytes = encode_packet(bytes, PacketHeader::Ordered { stream_id });

                // send the packet
                send_ordered(stream.await, &mut can_flush, bytes, stop.clone()).await;
            }

            // reliable sequenced packets
            WriterJob::Feed(Packet {
                bytes,
                header: PacketHeader::ReliableSequenced { stream_id, .. },
            }) => {
                // get old/new stream asynchronously
                let stream = get_stream(&mut ordered, &connection, stream_id);

                // generate seq id
                let s = reliable_seq.entry(stream_id).or_insert(0);
                let seq_id = *s;
                *s = s.wrapping_add(1);

                // encode the packet
                let bytes =
                    encode_packet(bytes, PacketHeader::ReliableSequenced { stream_id, seq_id });

                // send the packet
                send_ordered(stream.await, &mut can_flush, bytes, stop.clone()).await;
            }

            // reliable unordered packets
            WriterJob::Feed(Packet {
                bytes,
                header: PacketHeader::ReliableUnordered,
            }) => {
                // encode the packet
                let bytes = encode_packet(bytes, PacketHeader::ReliableUnordered);

                // send the packet
                send_unordered(&connection, bytes, stop.clone());
            }

            // unreliable sequenced packets
            WriterJob::Feed(Packet {
                bytes,
                header: PacketHeader::UnreliableSequenced { stream_id, .. },
            }) => {
                // generate seq id
                let s = unreliable_seq.entry(stream_id).or_insert(0);
                let seq_id = *s;
                *s = s.wrapping_add(1);

                // encode the packet
                let bytes = encode_packet(
                    bytes,
                    PacketHeader::UnreliableSequenced { stream_id, seq_id },
                );

                unwrap_or!(connection.send_datagram(bytes), {
                    break;
                });
            }

            // unreliable packets
            WriterJob::Feed(Packet {
                bytes,
                header: PacketHeader::Unreliable,
            }) => {
                // encode the packet
                let bytes = encode_packet(bytes, PacketHeader::Unreliable);

                unwrap_or!(connection.send_datagram(bytes), {
                    break;
                });
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
    stop: Arc<AtomicBool>,
) -> Option<WriterJob> {
    if stop.load(Ordering::SeqCst) {
        return None;
    }

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

async fn flush(ordered: &mut HashMap<Option<u8>, FWrite>, can_flush: &mut bool) {
    *can_flush = false;

    // finish all pending streams

    join_all(ordered.drain().map(|(_, mut stream)| async move {
        unwrap_or!(stream.flush().await, return);
        unwrap_or!(stream.get_mut().finish().await, return);
    }))
    .await;
}

fn encode_packet(bytes: Bytes, header: PacketHeader) -> Bytes {
    let mut packet = BytesMut::new().writer();
    bincode::serialize_into(&mut packet, &Packet { header, bytes }).unwrap();
    packet.into_inner().into()
}

async fn get_stream<'a>(
    streams: &'a mut HashMap<Option<u8>, FWrite>,
    connection: &'a Connection,
    stream: Option<u8>,
) -> Option<&'a mut FWrite> {
    match streams.entry(stream) {
        Entry::Occupied(entry) => Some(entry.into_mut()),
        Entry::Vacant(entry) => Some(entry.insert(FramedWrite::new(
            unwrap_or!(connection.open_uni().await, return None),
            LengthDelimitedCodec::default(),
        ))),
    }
}

async fn send_ordered(
    stream: Option<&mut FWrite>,
    can_flush: &mut bool,
    bytes: Bytes,
    stop: Arc<AtomicBool>,
) {
    // get the stream
    let stream = unwrap_or!(stream.ok_or_else(|| "Missing stream".to_owned()), {
        stop.store(true, Ordering::SeqCst);
        return;
    });

    // feed to it
    unwrap_or!(stream.feed(bytes).await, {
        stop.store(true, Ordering::SeqCst);
        return;
    });

    *can_flush = true;
}

fn send_unordered(connection: &Connection, bytes: Bytes, stop: Arc<AtomicBool>) {
    let open_uni = connection.open_uni();
    tokio::spawn(async move {
        // get a new stream
        let stream = unwrap_or!(open_uni.await, {
            stop.store(true, Ordering::SeqCst);
            return;
        });
        let mut stream = FramedWrite::new(stream, LengthDelimitedCodec::default());

        // send with it
        unwrap_or!(stream.send(bytes).await, {
            stop.store(true, Ordering::SeqCst);
            return;
        });

        // flush it
        unwrap_or!(stream.get_mut().finish().await, {
            stop.store(true, Ordering::SeqCst);
        });
    });
}

//

enum WriterJob {
    Feed(Packet),
    Flush,
}

type FWrite = FramedWrite<SendStream, LengthDelimitedCodec>;
