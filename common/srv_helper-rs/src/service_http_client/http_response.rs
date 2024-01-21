//!
//! Commlib: HttpResponse
//!

///
#[repr(C)]
pub struct HttpResponse {
    pub succeed: bool,      // indicate if the http request is successful processed
    pub response_code: u32, // the status code RETURNed from libcurl, e.g. 200, 404 ...

    pub header_table: hashbrown::HashMap<String, Vec<String>>, // the RETURNed header table
    pub error_buffer: String, // if response_code != 200, please read error_buffer to find the reason
    pub response_rawdata: String, // the RETURNed raw data
}

impl HttpResponse {
    ///
    pub fn new() -> Self {
        Self {
            succeed: false,
            response_code: 0,
            header_table: hashbrown::HashMap::new(),

            error_buffer: "".to_owned(),
            response_rawdata: "".to_owned(),
        }
    }
}
