//

pub mod listener;
pub mod packet;
pub mod socket;

//

mod reader;
mod writer;

//

pub use bytes;

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
