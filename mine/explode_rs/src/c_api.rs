use commlib::with_tls_mut;

use crate::explode::G_EXPLODE;

#[no_mangle]
pub extern "C" fn follow_ip(ip: *const u8, len: u64) {
    //
    let ip_str = unsafe {
        let slice = std::slice::from_raw_parts(ip, len as usize);
        std::str::from_utf8_unchecked(slice)
    };

    with_tls_mut!(G_EXPLODE, e, {
        e.add_ip(ip_str);
    });
}

#[no_mangle]
pub extern "C" fn filter_ip(ip: *const u8, len: u64) {
     //
     let ip_str = unsafe {
        let slice = std::slice::from_raw_parts(ip, len as usize);
        std::str::from_utf8_unchecked(slice)
    };

    with_tls_mut!(G_EXPLODE, e, {
        e.filter_ip(ip_str);
    });
}