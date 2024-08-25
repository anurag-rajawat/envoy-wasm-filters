use log::{error, info};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{Action, ContextType, LogLevel};
use serde::{Deserialize, Serialize};
use std::time::Duration;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {Box::new(HttpHeadersRoot)});
}}

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(Telemetry {
            context_id,
            request: None,
            response: None,
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Telemetry {
    context_id: u32,
    request: Option<Reqquest>,
    response: Option<Response>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Reqquest {
    authority: String,
    path: String,
    method: String,
    scheme: String,
    request_protocol: String,
    request_id: String,
    user_agent: String,
    source_url: String,
    source_port: u16,
    destination_url: String,
    destination_port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    status_code: String,
    content_length: String,
    content_type: String,
    body: String,
    body_size: String,
}

impl Context for Telemetry {}

impl HttpContext for Telemetry {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        let (dest_url, dest_port) = get_url_and_port(
            String::from_utf8(
                self.get_property(vec!["destination", "address"])
                    .unwrap()
                    .to_vec(),
            )
            .unwrap_or(String::from("")),
        );

        let (src_url, src_port) = get_url_and_port(
            String::from_utf8(
                self.get_property(vec!["source", "address"])
                    .unwrap()
                    .to_vec(),
            )
            .unwrap_or(String::from("")),
        );

        let telemetry = Telemetry {
            context_id: self.context_id,
            request: Some(Reqquest {
                authority: self
                    .get_http_request_header(":authority")
                    .unwrap_or(String::from("")),
                path: self
                    .get_http_request_header(":path")
                    .unwrap_or(String::from("")),
                method: self
                    .get_http_request_header(":method")
                    .unwrap_or(String::from("")),
                scheme: self
                    .get_http_request_header(":scheme")
                    .unwrap_or(String::from("")),
                request_protocol: String::from_utf8(
                    self.get_property(vec!["request", "protocol"])
                        .unwrap()
                        .to_vec(),
                )
                .unwrap_or(String::from("")),
                request_id: self
                    .get_http_request_header("x-request-id")
                    .unwrap_or(String::from("")),
                user_agent: self
                    .get_http_request_header("user-agent")
                    .unwrap_or(String::from("")),
                source_url: src_url,
                source_port: src_port,
                destination_url: dest_url,
                destination_port: dest_port,
            }),
            response: None,
        };

        let telemetry_json = serde_json::to_string(&telemetry).unwrap();
        let result = self.dispatch_http_call(
            "how-to-know-about-this",
            vec![("Powered-by", "proxy-wasm")],
            Some(telemetry_json.as_bytes()),
            vec![],
            Duration::from_secs(5),
        );
        info!("Result is: {}", result.unwrap_or_default());
        info!("Request telemetry json is: {:?}", telemetry_json);
        Action::Continue
    }

    fn on_http_request_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        todo!()
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        let telemetry = Telemetry {
            context_id: self.context_id,
            request: None,
            response: Some(Response {
                status_code: self
                    .get_http_response_header(":status")
                    .unwrap_or(String::from("")),
                content_length: self
                    .get_http_response_header("content-length")
                    .unwrap_or(String::from("")),
                content_type: self
                    .get_http_response_header("content-type")
                    .unwrap_or(String::from("")),
                body: "".to_string(),
                body_size: "".to_string(),
            }),
        };

        let telemetry_json = serde_json::to_string(&telemetry).unwrap();
        info!("Response telemetry is: {:?}", telemetry);
        info!("Response telemetry json is: {:?}", telemetry_json);
        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        let body = String::from_utf8(self.get_http_response_body(0, _body_size).unwrap().to_vec())
            .unwrap();
        info!("Response body is: {:?}", body);
        Action::Continue
    }
}

fn get_url_and_port(destination_address: String) -> (String, u16) {
    let parts: Vec<&str> = destination_address.split(':').collect();

    let mut url: String = "".to_string();
    let mut port: u16 = 0;

    if parts.len() == 2 {
        url = parts[0].parse().unwrap();
        port = parts[1].parse().unwrap();
    } else {
        error!("Invalid destination address");
    }

    (url, port)
}
