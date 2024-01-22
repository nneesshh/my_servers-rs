pub mod proto {
    include!("../protos/out/proto.rs");
}

include!("../ffi/filter.rs");

//mod db_mine;
mod explode;
mod mine_fetcher;

///
mod safe_api;
pub use safe_api::*;
