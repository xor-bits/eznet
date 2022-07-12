use crate::{
    filter::filter_unwanted, packet::Packet, reader::reader_worker_job, socket::ConnectError,
    writer::writer_worker_job,
};
use futures::future::join;
use quinn::{Connection, Endpoint, NewConnection};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

//

#[derive(Debug)]
pub struct SocketInner {
    pub(crate) endpoint: Endpoint,
    pub(crate) connection: Connection,

    pub(crate) channels: Option<(mpsc::Sender<Packet>, mpsc::Receiver<Packet>)>,

    pub(crate) write_worker: JoinHandle<()>,
    pub(crate) read_worker: JoinHandle<()>,
    pub(crate) should_stop: broadcast::Sender<()>,
}

//

impl SocketInner {
    pub(crate) fn close(self) {
        let Self {
            endpoint,
            connection,
            channels,
            write_worker,
            read_worker,
            should_stop,
        } = self;

        futures::executor::block_on(async move {
            let _ = should_stop.send(());
            let _ = join(write_worker, read_worker).await;
            let _ = (channels, connection, endpoint);

            log::debug!("Closing socket");

            // TODO: 3, see README.md
        });
    }

    pub(crate) async fn new(conn: NewConnection, endpoint: Endpoint) -> Result<Self, ConnectError> {
        let NewConnection {
            connection,
            mut uni_streams,
            datagrams,
            ..
        } = conn;

        filter_unwanted(&mut uni_streams, &connection).await?;

        // TODO: 4, see README.md
        let (worker_send, recv) = mpsc::channel(256);
        let (send, worker_recv) = mpsc::channel(256);

        let (should_stop, worker_should_stop_1) = broadcast::channel(1);
        let worker_should_stop_2 = worker_should_stop_1.resubscribe();

        // spawn writer worker
        let write_worker = tokio::spawn(writer_worker_job(
            connection.clone(),
            worker_recv,
            worker_should_stop_1,
        ));

        // spawn reader worker
        let read_worker = tokio::spawn(reader_worker_job(
            uni_streams,
            datagrams,
            worker_send,
            worker_should_stop_2,
        ));

        Ok(Self {
            endpoint,
            connection,

            channels: Some((send, recv)),

            write_worker,
            read_worker,
            should_stop,
        })
    }
}
