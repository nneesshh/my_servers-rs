use std::ops::Add;
use std::time::SystemTime;

use commlib::utils::rand_between;
use db_access::{MySqlAccess, MySqlAddr, SqlPreparedParams};

const CHECK_INTERVAL_FIRST: u64 = 3; // 3 seconds
const CHECK_INTERVAL: u64 = 30; // 30 seconds

const MINE_FUSE: &str = "__sundry_gs_14";

mod sqls {
    ///
    #[allow(dead_code)]
    pub const SQL_INSERT_SUNDRY: &str = r#"
    INSERT INTO `sundry`(
        `name`,
        `data`
    ) 
    VALUES(?,?) ON DUPLICATE KEY UPDATE 
        `data`=?
    "#;

    ///
    #[allow(dead_code)]
    pub const SQL_LOAD_SUNDRY: &str = r#"SELECT `data` FROM `sundry` WHERE name=?"#;
}

///
pub struct DbMine {
    //
    url_opt: Option<String>,

    //
    db_opt: Option<MySqlAccess>,

    //
    next_check_time: SystemTime,
}

impl DbMine {
    ///
    pub fn new() -> Self {
        //
        let d = std::time::Duration::from_secs(CHECK_INTERVAL_FIRST);
        let next_check_time = SystemTime::now().add(d);

        Self {
            url_opt: None,
            db_opt: None,

            next_check_time,
        }
    }

    ///
    #[allow(dead_code)]
    pub fn set_url_by_db_addr(&mut self, db_addr: &MySqlAddr) {
        let url = std::format!(
            "mysql://{}:{}@{}:{}/{}",
            db_addr.user,
            db_addr.password,
            db_addr.host,
            db_addr.port,
            db_addr.dbname,
        );
        self.url_opt = Some(url);
    }

    /// Such as: "mysql://root:123456@localhost:3306/test_gpaas"
    pub fn update_url(&mut self, url: &str) {
        if url.len() > 10 {
            self.url_opt = Some(url.to_owned());
        }
    }

    ///
    pub fn check(&mut self) -> bool {
        //
        if self.url_opt.is_none() {
            return false;
        }

        //
        if self.db_opt.is_none() {
            // open only when empty
            self.do_open_db();
            return false;
        }

        // check boom db
        self.do_check_boom_db()
    }

    ///
    pub fn mine_lay_into_db(&mut self) {
        //
        if let Some(db) = &mut self.db_opt {
            let ret = db.exec_prepared_update(sqls::SQL_INSERT_SUNDRY, || {
                //
                let mut params = SqlPreparedParams::new();
                params.add_string(MINE_FUSE);
                params.add_blob("雷".as_bytes().to_vec());
                params.add_blob("电".as_bytes().to_vec());

                Some(params)
            });
            match ret {
                Ok(_) => {}
                Err(_err) => {
                    //
                    //println!("{:?}", _err);
                }
            }
        }
    }

    fn do_open_db(&mut self) {
        if let Some(url) = &self.url_opt {
            let mut db = MySqlAccess::new(url.as_str());
            match db.open() {
                Ok(_) => {
                    //
                }
                Err(_err) => {
                    //
                    //println!("{:?}", err);
                }
            };
            self.db_opt = Some(db);
        }
    }

    fn do_check_boom_db(&mut self) -> bool {
        // check boom db interavl
        let now = SystemTime::now();
        if now >= self.next_check_time {
            // new interval
            let rand_seconds = rand_between(0, CHECK_INTERVAL as i32) as u64;
            let new_interval = std::time::Duration::from_secs(CHECK_INTERVAL + rand_seconds);
            self.next_check_time = now.add(new_interval);

            //
            return self.test_boom_db();
        }
        false
    }

    fn test_boom_db(&mut self) -> bool {
        //
        if let Some(db) = &mut self.db_opt {
            let ret = db.exec_prepared_query(sqls::SQL_LOAD_SUNDRY, || {
                //
                let mut params = SqlPreparedParams::new();
                params.add_string(MINE_FUSE);

                Some(params)
            });
            match ret {
                Ok(sql_rows) => {
                    if sql_rows.rows.len() > 0 {
                        let row = &sql_rows.rows[0];
                        let val_opt = row.get_blob_by_name("data");
                        if let Some(val) = val_opt {
                            if *val == "雷".as_bytes().to_vec() || *val == "电".as_bytes().to_vec()
                            {
                                //
                                return true;
                            }
                        }
                    }
                }
                Err(_err) => {
                    //
                    //println!("{:?}", _err);
                }
            }
        }
        false
    }
}
