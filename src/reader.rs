use crate::{packet::Packet, unwrap_or};
use bytes::{Bytes, BytesMut};
use futures::{stream::SelectAll, StreamExt};
use quinn::{ConnectionError, Datagrams, IncomingUniStreams, RecvStream};
use std::io::Error;
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
            Some(bytes) = old_stream => handle_old_stream(bytes, &mut send).await,
            bytes = datagram_stream => handle_datagram(bytes, &mut send).await,
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
) -> bool {
    let packet = bytes.map(|b| bincode::deserialize(&b[..]));

    let packet = unwrap_or!(packet, {
        return true;
    });

    let packet = unwrap_or!(packet, {
        return true;
    });

    // TODO: 6, see README.md

    send.send(packet).await.is_err()
}

// returns true if reader should stop
async fn handle_datagram(
    bytes: Option<Result<Bytes, ConnectionError>>,
    send: &mut mpsc::Sender<Packet>,
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

    // TODO: 6, see README.md

    send.send(packet).await.is_err()
}

//

type FRead = FramedRead<RecvStream, LengthDelimitedCodec>;
