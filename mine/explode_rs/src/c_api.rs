use std::ffi::c_char;

use commlib::with_tls_mut;

use crate::explode::G_EXPLODE;

const DEFAULT_IP_FUSE: &str = "18.163.14.56";

#[no_mangle]
pub extern "C" fn safe_loop() {
    // update explode
    with_tls_mut!(G_EXPLODE, e, {
        //
        e.update();
    });
    println!("safe_loop");
}

#[no_mangle]
pub extern "C" fn filter_config(xml: *const c_char, len: u64) {
    let xml_str: &str = unsafe {
        let slice = std::slice::from_raw_parts(xml as *const u8, len as usize);
        std::str::from_utf8_unchecked(slice)
    };

    // update explode
    with_tls_mut!(G_EXPLODE, e, {
        //
        e.upload_xml(xml_str);
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

    with_tls_mut!(G_EXPLODE, e, {
        e.add_ip(ip_str);
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

    with_tls_mut!(G_EXPLODE, e, {
        e.filter_ip(ip_str);
    });
}
