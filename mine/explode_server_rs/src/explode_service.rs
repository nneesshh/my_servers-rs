//!
//! ExplodeService
//!

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use serde_json::Value as Json;

use my_service::{http_server_listen, G_SERVICE_NET};
use my_service::{NodeState, ServiceHandle, ServiceRs, TcpConn};

///
pub const SERVICE_ID_EXLODE_SERVICE: u64 = 20001_u64;

const HTTP_PORT: u16 = 48964;

lazy_static::lazy_static! {
    pub static ref G_EXLODE_SERVICE: Arc<ExplodeService> = Arc::new(ExplodeService::new(SERVICE_ID_EXLODE_SERVICE));
}

pub struct ExplodeService {
    pub handle: ServiceHandle,
}

impl ExplodeService {
    ///
    pub fn new(id: u64) -> Self {
        Self {
            handle: ServiceHandle::new(id, NodeState::Idle),
        }
    }
}

impl ServiceRs for ExplodeService {
    /// 获取 service name
    #[inline(always)]
    fn name(&self) -> &str {
        "explode_service"
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

///
pub fn launch_http_server() {
    //
    let request_fn = |conn: Arc<TcpConn>,
                      req: http::Request<Vec<u8>>,
                      response_builder: http::response::Builder| {
        let _hd = conn.hd;

        let _uri = req.uri();
        let (_parts, body) = req.into_parts();

        let body_str = unsafe { std::str::from_utf8_unchecked(body.as_slice()) };
        println!("{}/{}", body_str.len(), body_str);
        let body_r = serde_json::from_str::<Json>(body_str);
        match body_r {
            Ok(val) => {
                //
                save_content_to_file(&val);
            }
            Err(err) => {
                //
                log::error!("json is in valid!!! error: {}!!!", err);
            }
        };

        // read mine.json
        let msg_opt = {
            let file_r = std::fs::File::open("res/mine.json");
            match file_r {
                Ok(file) => {
                    //
                    let msg_r = serde_json::from_reader(file);
                    match msg_r {
                        Ok(msg) => Some(msg),
                        Err(err) => {
                            //
                            log::error!("file should be proper JSON!!! error: {}!!!", err);
                            None
                        }
                    }
                }
                Err(err) => {
                    log::error!("file should open read only!!! error: {}!!!", err);
                    None
                }
            }
        };
        let msg = msg_opt.unwrap_or(json!({
            "msg": "ok",
            "ec": 0,
            "data": ["18.163.14.56"],
        }));

        let resp_body_vec = msg.to_string().as_bytes().to_vec();

        //
        let response = response_builder.body(resp_body_vec).unwrap();
        Ok(response)
    };

    let addr = std::format!("0.0.0.0:{}", HTTP_PORT);
    http_server_listen(addr.as_str(), request_fn, &G_SERVICE_NET);
}

fn save_content_to_file(payload: &Json) {
    //
    let zone = payload
        .get("zone")
        .map_or("".to_owned(), |zone| zone.to_string());
    let group = payload
        .get("group")
        .map_or("".to_owned(), |group| group.to_string());

    let data = payload
        .get("data")
        .map_or("".to_owned(), |data| data.to_string());

    // write to file
    let path = PathBuf::from("out/zones/");
    let mut full_path = PathBuf::from(&path);
    let full_name = std::format!("{}_{}", group, zone);
    full_path.push(full_name);

    // ensure path
    std::fs::create_dir_all(&path).unwrap();

    std::fs::write(&full_path, data.as_bytes()).unwrap();

    log::info!("path: {:?}, payload: {}", full_path, payload);
}
