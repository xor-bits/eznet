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
    mut send: mpsc::Sender<Bytes>,
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
    match stream {
        Some(Ok(stream)) => {
            recv_streams.push(stream);
            false
        }
        Some(Err(err)) => {
            log::debug!("Disconnecting, reason: {err} (a)");
            true
        }
        _ => {
            log::debug!("Empty new stream");
            true
        }
    }
}

// returns true if reader should stop
async fn handle_old_stream(bytes: Result<BytesMut, Error>, send: &mut mpsc::Sender<Bytes>) -> bool {
    match bytes {
        Ok(bytes) => send.send(bytes.into()).await.is_err(),
        Err(err) => {
            log::debug!("Disconnecting, reason: {err} (b)");
            true
        } /* _ => {
              log::debug!("Empty old stream");
              true
          } */
    }
}

// returns true if reader should stop
async fn handle_datagram(
    bytes: Option<Result<Bytes, ConnectionError>>,
    send: &mut mpsc::Sender<Bytes>,
) -> bool {
    match bytes {
        Some(Ok(bytes)) => send.send(bytes).await.is_err(),
        Some(Err(err)) => {
            log::debug!("Disconnecting, reason: {err} (c)");
            true
        }
        _ => {
            log::debug!("Empty datagram");
            true
        }
    }
}

//

type FRead = FramedRead<RecvStream, LengthDelimitedCodec>;
