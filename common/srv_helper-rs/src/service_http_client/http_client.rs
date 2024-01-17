//!
//! Commlib: HttpClient
//!

use atomic::{Atomic, Ordering};
use parking_lot::RwLock;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::mem;
use std::sync::Arc;
use std::time::Duration;

use ureq::{Agent, AgentBuilder};

use commlib::with_tls_mut;

use crate::{ServiceHttpClientRs, ServiceRs};

use super::{HttpContext, HttpRequest, HttpRequestType};

static NEXT_TOKEN_ID: Atomic<usize> = Atomic::<usize>::new(1);

thread_local! {
    static G_HTTP_CLIENT: UnsafeCell<HttpClient> = UnsafeCell::new(HttpClient::new());

    static G_CURL_PAYLOAD_STORAGE: UnsafeCell<CurlPayloadStorage> = UnsafeCell::new(CurlPayloadStorage::new());
}

struct CurlPayload {
    easy_handle: EasyHandle, // EasyHandle owns raw pointer, can send across thread
    context: Arc<RwLock<HttpContext>>,
}

struct CurlPayloadStorage {
    /// custom handle table
    payload_table: hashbrown::HashMap<usize, CurlPayload>, // token 2 payload
}

impl CurlPayloadStorage {
    ///
    pub fn new() -> Self {
        Self {
            payload_table: hashbrown::HashMap::with_capacity(256),
        }
    }
}

///
pub struct HttpClient {
    request_queue: VecDeque<HttpRequest>,
    agent: Agent,
}

impl HttpClient {
    ///
    pub fn new() -> Self {
        let builder = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5));

        Self {
            request_queue: VecDeque::with_capacity(64),
            agent:  builder.build()
        }
    }

    ///
    pub fn send(&mut self, req: HttpRequest, srv_http_cli: &ServiceHttpClientRs) {
        // 运行于 srv_http_cli 线程
        assert!(srv_http_cli.is_in_service_thread());

        self.enqueue(req);
    }

    ///
    pub fn run_loop(&mut self, srv_http_cli: &ServiceHttpClientRs) {
        // 运行于 srv_http_cli 线程
        assert!(srv_http_cli.is_in_service_thread());

        // process requests
        const MAX_REQUESTS: usize = 100_usize;
        let mut count = 0_usize;
        while count <= MAX_REQUESTS {
            if let Some(req) = self.request_queue.pop_front() {
                //
                let context = Arc::new(RwLock::new(HttpContext::new(req)));

                let ll_req = self.agent.post(req.url.as_str());
                if let Some(data) = req.data_opt {
                    let resp_r = ll_req
                    .set("Content-Type", "application/json")
                    .send_string(req.data_opt);

                    //
                    {
                        let mut guard = context.write();
                        match resp_r{
                            Ok(resp) => {
                                //
                                let mut guard = context.write();
                                guard.response.succeed = true;
                                guard.response.response_code = resp.status();
                                
                                // for header in resp.headers_names() {
                                //     guard.response.response_headers.push(header);
                                // }

                                //
                                // match resp.into_string() {
                                //     Ok(body) => {
                                //         //
                                //         guard.response.response_rawdata = body;
                                //     }
                                //     Err(err) => {
                                //         //
                                //         log::error!("http_client run_loop body error: {}!!! body size overflow?!!!", err );
                                //         guard.response.error_buffer = err.to_string();
                                //     }
                                // }

                                //
                                req.request_cb(&context);
                            }
                            Err(err) => {
                                //
                                log::error!("http_client run_loop error: {}", err );
                                        guard.response.error_buffer = err.to_string();
                            }
                        }
                    }
                }
            } else {
                break;
            }

            //
            count += 1;
        }
    }

    #[inline(always)]
    fn enqueue(&mut self, req: HttpRequest) {
        self.request_queue.push_back(req);
    }
}

///
#[inline(always)]
pub fn http_client_update(srv_http_cli: &ServiceHttpClientRs) {
    // 运行于 srv_http_cli 线程
    assert!(srv_http_cli.is_in_service_thread());

    with_tls_mut!(G_HTTP_CLIENT, g, {
        g.run_loop(srv_http_cli);
    });
}

///
pub fn http_client_get<F>(
    url: String,
    headers: Vec<String>,
    cb: F,
    srv_http_cli: &Arc<ServiceHttpClientRs>,
) where
    F: Fn(u32, String) + Send + Sync + 'static,
{
    // 运行于 srv_http_cli 线程
    assert!(srv_http_cli.is_in_service_thread());

    let request_cb = move |context: &mut HttpContext| {
        //
        let resp_code = context.response.response_code;
        let resp_data = mem::replace(&mut context.response.response_rawdata, "".to_owned());
        cb(resp_code, resp_data);
    };
    let req = HttpRequest {
        r#type: HttpRequestType::GET, // Method: GET
        url,
        data_opt: None,
        headers,
        request_cb: Arc::new(request_cb),
    };

    with_tls_mut!(G_HTTP_CLIENT, g, {
        g.send(req, srv_http_cli);
    });
}

///
pub fn http_client_post<F>(
    url: String,
    data: String,
    headers: Vec<String>,
    cb: F,
    srv_http_cli: &Arc<ServiceHttpClientRs>,
) where
    F: Fn(u32, String) + Send + Sync + 'static,
{
    // 运行于 srv_http_cli 线程
    assert!(srv_http_cli.is_in_service_thread());

    let request_cb = move |context: &mut HttpContext| {
        //
        let resp_code = context.response.response_code;
        let resp_data = mem::replace(&mut context.response.response_rawdata, "".to_owned());
        cb(resp_code, resp_data);
    };
    let req = HttpRequest {
        r#type: HttpRequestType::POST, // Method: POST
        url,
        data_opt: Some(data),
        headers,
        request_cb: Arc::new(request_cb),
    };

    with_tls_mut!(G_HTTP_CLIENT, g, {
        g.send(req, srv_http_cli);
    });
}


#[cfg(test)]
mod http_test {
    use serde_json::json;

    use crate::{launch_service, G_SERVICE_HTTP_CLIENT};

    #[test]
    fn test_http_client() {
        let body = json!({"foo": false, "bar": null, "answer": 42, "list": [null, "world", true]})
            .to_string();

        //
        let srv_http_cli = G_SERVICE_HTTP_CLIENT.clone();

        launch_service(&srv_http_cli, || {
            //
        });

        srv_http_cli.http_post(
            "http://127.0.0.1:7878",
            vec!["Content-Type: application/json".to_owned()],
            body,
            |code, resp| {
                //
                log::info!("hello http code: {}, resp: {}", code, resp);
            },
        )
    }
}
