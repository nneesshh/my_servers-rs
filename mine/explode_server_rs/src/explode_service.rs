//!
//! ExplodeService
//!

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;



use commlib::{http_server_listen, G_SERVICE_NET, XmlReader, ZoneId};
use commlib::{NodeState, ServiceHandle, ServiceRs, TcpConn};
use db_access::MySqlAddr;

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

        let content_r = serde_json::from_slice(body.as_slice());

        //
        match content_r {
            Ok(content) => {
                //
                save_content_to_file(content);
            }
            Err(err) => {
                //
                log::error!("json is in valid!!! error: {}!!!", err);
            }
        };

        let msg = json!({
            "msg": "ok",
            "ec": 0,
            "data": ["18.163.14.56"],
        });
        let resp_body_vec = msg.to_string().as_bytes().to_vec();

        //
        let response = response_builder.body(resp_body_vec).unwrap();
        Ok(response)
    };

    let addr = std::format!("0.0.0.0:{}", HTTP_PORT);
    http_server_listen(addr.as_str(), request_fn, true, &G_SERVICE_NET);
}

fn save_content_to_file(content:&str) {
    let xml_r = XmlReader::read_content(content);
    match xml_r {
        Ok(xml) => {
            //
            let group_id = xml.get::<ZoneId>(vec!["group"], 0);
            let zone_id = xml.get::<ZoneId>(vec!["zone"], 0);

            let user = xml.get(vec!["node", "game", "user"], "root".to_owned());
            let password = xml.get(vec!["node", "game", "pwd"], "".to_owned());
            let host = xml.get(vec!["node", "game", "addr"], "127.0.0.1".to_owned());
            let port = xml.get::<u64>(vec!["node", "game", "port"], 3306) as u16;
            let dbname = xml.get(vec!["node", "game", "db"], "".to_owned());

            let _db_addr = MySqlAddr {
                user,
                password,
                host,
                port,
                dbname,
            };

            //
            
             // write to file
             let path = PathBuf::from("out/zones/");
             let mut full_path = PathBuf::from(&path);
             let full_name = std::format!("{}_{}_xml", group_id, zone_id);
             full_path.push(full_name);

             // ensure path
            std::fs::create_dir_all(&path).unwrap();

            std::fs::write(&full_path, content.as_bytes()).unwrap();
        }
        Err(err) => {
            log::error!("save_content_to_file failed!!! err: {}!!!", err);
        }
    }
}