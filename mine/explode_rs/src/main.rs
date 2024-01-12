use std::ffi::c_char;

pub mod proto {
    include!("../protos/out/proto.rs");
}

mod db_mine;
mod explode;
mod mine_fetcher;

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
    let log_path = std::path::PathBuf::from("explode");
    let log_level = my_logger::LogLevel::Info as u16;
    my_logger::init(&log_path, "client", log_level, false);

    //
    let mut ips = vec!["127.0.0.1"];

    for ip in &ips {
        //
        let ip = *ip;
        c_api::follow_ip(ip.as_ptr() as *const c_char, ip.len() as u64);
    }

    ips.push("localhost");

    // 登录触发
    // let in_ip = "127.0.0.1";
    // c_api::filter_ip(in_ip.as_ptr() as *const c_char, in_ip.len() as u64);

    // fetch 触发
    let path = "res/dragon_5001.xml";
    c_api::filter_config(path.as_ptr() as *const c_char, path.len() as u64);

    println!("start loop ...");
    for i in 0.. {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        c_api::safe_loop();
        println!("safe_loo {}", i);
    }
    println!("loop over.");
}
