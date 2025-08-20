use std::error::Error;
use tonic::Status;

/*
** Tonic report the client disconnection within Error.source
** because it's a broken pipe raised by the h2 crate
** Thereis a pending issue on github to have better integrated way to detect
** client disconnection / broken pipe.
** For now i have just redo the tricks from the tonic/examples/src/streaming/server.rs
*/
pub fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;
    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }
        // h2::Error do not expose std::io::Error with `source()`
        // https://github.com/hyperium/h2/pull/462
        if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
            if let Some(io_err) = h2_err.get_io() {
                return Some(io_err);
            }
        }
        err = err.source()?;
    }
}
