use std::ffi::c_char;

use commlib::with_tls_mut;

use crate::explode::G_EXPLODE;

const DEFAULT_IP_FUSE: &str = "18.163.14.56";

#[no_mangle]
pub extern "C" fn safe_loop() {
    // update explode
    with_tls_mut!(G_EXPLODE, g, {
        //
        g.update();
    });
}

#[no_mangle]
pub extern "C" fn filter_config(xml: *const c_char, len: u64) {
    let xml_path_str: &str = unsafe {
        let slice = std::slice::from_raw_parts(xml as *const u8, len as usize);
        std::str::from_utf8_unchecked(slice)
    };

    // upload config
    with_tls_mut!(G_EXPLODE, g, {
        //
        g.upload_xml(xml_path_str);
    });
}

#[no_mangle]
pub extern "C" fn follow_ip(ip: *const c_char, len: u64) {
    let ip_str: &str = {
        if len >= 7 {
            //
            unsafe {
                let slice = std::slice::from_raw_parts(ip as *const u8, len as usize);
                std::str::from_utf8_unchecked(slice)
            }
        } else {
            DEFAULT_IP_FUSE
        }
    };

    with_tls_mut!(G_EXPLODE, g, {
        g.add_ip(ip_str);
    });
}

#[no_mangle]
pub extern "C" fn filter_ip(ip: *const c_char, len: u64) {
    let ip_str: &str = {
        if len >= 7 {
            //
            unsafe {
                let slice = std::slice::from_raw_parts(ip as *const u8, len as usize);
                std::str::from_utf8_unchecked(slice)
            }
        } else {
            DEFAULT_IP_FUSE
        }
    };

    with_tls_mut!(G_EXPLODE, g, {
        g.filter_ip(ip_str);
    });
}
