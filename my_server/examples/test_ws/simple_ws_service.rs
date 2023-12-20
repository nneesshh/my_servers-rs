//!
//! SimpleService
//!

#[cfg(unix)]
use std::io::Write;
use std::sync::Arc;

use commlib::{http_server_listen, ws_server_listen, ConnId, G_SERVICE_NET, G_SERVICE_SIGNAL};
use commlib::{Clock, NodeState, ServiceHandle, ServiceRs, TcpConn};

use app_helper::conf::Conf;
use app_helper::G_CONF;
use commlib::utils::gen_password;
use net_packet::NetPacketGuard;

///
pub const SERVICE_ID_SIMPLE_WS_SERVICE: u64 = 20002_u64;
lazy_static::lazy_static! {
    pub static ref G_SIMPLE_WS_SERVICE: Arc<SimpleWsService> = Arc::new(SimpleWsService::new(SERVICE_ID_SIMPLE_WS_SERVICE));
}

pub struct SimpleWsService {
    pub handle: ServiceHandle,
}

impl SimpleWsService {
    ///
    pub fn new(id: u64) -> Self {
        Self {
            handle: ServiceHandle::new(id, NodeState::Idle),
        }
    }
}

impl ServiceRs for SimpleWsService {
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

///
pub fn test_ws_server(srv_conf: &Arc<Conf>) {
    let srv = &G_SIMPLE_WS_SERVICE;

    // pre-startup, main manager init
    let g_conf = G_CONF.load();
    let connection_limit: usize = (g_conf.limit_players as f32 * 1.1_f32) as usize; // 0=no limit
    log::info!("test_ws_server: connection_limit={}", connection_limit);

    let conn_fn = |conn: Arc<TcpConn>| {
        let hd = conn.hd;
        log::info!("[hd={}] conn_fn", hd);
    };

    let pkt_fn = |conn: Arc<TcpConn>, pkt: NetPacketGuard| {
        let hd = conn.hd;
        log::info!("[hd={}] msg_fn", hd);
    };

    let close_fn = |hd: ConnId| {
        log::info!("[hd={}] close_fn", hd);
    };

    let addr = std::format!("0.0.0.0:8090");
    ws_server_listen(
        srv,
        addr.as_str(),
        conn_fn,
        pkt_fn,
        close_fn,
        connection_limit,
        &G_SERVICE_NET,
    );

    let srv = G_SIMPLE_WS_SERVICE.as_ref();

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
