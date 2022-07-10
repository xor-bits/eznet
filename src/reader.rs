use crate::{
    packet::{Packet, PacketHeader},
    unwrap_or,
};
use bytes::{Bytes, BytesMut};
use futures::{stream::SelectAll, StreamExt};
use quinn::{ConnectionError, Datagrams, IncomingUniStreams, RecvStream};
use std::{collections::HashMap, io::Error};
use tokio::sync::{broadcast, mpsc};
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

//

pub async fn reader_worker_job(
    mut uni_streams: IncomingUniStreams,
    mut datagrams: Datagrams,
    mut send: mpsc::Sender<Packet>,
    mut should_stop: broadcast::Receiver<()>,
) {
    let mut recv_streams = SelectAll::new();

    let mut reliable_seq: HashMap<Option<u8>, u16> = Default::default();
    let mut unreliable_seq: HashMap<Option<u8>, u16> = Default::default();

    loop {
        let new_stream = async {
            uni_streams
                .next()
                .await
                .map(|s| s.map(|s| FramedRead::new(s, LengthDelimitedCodec::default())))
        };

        let old_stream = recv_streams.next();

        let datagram_stream = datagrams.next();

        if tokio::select! {
            stream = new_stream => handle_new_stream(stream, &mut recv_streams),
            Some(bytes) = old_stream => handle_old_stream(bytes, &mut send, &mut reliable_seq, &mut unreliable_seq).await,
            bytes = datagram_stream => handle_datagram(bytes, &mut send, &mut reliable_seq, &mut unreliable_seq).await,
            _ = should_stop.recv() => true,
        } {
            break;
        };
    }

    log::debug!("Reader worker stopped");
}

// returns true if reader should stop
fn handle_new_stream(
    stream: Option<Result<FRead, ConnectionError>>,
    recv_streams: &mut SelectAll<FRead>,
) -> bool {
    let stream = stream.ok_or("Empty new stream");

    let stream = unwrap_or!(stream, {
        return true;
    });

    let stream = unwrap_or!(stream, {
        return true;
    });

    recv_streams.push(stream);
    false
}

// returns true if reader should stop
async fn handle_old_stream(
    bytes: Result<BytesMut, Error>,
    send: &mut mpsc::Sender<Packet>,
    reliable_seq: &mut HashMap<Option<u8>, u16>,
    unreliable_seq: &mut HashMap<Option<u8>, u16>,
) -> bool {
    let packet = bytes.map(|b| bincode::deserialize(&b[..]));

    let packet = unwrap_or!(packet, {
        return true;
    });

    let packet = unwrap_or!(packet, {
        return true;
    });

    if let Some(packet) = drop_sequenced(packet, reliable_seq, unreliable_seq) {
        send.send(packet).await.is_err()
    } else {
        false
    }
}

// returns true if reader should stop
async fn handle_datagram(
    bytes: Option<Result<Bytes, ConnectionError>>,
    send: &mut mpsc::Sender<Packet>,
    reliable_seq: &mut HashMap<Option<u8>, u16>,
    unreliable_seq: &mut HashMap<Option<u8>, u16>,
) -> bool {
    let packet = bytes
        .ok_or("Empty datagram")
        .map(|b| b.map(|b| bincode::deserialize(&b[..])));

    let packet = unwrap_or!(packet, {
        return true;
    });

    let packet = unwrap_or!(packet, {
        return true;
    });

    let packet = unwrap_or!(packet, {
        return true;
    });

    if let Some(packet) = drop_sequenced(packet, reliable_seq, unreliable_seq) {
        send.send(packet).await.is_err()
    } else {
        false
    }
}

fn drop_sequenced(
    packet: Packet,
    reliable_seq: &mut HashMap<Option<u8>, u16>,
    unreliable_seq: &mut HashMap<Option<u8>, u16>,
) -> Option<Packet> {
    // TODO: then_some
    if match packet.header {
        PacketHeader::ReliableSequenced { stream_id, seq_id } => {
            drop_sequenced_common(stream_id, seq_id, reliable_seq)
        }
        PacketHeader::UnreliableSequenced { stream_id, seq_id } => {
            drop_sequenced_common(stream_id, seq_id, unreliable_seq)
        }
        _ => true,
    } {
        Some(packet)
    } else {
        None
    }
}

fn drop_sequenced_common(
    stream_id: Option<u8>,
    seq_id: u16,
    seq: &mut HashMap<Option<u8>, u16>,
) -> bool {
    let recv_seq_id = seq.entry(stream_id).or_insert(0);
    let send_seq_id = seq_id;

    // convert them to a form where it is comparable
    let rsi = u16::MAX / 2 - 1;
    let ssi = ((send_seq_id as i32 - *recv_seq_id as i32).rem_euclid(u16::MAX as i32) as u16)
        .wrapping_add(u16::MAX / 2);

    if cfg!(test) {
        dbg!(&recv_seq_id);
        dbg!(&send_seq_id);
        dbg!(&rsi);
        dbg!(&ssi);
    }

    if ssi > rsi {
        // got packet that is 'newer'
        *recv_seq_id = send_seq_id + 1;
        true
    } else {
        // got packet that is 'older'
        log::debug!("Dropping out of sequence packet");
        false
    }
}

//

type FRead = FramedRead<RecvStream, LengthDelimitedCodec>;

//

#[cfg(test)]
mod tests {
    use crate::reader::drop_sequenced_common;
    use std::collections::hash_map::HashMap;

    #[test]
    fn drop_sequenced_common_test_0() {
        let mut seq = HashMap::new();
        seq.insert(None, 0);

        assert!(drop_sequenced_common(None, 1, &mut seq) == true);
        assert!(drop_sequenced_common(None, 1, &mut seq) == false);
        assert!(drop_sequenced_common(None, 1, &mut seq) == false);
        assert!(drop_sequenced_common(None, 2, &mut seq) == true);
        assert!(drop_sequenced_common(None, 2, &mut seq) == false);
        assert!(drop_sequenced_common(None, 2, &mut seq) == false);
        assert!(drop_sequenced_common(None, 200, &mut seq) == true);
        assert!(drop_sequenced_common(None, 2, &mut seq) == false);
        assert!(drop_sequenced_common(None, u16::MAX / 4, &mut seq) == true);
        assert!(drop_sequenced_common(None, u16::MAX / 2, &mut seq) == true);
        assert!(drop_sequenced_common(None, u16::MAX / 4 * 3, &mut seq) == true);
        assert!(drop_sequenced_common(None, u16::MAX - 100, &mut seq) == true);
        assert!(drop_sequenced_common(None, u16::MAX - 100, &mut seq) == false);
        assert!(drop_sequenced_common(None, u16::MAX - 99, &mut seq) == true);
        assert!(drop_sequenced_common(None, u16::MAX - 99, &mut seq) == false);
        assert!(drop_sequenced_common(None, 0, &mut seq) == true);
        assert!(drop_sequenced_common(None, 0, &mut seq) == false);
    }
}
