pub mod proto {
    include!("../protos/out/proto.rs");
}

mod db_mine;
mod explode;
mod mine_fetcher;

///
mod c_api;
pub use c_api::*;
