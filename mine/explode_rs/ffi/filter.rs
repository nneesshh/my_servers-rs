#[cxx::bridge]
pub mod ffi {

    extern "Rust" {
        fn safe_loop();
        
        fn filter_ip(ip: &str);
        fn follow_ip(ip: &str);

        fn filter_config(xml_path: &str);

        fn string_to_u64(s: &str) -> u64;
        unsafe fn string_to_vec(s: &str, sep: &str) -> Vec<String>;
    }
}
