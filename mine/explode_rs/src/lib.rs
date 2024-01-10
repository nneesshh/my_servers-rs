

pub mod proto {
    include!("../protos/out/proto.rs");
}

mod explode;

///
mod c_api;
pub use c_api::{filter_ip, follow_ip};