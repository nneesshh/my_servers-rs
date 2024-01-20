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

use super::{HttpResponse, HttpRequest, HttpRequestType};

static NEXT_TOKEN_ID: Atomic<usize> = Atomic::<usize>::new(1);

thread_local! {
    static G_HTTP_CLIENT: UnsafeCell<HttpClient> = UnsafeCell::new(HttpClient::new());
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
            //
            if let Some(req) = self.request_queue.pop_front() {
                // invoke ll_req with agent
                let ll_req = self.agent.post(req.url.as_str());
                if let Some(data) = req.data_opt {
                    //
                    let response = HttpResponse::new();

                    //
                    let ll_resp_r = ll_req
                    .set("Content-Type", "application/json")
                    .send_string(data.as_str());
                    match ll_resp_r{
                        Ok(ll_resp) => {
                            //
                            response.succeed = true;
                            response.response_code = ll_resp.status() as u32;
                            
                            for header in ll_resp.headers_names() {
                                response.response_headers.push(header);
                            }

                            //
                            match ll_resp.into_string() {
                                Ok(body) => {
                                    //
                                    response.response_rawdata = body;
                                }
                                Err(err) => {
                                    //
                                    log::error!("http_client run_loop body error: {}!!! body size overflow?!!!", err );
                                    response.error_buffer = err.to_string();
                                }
                            }
                        }
                        Err(err) => {
                            //
                            log::error!("http_client run_loop error: {}", err );
                            response.succeed = false;

                            response.error_buffer = err.to_string();

                                    
                        }
                    }

                     //
                     (req.request_cb)(&mut response);
                }
            } else {
                // queue is empty, break
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

    let request_cb = move |context: &mut HttpResponse| {
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

    let request_cb = move |response: &mut HttpResponse| {
        //
        let resp_code = response.response_code;
        let resp_data = mem::replace(&mut response.response_rawdata, "".to_owned());
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
