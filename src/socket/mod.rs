use self::writer::Writer;
use crate::filter::FilterError;
use quinn::{Endpoint, NewConnection};
use thiserror::Error;

//

//mod reader;
mod writer;

//

pub struct Socket {
    writer: Writer,
    //reader: Reader,
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("connect error ({0})")]
    Connect(#[from] quinn::ConnectError),

    #[error("connection error ({0})")]
    Connection(#[from] quinn::ConnectionError),

    #[error("failed to bind socket ({0})")]
    IoError(#[from] std::io::Error),

    #[error("invalid socket address ({0})")]
    InvalidSocketAddress(std::io::Error),

    #[error("no socket address")]
    NoSocketAddress,

    #[error("peer filtered out ({0})")]
    FilterError(#[from] FilterError),
}

//

impl Socket {
    pub(crate) fn new(
        NewConnection {
            connection,
            uni_streams,
            bi_streams,
            datagrams,
            ..
        }: NewConnection,
        endpoint: Endpoint,
    ) -> Self {

        Self {
            writer: Writer::new(endpoint, connection),
        }
    }
}
