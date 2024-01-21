//!
//! SimpleService
//!

#[cfg(unix)]
use std::io::Write;
use std::sync::Arc;

use commlib::Clock;

use app_helper::conf::Conf;
use app_helper::G_CONF;
use srv_helper::{http_server_listen, G_SERVICE_NET, G_SERVICE_SIGNAL};
use srv_helper::{NodeState, ServiceHandle, ServiceRs, TcpConn};

///
pub const SERVICE_ID_SIMPLE_SERVICE: u64 = 20001_u64;
lazy_static::lazy_static! {
    pub static ref G_SIMPLE_SERVICE: Arc<SimpleService> = Arc::new(SimpleService::new(SERVICE_ID_SIMPLE_SERVICE));
}

pub struct SimpleService {
    pub handle: ServiceHandle,
}

impl SimpleService {
    ///
    pub fn new(id: u64) -> Self {
        Self {
            handle: ServiceHandle::new(id, NodeState::Idle),
        }
    }
}

impl ServiceRs for SimpleService {
    /// 获取 service name
    #[inline(always)]
    fn name(&self) -> &str {
        "simple_service"
    }

    /// 获取 service 句柄
    #[inline(always)]
    fn get_handle(&self) -> &ServiceHandle {
        &self.handle
    }

    /// 配置 service
    fn conf(&self) {}

    /// update
    #[inline(always)]
    fn update(&self) {}

    /// 在 service 线程中执行回调任务
    #[inline(always)]
    fn run_in_service(&self, cb: Box<dyn FnOnce() + Send + 'static>) {
        self.get_handle().run_in_service(cb);
    }

    /// 当前代码是否运行于 service 线程中
    #[inline(always)]
    fn is_in_service_thread(&self) -> bool {
        self.get_handle().is_in_service_thread()
    }

    /// 等待线程结束
    fn join(&self) {
        self.get_handle().join_service();
    }
}

use serde_json::json;

use srv_helper::G_SERVICE_HTTP_CLIENT;

pub fn test_http_client(srv: &Arc<SimpleService>) {
    let body =
        json!({"foo": false, "bar": null, "answer": 42, "list": [null, "world", true]}).to_string();

    //
    let srv2 = srv.clone();
    G_SERVICE_HTTP_CLIENT.http_post(
        "http://127.0.0.1:7878",
        vec!["Content-Type: application/json".to_owned()],
        body,
        move |code, resp| {
            //
            srv2.run_in_service(Box::new(move || {
                log::info!("hello http code: {}, resp: {}", code, resp);
            }));
        },
    )
}

///
pub fn test_http_server(_conf: &Arc<Conf>) {
    // pre-startup, main manager init
    let g_conf = G_CONF.load();

    let request_fn = |conn: Arc<TcpConn>,
                      _req: http::Request<Vec<u8>>,
                      response_builder: http::response::Builder| {
        let _hd = conn.hd;
        //log::info!("[hd={}] request_fn", hd);

        /*
        let rand_pass = commlib::util::gen_password(10);
        let msg = std::format!("hello simple service, rand_pass={}", rand_pass);
        */
        //let msg = "hello simple servic";
        let msg = json!({
            "msg": "ok",
            "ec": 0,
            "data": {
                "configs": [
                    {
                        "groups": "1001,1009|1016,1021|1024,1025,1026|1027,1028,1029,1030,1031|1032,1033,1034|1035,1036,1037,1038,1039",
                        "node": 999990001,
                        "zone": 500,
                        "id": 1
                    },
                    {
                        "groups": "1001,1009|1016,1021,1024|1025,1026,1027|1028,1029,1030|1031,1032,1033|1034,1035,1036|1037,1038,1039",
                        "node": 999990001,
                        "zone": 500,
                        "id": 2
                    },
                    {
                        "groups": "1001,1009|1016,1021|1024,1025,1026|1027,1028,1029,1030,1031|1032,1033,1034|1035,1036,1037,1038,1039",
                        "node": 999990001,
                        "zone": 500,
                        "id": 3
                    },
                    {
                        "groups": "1001,1009|1016,1021|1024,1025,1026|1027,1028,1029,1030,1031|1032,1033,1034|1035,1036,1037,1038,1039",
                        "node": 999990001,
                        "zone": 500,
                        "id": 4
                    },
                    {
                        "groups": "1001,1009,1016,1021|1024,1025,1026,1027,1028,1029,1030,1031|1032,1033,1034|1035,1036,1037,1038,1039",
                        "node": 999990001,
                        "zone": 500,
                        "id": 5
                    },
                    {
                        "groups": "1001,1009|1016,1021|1024,1025,1026|1027,1028,1029,1030,1031|1032,1033,1034|1035,1036,1037,1038,1039",
                        "node": 999990001,
                        "zone": 500,
                        "id": 7
                    },
                    {
                        "groups": "1001,1009|1016,1021|1024,1025,1026|1027,1028,1029,1030,1031|1032,1033,1034|1035,1036,1037,1038,1039",
                        "node": 999990001,
                        "zone": 500,
                        "id": 8
                    },
                    {
                        "groups": "1001,1009|1016,1021|1024,1025,1026|1027,1028,1029,1030,1031|1032,1033,1034|1035,1036,1037,1038,1039",
                        "node": 999990001,
                        "zone": 500,
                        "id": 9
                    }
                ]
            }
        });
        let resp_body_vec = msg.to_string().as_bytes().to_vec();

        //
        let response = response_builder.body(resp_body_vec).unwrap();
        Ok(response)
    };

    let addr = std::format!("0.0.0.0:{}", g_conf.http_port);
    http_server_listen(addr.as_str(), request_fn, &G_SERVICE_NET);

    /*#[cfg(unix)]
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(100)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();*/

    #[cfg(unix)]
    let guard = pprof::ProfilerGuard::new(100).unwrap();

    let srv = G_SIMPLE_SERVICE.as_ref();

    /*Clock::set_timeout(srv, 10 * 1000, move || {
        //
        log::info!("simple service timeout.");

        //
        #[cfg(unix)]
        if let Ok(report) = guard.report().build() {
            let file = std::fs::File::create("flamegraph.svg").unwrap();
            let mut options = pprof::flamegraph::Options::default();
            options.image_width = Some(2500);
            report.flamegraph_with_options(file, &mut options).unwrap();
        }

        std::process::exit(0);
    });*/

    G_SERVICE_SIGNAL.listen_sig_int(srv, move || {
        //
        log::info!("simple service signal raised.");

        //
        #[cfg(unix)]
        if let Ok(report) = guard.report().build() {
            let file = std::fs::File::create("flamegraph.svg").unwrap();
            let mut options = pprof::flamegraph::Options::default();
            options.image_width = Some(2500);
            report.flamegraph_with_options(file, &mut options).unwrap();
            println!("report: {:?}", &report);
        }

        std::process::exit(0);
    })
}
