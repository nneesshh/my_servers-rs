use app_helper::App;
use commlib::ServiceRs;

pub mod proto {
    include!("../../protos/out/proto.rs");
}

//mod simple_cluster;

mod cross_manager;

mod test_conf;
mod test_service;

mod app_startup;
mod test_manager;

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
    app.init(&test_service::G_TEST_SERVICE, |conf| {
        app_startup::launch(conf);
    });
    app.run();
}
