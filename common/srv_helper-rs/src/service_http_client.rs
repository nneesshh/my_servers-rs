//!
//! Common Library: service-http
//!

///
mod http_response;
pub use http_response::HttpResponse;

///
mod http_request;
pub use http_request::{HttpRequest, HttpRequestType};

///
mod http_client;

///
mod service_http_client_impl;
pub use service_http_client_impl::ServiceHttpClientRs;
