pub use bytes;
use futures::Future;

//

pub mod listener;
pub mod packet;
pub mod socket;

//

mod filter;
mod reader;
mod writer;

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

//

pub(crate) async fn attempt_all_async<I, A, B, F: FnMut(I::Item) -> Fut, Fut>(
    iter: I,
    mut f: F,
    empty: B,
) -> Result<A, B>
where
    I: Iterator,
    Fut: Future<Output = Result<A, B>>,
{
    let mut last_err = empty;
    for item in iter {
        match f(item).await {
            Ok(socket) => return Ok(socket),
            Err(err) => last_err = err,
        }
    }
    Err(last_err)
}

pub(crate) fn attempt_all<I, A, B, F: FnMut(I::Item) -> Result<A, B>>(
    iter: I,
    mut f: F,
    empty: B,
) -> Result<A, B>
where
    I: Iterator,
{
    let mut last_err = empty;
    for item in iter {
        match f(item) {
            Ok(socket) => return Ok(socket),
            Err(err) => last_err = err,
        }
    }
    Err(last_err)
}
