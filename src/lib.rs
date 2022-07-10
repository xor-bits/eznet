//

pub mod listener;
pub mod packet;
pub mod socket;

//

mod filter;
mod reader;
mod writer;

//

pub use bytes;

//

pub static VERSION: &str = concat!(
    concat!(env!("CARGO_PKG_NAME"), "-"),
    env!("CARGO_PKG_VERSION")
);

//

#[macro_export]
macro_rules! unwrap_or {
    ($e:expr, $or:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(err) => {
                log::debug!(
                    "Disconnected, reason: {err} ({}:{}:{})",
                    file!(),
                    line!(),
                    column!()
                );
                $or;
            }
        }
    };
}
