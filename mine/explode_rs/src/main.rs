
pub mod proto {
    include!("../protos/out/proto.rs");
}

mod explode;
mod c_api;

fn main() {
    // panic hook
    std::panic::set_hook(Box::new(|panic_info| {
        println!(
            "panic info: {:?}, {:?}, panic occurred in {:?}",
            panic_info.payload().downcast_ref::<&str>(),
            panic_info.to_string(),
            panic_info.location()
        );
        log::error!(
            "panic info: {:?}, {:?}, panic occurred in {:?}",
            panic_info.payload().downcast_ref::<&str>(),
            panic_info.to_string(),
            panic_info.location()
        );
    }));

    //
    let ips = vec!["127.0.0.1"];

    for ip in &ips {
        //
        let ip = *ip;
        c_api::follow_ip(ip.as_ptr(), ip.len() as u64);
    }
}
