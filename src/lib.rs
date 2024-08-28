use log::{error, warn};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{Action, ContextType, LogLevel};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default)]
struct Plugin {
    _context_id: u32,
    config: PluginConfig,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct PluginConfig {
    upstream_name: String,
    api_path: String,
    authority: String,
}

#[derive(Serialize)]
struct Telemetry {
    telemetry_type: Type,
    request: Option<Reqquest>,
    response: Option<Ressponse>,
}

#[derive(Serialize)]
enum Type {
    RequestHeader,
    Request,
    ResponseHeader,
    Response,
}

#[derive(Serialize)]
struct Reqquest {
    headers: Option<ReqHeaders>,
    source_url: String,
    source_port: u16,
    destination_url: String,
    destination_port: u16,
}

#[derive(Serialize)]
struct Ressponse {
    headers: Option<ResHeaders>,
    body: String,
}

#[derive(Serialize)]
struct ReqHeaders {
    authority: String,
    path: String,
    method: String,
    scheme: String,
    protocol: String,
    request_id: String,
    user_agent: String,
}

#[derive(Serialize)]
struct ResHeaders {
    status_code: u16,
    content_length: u32,
    content_type: String,
}

fn _start() {
    proxy_wasm::main! {{
        proxy_wasm::set_log_level(LogLevel::Info);
        proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {Box::new(Plugin::default())});
    }}
}

impl Context for Plugin {}

impl RootContext for Plugin {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            if let Ok(config) = serde_json::from_slice::<PluginConfig>(&config_bytes) {
                self.config = config;
            } else {
                error!("Failed to parse plugin config");
            }
        } else {
            warn!("No plugin config found");
        }
        true
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(Plugin {
            _context_id,
            config: self.config.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

impl HttpContext for Plugin {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        let (dest_url, dest_port) = get_url_and_port(
            String::from_utf8(
                self.get_property(vec!["destination", "address"])
                    .unwrap()
                    .to_vec(),
            )
            .unwrap_or("".to_string()),
        );

        let (src_url, src_port) = get_url_and_port(
            String::from_utf8(
                self.get_property(vec!["source", "address"])
                    .unwrap()
                    .to_vec(),
            )
            .unwrap_or("".to_string()),
        );

        let telemetry = Telemetry {
            telemetry_type: Type::RequestHeader,
            request: Some(Reqquest {
                headers: Some(construct_req_headers(self)),
                source_url: src_url,
                source_port: src_port,
                destination_url: dest_url,
                destination_port: dest_port,
            }),
            response: None,
        };

        let telemetry_json = serde_json::to_string(&telemetry).unwrap();
        dispatch_http_call_to_sentryflow(self, telemetry_json)
    }

    fn on_http_request_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        todo!()
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        let telemetry = Telemetry {
            telemetry_type: Type::ResponseHeader,
            request: None,
            response: Some(Ressponse {
                headers: Some(construct_res_headers(self)),
                body: "".to_string(),
            }),
        };

        let telemetry_json = serde_json::to_string(&telemetry).unwrap();
        dispatch_http_call_to_sentryflow(self, telemetry_json)
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        let body = String::from_utf8(self.get_http_response_body(0, _body_size).unwrap().to_vec())
            .unwrap();

        let telemetry = Telemetry {
            telemetry_type: Type::Response,
            request: None,
            response: Some(Ressponse {
                headers: None,
                body,
            }),
        };

        let telemetry_json = serde_json::to_string(&telemetry).unwrap();
        dispatch_http_call_to_sentryflow(self, telemetry_json)
    }
}

fn dispatch_http_call_to_sentryflow(obj: &mut Plugin, telemetry: String) -> Action {
    // Todo: Configure it to consume provided plugin config
    const UPSTREAM: &str = "filterserver";

    let headers = vec![
        (":method", "POST"),
        (":authority", "sentryflow"),
        (":path", "/api/telemetry"),
        ("accept", "*/*"),
        ("Content-Type", "application/json"),
    ];

    let http_call_res = obj.dispatch_http_call(
        UPSTREAM,
        headers,
        Some(telemetry.as_bytes()),
        vec![],
        Duration::from_secs(1),
    );

    if http_call_res.is_err() {
        error!(
            "Failed to dispatch HTTP call, to '{}' status: {http_call_res:#?}",
            UPSTREAM,
        );
    }

    Action::Continue
}

fn construct_res_headers(obj: &mut Plugin) -> ResHeaders {
    let status_code: u16 = obj
        .get_http_response_header(":status")
        .unwrap()
        .parse()
        .unwrap_or(0);

    let content_length: u32 = obj
        .get_http_response_header("content-length")
        .unwrap()
        .parse()
        .unwrap_or(0);

    ResHeaders {
        status_code,
        content_length,
        content_type: obj
            .get_http_response_header("content-type")
            .unwrap_or("".to_string()),
    }
}

fn construct_req_headers(obj: &mut Plugin) -> ReqHeaders {
    ReqHeaders {
        authority: obj
            .get_http_request_header(":authority")
            .unwrap_or("".to_string()),
        path: obj
            .get_http_request_header(":path")
            .unwrap_or("".to_string()),
        method: obj
            .get_http_request_header(":method")
            .unwrap_or("".to_string()),
        scheme: obj
            .get_http_request_header(":scheme")
            .unwrap_or("".to_string()),
        protocol: String::from_utf8(
            obj.get_property(vec!["request", "protocol"])
                .unwrap()
                .to_vec(),
        )
        .unwrap_or("".to_string()),
        request_id: obj
            .get_http_request_header("x-request-id")
            .unwrap_or("".to_string()),
        user_agent: obj
            .get_http_request_header("user-agent")
            .unwrap_or("".to_string()),
    }
}

fn get_url_and_port(address: String) -> (String, u16) {
    let parts: Vec<&str> = address.split(':').collect();

    let mut url = "".to_string();
    let mut port = 0;

    if parts.len() == 2 {
        url = parts[0].parse().unwrap();
        port = parts[1].parse().unwrap();
    } else {
        error!("Invalid address");
    }

    (url, port)
}
