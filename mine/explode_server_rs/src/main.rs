use app_helper::App;

pub mod proto {
    include!("../protos/out/proto.rs");
}

mod explode_service;

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
    let log_path = std::path::PathBuf::from("mine");
    let log_level = my_logger::LogLevel::Info as u16;
    my_logger::init(&log_path, "server", log_level, false);

    //
    let mut app = App::new_raw("explode");
    app.init_raw(&explode_service::G_EXLODE_SERVICE, || {
        explode_service::launch_http_server();
    });
    app.run();
}
