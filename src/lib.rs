use log::error;
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{Action, ContextType, LogLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    headers: HashMap<String, String>,
    source_url: String,
    source_port: u16,
    source_namespace: String,
    destination_url: String,
    destination_port: u16,
    destination_namespace: String,
    body: String,
}

#[derive(Serialize, Clone, Default, Debug)]
struct Ressponse {
    headers: HashMap<String, String>,
    body: String,
}

fn _start() {
    proxy_wasm::main! {{
        proxy_wasm::set_log_level(LogLevel::Warn);
        proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {Box::new(Plugin::default())});
    }}
}

impl Context for Plugin {
    fn on_done(&mut self) -> bool {
        find_and_update_dest_namespace(self);
        dispatch_http_call_to_upstream(self);
        true
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
            error!("No plugin config found");
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

        let req_headers = self.get_http_request_headers();
        let mut headers: HashMap<String, String> = HashMap::with_capacity(req_headers.len());
        for header in req_headers {
            headers.insert(header.0, header.1);
        }

        self.telemetry.context_id = self._context_id;
        self.telemetry.request.headers = headers;
        self.telemetry.request.source_url = src_url;
        self.telemetry.request.source_port = src_port;
        self.telemetry.request.destination_url = dest_url;
        self.telemetry.request.destination_port = dest_port;
        find_and_update_src_namespace(self);

        Action::Continue
    }

    fn on_http_request_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        // Currently, we're sending the entire HTTP request body. We might need to
        // implement a size limit. For example, if the body size exceeds a certain threshold,
        // we could choose not to send it.
        let body = String::from_utf8(
            self.get_http_request_body(0, _body_size)
                .unwrap_or_default(),
        )
        .unwrap_or_default();

        if !body.is_empty() {
            self.telemetry.request.body = body;
        }
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        let res_headers = self.get_http_response_headers();
        let mut headers: HashMap<String, String> = HashMap::with_capacity(res_headers.len());
        for res_header in res_headers {
            headers.insert(res_header.0, res_header.1);
        }

        self.telemetry.response.headers = headers;
        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        // Currently, we're sending the entire HTTP response body. We might need to
        // implement a size limit. For example, if the body size exceeds a certain threshold,
        // we could choose not to send it.
        let body = String::from_utf8(
            self.get_http_response_body(0, _body_size)
                .unwrap_or_default(),
        )
        .unwrap_or_default();
        if !body.is_empty() {
            self.telemetry.response.body = body;
        }
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

fn find_and_update_src_namespace(obj: &mut Plugin) {
    // Dirty way to get the source namespace
    // "sidecar~10.244.0.10~httpd-c6d6cb94b-k6rv6.default~default.svc.cluster.local",
    // sidecar~<POD_IP>~<POD_NAME.NAMESPACE><SERVICE>
    let src_ns = obj
        .get_http_request_header("x-envoy-peer-metadata-id")
        .unwrap_or("".to_string());
    if !src_ns.is_empty() {
        let parts: Vec<&str> = src_ns.split("~").collect();
        let svc_parts: Vec<&str> = parts[3].split(".").collect();
        obj.telemetry.request.source_namespace = svc_parts[0].to_string();
    }
}

fn find_and_update_dest_namespace(obj: &mut Plugin) {
    let dest_ns = String::from_utf8(
        obj.get_property(vec![
            "upstream_host_metadata",
            "filter_metadata",
            "istio",
            "workload",
        ])
        .unwrap_or_default(),
    )
    .unwrap_or_default();

    // e.g., filterserver;sentryflow;filterserver;;Kubernetes
    if !dest_ns.is_empty() {
        let parts: Vec<&str> = dest_ns.split(";").collect();
        if parts.len() == 5 || parts.len() == 4 {
            obj.telemetry.request.destination_namespace = parts[1].to_string();
        }
    }
}
