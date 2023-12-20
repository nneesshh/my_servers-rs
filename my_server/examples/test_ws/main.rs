use app_helper::App;

pub mod proto {
    include!("../../protos/out/proto.rs");
}

mod simple_ws_service;

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
    let arg_vec: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let mut app = App::new(&arg_vec, "test");
    app.init(&simple_ws_service::G_SIMPLE_WS_SERVICE, |conf| {
        simple_ws_service::test_ws_server(conf);
    });
    app.run();
}
