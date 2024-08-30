use log::{error, warn};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{Action, ContextType, LogLevel};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Default)]
struct Plugin {
    _context_id: u32,
    config: PluginConfig,
    telemetry: Telemetry,
}

#[derive(Deserialize, Clone, Default)]
struct PluginConfig {
    upstream_name: String,
    api_path: String,
    authority: String,
}

#[derive(Serialize, Default, Clone)]
struct Telemetry {
    context_id: u32,
    request: Reqquest,
    response: Ressponse,
}

#[derive(Serialize, Clone, Default)]
struct Reqquest {
    headers: ReqHeaders,
    source_url: String,
    source_port: u16,
    destination_url: String,
    destination_port: u16,
    body: String,
}

#[derive(Serialize, Clone, Default)]
struct Ressponse {
    headers: ResHeaders,
    body: String,
}

#[derive(Serialize, Clone, Default)]
struct ReqHeaders {
    host: String,
    path: String,
    method: String,
    scheme: String,
    protocol: String,
    request_id: String,
    user_agent: String,
}

#[derive(Serialize, Clone, Default)]
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

impl Context for Plugin {
    fn on_done(&mut self) -> bool {
        dispatch_http_call_to_upstream(self);
        false
    }
}

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
            telemetry: Default::default(),
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
                    .unwrap_or_default(),
            )
            .unwrap_or_default(),
        );

        let (src_url, src_port) = get_url_and_port(
            String::from_utf8(
                self.get_property(vec!["source", "address"])
                    .unwrap_or_default(),
            )
            .unwrap_or_default(),
        );

        self.telemetry.context_id = self._context_id;
        self.telemetry.request = Reqquest {
            headers: construct_req_headers(self),
            source_url: src_url,
            source_port: src_port,
            destination_url: dest_url,
            destination_port: dest_port,
            body: "".to_string(),
        };

        Action::Continue
    }

    fn on_http_request_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        // Currently, we're sending the entire HTTP response body. We might need to
        // implement a size limit. For example, if the body size exceeds a certain threshold,
        // we could choose not to send it.
        let body = String::from_utf8(
            self.get_http_request_body(0, _body_size)
                .unwrap_or_default(),
        )
        .unwrap_or_default();
        self.telemetry.request.body = body;

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        self.telemetry.response = Ressponse {
            headers: construct_res_headers(self),
            body: "".to_string(),
        };

        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        let body = String::from_utf8(
            self.get_http_response_body(0, _body_size)
                .unwrap_or_default(),
        )
        .unwrap_or_default();

        let existing_res_headers = self.telemetry.response.headers.clone();
        self.telemetry.response = Ressponse {
            headers: existing_res_headers,
            body,
        };

        Action::Continue
    }
}

fn dispatch_http_call_to_upstream(obj: &mut Plugin) {
    let telemetry_json = serde_json::to_string(&obj.telemetry).unwrap_or_default();

    let headers = vec![
        (":method", "POST"),
        (":authority", &obj.config.authority),
        (":path", &obj.config.api_path),
        ("accept", "*/*"),
        ("Content-Type", "application/json"),
    ];

    let http_call_res = obj.dispatch_http_call(
        &obj.config.upstream_name,
        headers,
        Some(telemetry_json.as_bytes()),
        vec![],
        Duration::from_secs(1),
    );

    if http_call_res.is_err() {
        error!(
            "Failed to dispatch HTTP call, to '{}' status: {http_call_res:#?}",
            &obj.config.upstream_name,
        );
    }
}

fn construct_res_headers(obj: &mut Plugin) -> ResHeaders {
    let status_code: u16 = obj
        .get_http_response_header(":status")
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    let content_length: u32 = obj
        .get_http_response_header("content-length")
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    ResHeaders {
        status_code,
        content_length,
        content_type: obj
            .get_http_response_header("content-type")
            .unwrap_or_default(),
    }
}

fn construct_req_headers(obj: &mut Plugin) -> ReqHeaders {
    ReqHeaders {
        host: obj
            .get_http_request_header(":authority")
            .unwrap_or_default(),
        path: obj.get_http_request_header(":path").unwrap_or_default(),
        method: obj.get_http_request_header(":method").unwrap_or_default(),
        scheme: obj.get_http_request_header(":scheme").unwrap_or_default(),
        protocol: String::from_utf8(
            obj.get_property(vec!["request", "protocol"])
                .unwrap_or_default(),
        )
        .unwrap_or_default(),
        request_id: obj
            .get_http_request_header("x-request-id")
            .unwrap_or_default(),
        user_agent: obj
            .get_http_request_header("user-agent")
            .unwrap_or_default(),
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
