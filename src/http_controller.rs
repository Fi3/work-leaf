use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::CommandChat;
use crate::agent::{AgentBackend, AgentId, ReadPermission};
use crate::cli::{
    ProcessCommand, SelectedAgent, parse_process_args, render_process_help, selected_agent_backend,
};
use crate::workspace::{WorkLeafController, WorkLeafEvent, WorkLeafLoading, WorkLeafSnapshot};

#[derive(Clone, Debug)]
pub struct HttpControllerClient {
    base_url: String,
    address: String,
}

impl HttpControllerClient {
    pub fn connect(base_url: impl Into<String>) -> Result<Self, OrchestratorHttpError> {
        let base_url = base_url.into();
        let address = parse_http_address(&base_url)?;
        let client = Self {
            base_url: format!("http://{address}"),
            address,
        };
        client.health()?;
        Ok(client)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn health(&self) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.get("/health")?;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<WorkLeafSnapshot, OrchestratorHttpError> {
        self.get("/snapshot")
    }

    pub fn state(&self) -> Result<WorkLeafControllerState, OrchestratorHttpError> {
        self.get("/state")
    }

    pub fn drain_events(&self) -> Result<Vec<WorkLeafEvent>, OrchestratorHttpError> {
        self.post("/events/drain", &EmptyRequest)
    }

    pub fn is_busy(&self) -> Result<bool, OrchestratorHttpError> {
        let response: BusyResponse = self.get("/busy")?;
        Ok(response.busy)
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> Result<bool, OrchestratorHttpError> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if !self.is_busy()? {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(!self.is_busy()?)
    }

    pub fn execute_command_line(&mut self, line: &str) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post(
            "/command",
            &LineRequest {
                line: line.to_string(),
            },
        )?;
        Ok(())
    }

    pub fn send_command_agent_message(
        &mut self,
        message: &str,
    ) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post(
            "/command-agent",
            &MessageRequest {
                message: message.to_string(),
            },
        )?;
        Ok(())
    }

    pub fn send_message(
        &mut self,
        agent_id: &AgentId,
        message: &str,
    ) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post(
            "/agent/message",
            &AgentMessageRequest {
                agent_id: agent_id.clone(),
                message: message.to_string(),
            },
        )?;
        Ok(())
    }

    pub fn interrupt_agent(&mut self, agent_id: &AgentId) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post(
            "/agent/interrupt",
            &AgentRequest {
                agent_id: agent_id.clone(),
            },
        )?;
        Ok(())
    }

    pub fn push_transcript_line(&mut self, line: String) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post("/transcript", &LineRequest { line })?;
        Ok(())
    }

    pub fn loading_text(&self, loading: WorkLeafLoading) -> Result<String, OrchestratorHttpError> {
        let response: LoadingTextResponse =
            self.post("/loading-text", &LoadingTextRequest { loading })?;
        Ok(response.text)
    }

    pub fn shutdown(&mut self) -> Result<(), OrchestratorHttpError> {
        let _: OkResponse = self.post("/shutdown", &EmptyRequest)?;
        Ok(())
    }

    fn get<T>(&self, path: &str) -> Result<T, OrchestratorHttpError>
    where
        T: DeserializeOwned,
    {
        self.request::<EmptyRequest, T>("GET", path, None)
    }

    fn post<T, R>(&self, path: &str, body: &T) -> Result<R, OrchestratorHttpError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.request("POST", path, Some(body))
    }

    fn request<T, R>(
        &self,
        method: &str,
        path: &str,
        body: Option<&T>,
    ) -> Result<R, OrchestratorHttpError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let body = match body {
            Some(body) => serde_json::to_vec(body)?,
            None => Vec::new(),
        };
        let mut stream = TcpStream::connect(&self.address)?;
        write!(
            stream,
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.address,
            body.len()
        )?;
        stream.write_all(&body)?;
        stream.flush()?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response)?;
        let (status, body) = parse_http_response(&response)?;
        if !(200..300).contains(&status) {
            let error = serde_json::from_slice::<ErrorResponse>(body)
                .map(|response| response.error)
                .unwrap_or_else(|_| String::from_utf8_lossy(body).to_string());
            return Err(OrchestratorHttpError::Api(error));
        }
        Ok(serde_json::from_slice(body)?)
    }
}

#[derive(Debug)]
pub enum OrchestratorHttpError {
    Io(io::Error),
    Json(serde_json::Error),
    Protocol(String),
    Api(String),
}

impl fmt::Display for OrchestratorHttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
            Self::Protocol(message) => write!(formatter, "{message}"),
            Self::Api(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for OrchestratorHttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Protocol(_) | Self::Api(_) => None,
        }
    }
}

impl From<io::Error> for OrchestratorHttpError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for OrchestratorHttpError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug)]
pub struct HttpControllerServer {
    listener: TcpListener,
    serve_web_ui: bool,
}

impl HttpControllerServer {
    pub fn bind(address: &str) -> Result<Self, OrchestratorHttpError> {
        Self::bind_with_web_ui(address, true)
    }

    pub(crate) fn bind_api_only(address: &str) -> Result<Self, OrchestratorHttpError> {
        Self::bind_with_web_ui(address, false)
    }

    fn bind_with_web_ui(address: &str, serve_web_ui: bool) -> Result<Self, OrchestratorHttpError> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            serve_web_ui,
        })
    }

    pub fn local_url(&self) -> Result<String, OrchestratorHttpError> {
        Ok(format!("http://{}", self.listener.local_addr()?))
    }

    pub fn serve<B>(self, controller: WorkLeafController<B>) -> Result<(), OrchestratorHttpError>
    where
        B: AgentBackend + Clone + Send + 'static,
    {
        self.serve_with_parent(controller, None)
    }

    pub fn serve_with_parent<B>(
        self,
        controller: WorkLeafController<B>,
        parent_pid: Option<u32>,
    ) -> Result<(), OrchestratorHttpError>
    where
        B: AgentBackend + Clone + Send + 'static,
    {
        let controller = Arc::new(Mutex::new(controller));
        let shutdown = Arc::new(AtomicBool::new(false));
        let serve_web_ui = self.serve_web_ui;
        while !shutdown.load(Ordering::SeqCst) {
            if parent_pid.is_some_and(|pid| !process_is_alive(pid)) {
                break;
            }
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let controller = Arc::clone(&controller);
                    let shutdown = Arc::clone(&shutdown);
                    thread::spawn(move || {
                        let _ = handle_connection(stream, controller, shutdown, serve_web_ui);
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(OrchestratorHttpError::Io(error)),
            }
        }
        if let Ok(mut controller) = controller.lock() {
            controller.shutdown();
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct WebUiServer {
    listener: TcpListener,
}

impl WebUiServer {
    pub(crate) fn bind(address: &str) -> Result<Self, OrchestratorHttpError> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener })
    }

    pub(crate) fn local_url(&self) -> Result<String, OrchestratorHttpError> {
        Ok(format!("http://{}", self.listener.local_addr()?))
    }

    pub(crate) fn serve_until_shutdown(
        self,
        shutdown: Arc<AtomicBool>,
    ) -> Result<(), OrchestratorHttpError> {
        while !shutdown.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    thread::spawn(move || {
                        let _ = handle_web_ui_connection(stream);
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(OrchestratorHttpError::Io(error)),
            }
        }
        Ok(())
    }
}

pub fn run_orchestrator_from_env() -> ! {
    let command = match parse_orchestrator_args(std::env::args()) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}");
            process::exit(2);
        }
    };

    match command {
        OrchestratorProcessCommand::Help => {
            print!("{}", render_orchestrator_help());
            process::exit(0);
        }
        OrchestratorProcessCommand::Launch(config) => {
            if let Err(error) = run_orchestrator(config) {
                eprintln!("{error}");
                process::exit(1);
            }
            process::exit(0);
        }
    }
}

fn run_orchestrator(config: OrchestratorProcessConfig) -> Result<(), String> {
    let project_dir = std::env::current_dir().map_err(|error| error.to_string())?;
    let profile = config.agent.profile();
    let backend = selected_agent_backend(
        project_dir.clone(),
        config.model,
        config.read_permission,
        config.agent,
    )
    .map_err(|error| error.to_string())?;
    let chat = CommandChat::new(project_dir, backend).with_agent_profile(profile);
    let controller = WorkLeafController::new(chat);
    let server = HttpControllerServer::bind(&config.listen).map_err(|error| error.to_string())?;
    println!(
        "WORK_LEAF_ORCHESTRATOR_URL={}",
        server.local_url().map_err(|error| error.to_string())?
    );
    io::stdout().flush().map_err(|error| error.to_string())?;
    let parent_pid = std::env::var("WORK_LEAF_PARENT_PID")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    server
        .serve_with_parent(controller, parent_pid)
        .map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OrchestratorProcessConfig {
    listen: String,
    model: Option<String>,
    read_permission: ReadPermission,
    agent: SelectedAgent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum OrchestratorProcessCommand {
    Help,
    Launch(OrchestratorProcessConfig),
}

fn parse_orchestrator_args<I, S>(args: I) -> Result<OrchestratorProcessCommand, crate::CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| {
        arg.ends_with("work-leaf-orchestrator") || arg.ends_with("work-leaf-orchestrator.exe")
    }) {
        args.remove(0);
    }

    let mut listen = "127.0.0.1:7878".to_string();
    let mut process_args = vec!["work-leaf".to_string()];
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--listen" => {
                if index + 1 >= args.len() {
                    return Err(crate::CliError::Usage(
                        "--listen requires a value".to_string(),
                    ));
                }
                listen = args[index + 1].clone();
                index += 2;
            }
            other => {
                process_args.push(other.to_string());
                index += 1;
            }
        }
    }

    match parse_process_args(process_args)? {
        ProcessCommand::Help => Ok(OrchestratorProcessCommand::Help),
        ProcessCommand::Launch {
            model,
            read_permission,
            agent,
            cli_url,
        } => {
            if cli_url.is_some() {
                return Err(crate::CliError::Usage(
                    "work-leaf-orchestrator cannot connect to an existing HTTP API; use work-leaf --cli <url>".to_string(),
                ));
            }
            Ok(OrchestratorProcessCommand::Launch(
                OrchestratorProcessConfig {
                    listen,
                    model,
                    read_permission,
                    agent,
                },
            ))
        }
        ProcessCommand::Daemon {
            model,
            read_permission,
            agent,
        } => Ok(OrchestratorProcessCommand::Launch(
            OrchestratorProcessConfig {
                listen,
                model,
                read_permission,
                agent,
            },
        )),
    }
}

fn render_orchestrator_help() -> String {
    let mut help = render_process_help();
    help.push_str("Daemon options:\n");
    help.push_str("  --listen <addr>         bind the localhost HTTP API address\n");
    help
}

fn handle_connection<B>(
    mut stream: TcpStream,
    controller: Arc<Mutex<WorkLeafController<B>>>,
    shutdown: Arc<AtomicBool>,
    serve_web_ui: bool,
) -> Result<(), OrchestratorHttpError>
where
    B: AgentBackend + Clone + Send + 'static,
{
    let Some(request) = read_http_request(&mut stream)? else {
        return Ok(());
    };
    let reply = route_request(request, controller, shutdown, serve_web_ui);
    write_http_reply(&mut stream, reply)?;
    Ok(())
}

fn handle_web_ui_connection(mut stream: TcpStream) -> Result<(), OrchestratorHttpError> {
    let Some(request) = read_http_request(&mut stream)? else {
        return Ok(());
    };
    let reply = route_web_ui_request(request);
    write_http_reply(&mut stream, reply)?;
    Ok(())
}

fn route_request<B>(
    request: HttpRequest,
    controller: Arc<Mutex<WorkLeafController<B>>>,
    shutdown: Arc<AtomicBool>,
    serve_web_ui: bool,
) -> HttpReply
where
    B: AgentBackend + Clone + Send + 'static,
{
    let path = request
        .path
        .split('?')
        .next()
        .unwrap_or(request.path.as_str());
    if request.method == "OPTIONS" {
        return cors_preflight_reply();
    }
    if serve_web_ui && let Some(reply) = web_ui_reply(request.method.as_str(), path) {
        return reply;
    }
    match (request.method.as_str(), path) {
        ("GET", "/health") => json_reply(200, &OkResponse { ok: true }),
        ("GET", "/snapshot") => with_controller(&controller, |controller| controller.snapshot()),
        ("GET", "/state") => with_controller(&controller, |controller| {
            let busy = controller.is_busy();
            WorkLeafControllerState {
                busy,
                snapshot: controller.snapshot(),
            }
        }),
        ("POST", "/events/drain") => with_controller(&controller, WorkLeafController::drain_events),
        ("GET", "/busy") => with_controller(&controller, |controller| BusyResponse {
            busy: controller.is_busy(),
        }),
        ("POST", "/command") => {
            let body = match decode_body::<LineRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| {
                controller.execute_command_line(&body.line);
                OkResponse { ok: true }
            })
        }
        ("POST", "/command-agent") => {
            let body = match decode_body::<MessageRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| {
                controller.send_command_agent_message(&body.message);
                OkResponse { ok: true }
            })
        }
        ("POST", "/agent/message") => {
            let body = match decode_body::<AgentMessageRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller_result(&controller, |controller| {
                controller
                    .send_message(&body.agent_id, &body.message)
                    .map(|()| OkResponse { ok: true })
            })
        }
        ("POST", "/agent/interrupt") => {
            let body = match decode_body::<AgentRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| {
                controller.interrupt_agent(&body.agent_id);
                OkResponse { ok: true }
            })
        }
        ("POST", "/transcript") => {
            let body = match decode_body::<LineRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| {
                controller.push_transcript_line(body.line);
                OkResponse { ok: true }
            })
        }
        ("POST", "/loading-text") => {
            let body = match decode_body::<LoadingTextRequest>(&request) {
                Ok(body) => body,
                Err(reply) => return reply,
            };
            with_controller(&controller, |controller| LoadingTextResponse {
                text: controller.loading_text(body.loading),
            })
        }
        ("POST", "/shutdown") => {
            with_controller(&controller, WorkLeafController::shutdown);
            shutdown.store(true, Ordering::SeqCst);
            json_reply(200, &OkResponse { ok: true })
        }
        _ => error_reply(404, "not found"),
    }
}

fn route_web_ui_request(request: HttpRequest) -> HttpReply {
    let path = request
        .path
        .split('?')
        .next()
        .unwrap_or(request.path.as_str());
    if request.method == "OPTIONS" {
        return cors_preflight_reply();
    }
    web_ui_reply(request.method.as_str(), path).unwrap_or_else(|| error_reply(404, "not found"))
}

fn web_ui_reply(method: &str, path: &str) -> Option<HttpReply> {
    match (method, path) {
        ("GET", "/") | ("GET", "/web-ui") | ("GET", "/web-ui/") => Some(static_reply(
            "text/html; charset=utf-8",
            include_bytes!("../web-ui/index.html"),
        )),
        ("GET", "/styles.css") | ("GET", "/web-ui/styles.css") => Some(static_reply(
            "text/css; charset=utf-8",
            include_bytes!("../web-ui/styles.css"),
        )),
        ("GET", "/app.js") | ("GET", "/web-ui/app.js") => Some(static_reply(
            "text/javascript; charset=utf-8",
            include_bytes!("../web-ui/app.js"),
        )),
        _ => None,
    }
}

fn with_controller<B, T, F>(
    controller: &Arc<Mutex<WorkLeafController<B>>>,
    operation: F,
) -> HttpReply
where
    B: AgentBackend + Clone + Send + 'static,
    T: Serialize,
    F: FnOnce(&mut WorkLeafController<B>) -> T,
{
    match controller.lock() {
        Ok(mut controller) => json_reply(200, &operation(&mut controller)),
        Err(_) => error_reply(500, "controller mutex poisoned"),
    }
}

fn with_controller_result<B, T, F>(
    controller: &Arc<Mutex<WorkLeafController<B>>>,
    operation: F,
) -> HttpReply
where
    B: AgentBackend + Clone + Send + 'static,
    T: Serialize,
    F: FnOnce(&mut WorkLeafController<B>) -> Result<T, crate::CliError>,
{
    match controller.lock() {
        Ok(mut controller) => match operation(&mut controller) {
            Ok(value) => json_reply(200, &value),
            Err(error) => error_reply(400, &error.to_string()),
        },
        Err(_) => error_reply(500, "controller mutex poisoned"),
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

#[derive(Debug)]
struct HttpReply {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
struct EmptyRequest;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkLeafControllerState {
    pub busy: bool,
    pub snapshot: WorkLeafSnapshot,
}

#[derive(Deserialize, Serialize)]
struct OkResponse {
    ok: bool,
}

#[derive(Deserialize, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize, Serialize)]
struct BusyResponse {
    busy: bool,
}

#[derive(Deserialize, Serialize)]
struct LineRequest {
    line: String,
}

#[derive(Deserialize, Serialize)]
struct MessageRequest {
    message: String,
}

#[derive(Deserialize, Serialize)]
struct AgentRequest {
    agent_id: AgentId,
}

#[derive(Deserialize, Serialize)]
struct AgentMessageRequest {
    agent_id: AgentId,
    message: String,
}

#[derive(Deserialize, Serialize)]
struct LoadingTextRequest {
    loading: WorkLeafLoading,
}

#[derive(Deserialize, Serialize)]
struct LoadingTextResponse {
    text: String,
}

fn read_http_request(stream: &mut TcpStream) -> Result<Option<HttpRequest>, OrchestratorHttpError> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(None);
    }
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP method".to_string()))?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP path".to_string()))?
        .to_string();

    let mut content_length = 0_usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        let header = header.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse::<usize>().map_err(|_| {
                OrchestratorHttpError::Protocol("invalid Content-Length".to_string())
            })?;
        }
    }

    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    Ok(Some(HttpRequest { method, path, body }))
}

fn decode_body<T>(request: &HttpRequest) -> Result<T, HttpReply>
where
    T: DeserializeOwned,
{
    serde_json::from_slice(&request.body).map_err(|error| error_reply(400, &error.to_string()))
}

fn write_http_reply(stream: &mut TcpStream, reply: HttpReply) -> Result<(), OrchestratorHttpError> {
    let status_text = status_text(reply.status);
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Accept\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reply.status,
        status_text,
        reply.content_type,
        reply.body.len()
    )?;
    stream.write_all(&reply.body)?;
    stream.flush()?;
    Ok(())
}

fn json_reply<T>(status: u16, body: &T) -> HttpReply
where
    T: Serialize,
{
    match serde_json::to_vec(body) {
        Ok(body) => HttpReply {
            status,
            content_type: "application/json",
            body,
        },
        Err(error) => error_reply(500, &error.to_string()),
    }
}

fn static_reply(content_type: &'static str, body: &'static [u8]) -> HttpReply {
    HttpReply {
        status: 200,
        content_type,
        body: body.to_vec(),
    }
}

fn cors_preflight_reply() -> HttpReply {
    json_reply(200, &OkResponse { ok: true })
}

fn error_reply(status: u16, error: &str) -> HttpReply {
    json_reply(
        status,
        &ErrorResponse {
            error: error.to_string(),
        },
    )
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn parse_http_response(response: &[u8]) -> Result<(u16, &[u8]), OrchestratorHttpError> {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP response headers".into()))?;
    let headers = std::str::from_utf8(&response[..header_end])
        .map_err(|_| OrchestratorHttpError::Protocol("HTTP headers are not UTF-8".into()))?;
    let status_line = headers
        .lines()
        .next()
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP status line".into()))?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| OrchestratorHttpError::Protocol("missing HTTP status code".into()))?
        .parse::<u16>()
        .map_err(|_| OrchestratorHttpError::Protocol("invalid HTTP status code".into()))?;
    Ok((status, &response[header_end + 4..]))
}

fn parse_http_address(base_url: &str) -> Result<String, OrchestratorHttpError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    let address = trimmed.strip_prefix("http://").ok_or_else(|| {
        OrchestratorHttpError::Protocol(
            "orchestrator URL must start with http:// and point at localhost".to_string(),
        )
    })?;
    if address.is_empty() || address.contains('/') {
        return Err(OrchestratorHttpError::Protocol(
            "orchestrator URL must not include a path".to_string(),
        ));
    }
    if !address.starts_with("127.0.0.1:") && !address.starts_with("localhost:") {
        return Err(OrchestratorHttpError::Protocol(
            "orchestrator URL must use localhost".to_string(),
        ));
    }
    Ok(address.to_string())
}

#[cfg(target_os = "linux")]
fn process_is_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(not(target_os = "linux"))]
fn process_is_alive(_pid: u32) -> bool {
    true
}
