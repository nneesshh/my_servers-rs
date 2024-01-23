pub mod proto {
    include!("../protos/out/proto.rs");
}

//mod db_mine;
mod explode;
mod mine_fetcher;

mod safe_api;

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
        safe_api::follow_ip(*ip);
    }

    ips.push("localhost");

    // 登录触发
    // let in_ip = "127.0.0.1";
    // safe_api::filter_ip(in_ip.as_ptr() as *const c_char, in_ip.len() as u64);

    // fetch 触发
    let path = "res/dragon_5001.xml";
    safe_api::filter_config(path);

    println!("start loop ...");
    for i in 0.. {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        safe_api::safe_loop();
        println!("safe_loo {}", i);
    }
    println!("loop over.");
}
