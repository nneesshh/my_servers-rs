use std::ffi::c_char;


pub mod proto {
    include!("../protos/out/proto.rs");
}

mod db_mine;
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
    let mut ips = vec!["127.0.0.1"];

    for ip in &ips {
        //
        let ip = *ip;
        c_api::follow_ip(ip.as_ptr() as *const c_char, ip.len() as u64);
    }

    ips.push("localhost");

    //
    let in_ip = "127.0.0.1";
    c_api::filter_ip(in_ip.as_ptr() as *const c_char, in_ip.len() as u64);

    println!("start loop ...");
    let url = "mysql://root:123456@localhost:3306/test_gpaas";
    for _i in 0.. {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        c_api::safe_loop();
    }
    println!("loop over.");
}
