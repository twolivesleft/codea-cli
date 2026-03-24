use anyhow::{Context, Result, anyhow, bail};
use base64::{Engine as _, engine::general_purpose};
use reqwest::blocking::{Client, Response};
use reqwest::header::CONTENT_TYPE;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader};
use std::time::Duration;

const PROTOCOL_VERSION: &str = "2024-11-05";

pub struct MCPClient {
    client: Client,
    url: String,
    next_id: u64,
    initialized: bool,
}

impl MCPClient {
    pub fn new(host: &str, port: u16, timeout_secs: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;
        Ok(Self {
            client,
            url: format!("http://{}:{}/mcp", host, port),
            next_id: 1,
            initialized: false,
        })
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn post_json(&self, payload: &Value) -> Result<Value> {
        let response = self
            .client
            .post(&self.url)
            .header(CONTENT_TYPE, "application/json")
            .json(payload)
            .send()?;
        parse_response(response)
    }

    fn notify(&self, method: &str, params: Value) -> Result<()> {
        let payload = json!({"jsonrpc": "2.0", "method": method, "params": params});
        let _ = self.post_json(&payload)?;
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "initialize",
            "params": {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "codea", "version": "0.1.0"}
            }
        });
        let _ = self.post_json(&payload)?;
        self.notify("notifications/initialized", json!({}))?;
        self.initialized = true;
        Ok(())
    }

    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        if !self.initialized {
            self.initialize()?;
        }

        let id = self.next_id();
        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments}
        });
        let response = self.post_json(&payload)?;

        if let Some(error) = response.get("error") {
            bail!("MCP error: {}", error);
        }

        let result = response.get("result").cloned().unwrap_or_else(|| json!({}));
        if result
            .get("isError")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            let text = result
                .get("content")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(|item| item.get("text"))
                .and_then(Value::as_str)
                .unwrap_or("Unknown error");
            bail!("Tool error: {}", text);
        }

        Ok(result)
    }

    pub fn text(result: &Value) -> String {
        result
            .get("content")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|item| item.get("type").and_then(Value::as_str) == Some("text"))
            .and_then(|item| item.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    }

    pub fn json_result(result: &Value) -> Result<Value> {
        serde_json::from_str(&Self::text(result)).context("Failed to decode JSON result")
    }

    pub fn image_bytes(result: &Value) -> Result<Option<Vec<u8>>> {
        for item in result
            .get("content")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if item.get("type").and_then(Value::as_str) == Some("image") {
                let data = item
                    .get("data")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("Missing image data"))?;
                return Ok(Some(general_purpose::STANDARD.decode(data)?));
            }
        }
        Ok(None)
    }

    pub fn list_projects(&mut self) -> Result<Vec<String>> {
        parse_string_array(&Self::json_result(
            &self.call_tool("listProjects", json!({}))?,
        )?)
    }

    pub fn list_files(&mut self, project_path: &str) -> Result<Vec<String>> {
        parse_string_array(&Self::json_result(
            &self.call_tool("listFiles", json!({"path": project_path}))?,
        )?)
    }

    pub fn list_dependencies(&mut self, project_path: &str) -> Result<Vec<String>> {
        parse_string_array(&Self::json_result(
            &self.call_tool("listDependencies", json!({"path": project_path}))?,
        )?)
    }

    pub fn read_file(&mut self, file_path: &str) -> Result<String> {
        Ok(Self::text(
            &self.call_tool("readFile", json!({"path": file_path}))?,
        ))
    }

    pub fn write_file(&mut self, file_path: &str, content: &str) -> Result<()> {
        let _ = self.call_tool("writeFile", json!({"path": file_path, "content": content}))?;
        Ok(())
    }

    pub fn run_project(&mut self, project_path: &str) -> Result<String> {
        Ok(Self::text(
            &self.call_tool("runProject", json!({"path": project_path}))?,
        ))
    }

    pub fn stop_project(&mut self) -> Result<String> {
        Ok(Self::text(&self.call_tool("stopProject", json!({}))?))
    }

    pub fn execute_lua(&mut self, code: &str) -> Result<String> {
        Ok(Self::text(
            &self.call_tool("executeLua", json!({"code": code}))?,
        ))
    }

    pub fn capture_screenshot(&mut self) -> Result<Option<Vec<u8>>> {
        Self::image_bytes(&self.call_tool("captureScreenshot", json!({}))?)
    }

    pub fn get_device_state(&mut self) -> Result<Value> {
        Self::json_result(&self.call_tool("getDeviceState", json!({}))?)
    }

    pub fn stream_logs(&self) -> Result<impl Iterator<Item = Result<String>>> {
        let response = self
            .client
            .get(self.url.replace("/mcp", "/logs/stream"))
            .send()?;
        Ok(SseLines::new(response))
    }
}

struct SseLines {
    reader: BufReader<Response>,
}

impl SseLines {
    fn new(response: Response) -> Self {
        Self {
            reader: BufReader::new(response),
        }
    }
}

impl Iterator for SseLines {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        loop {
            line.clear();
            match self.reader.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) => {
                    let trimmed = line.trim_end_matches(['\r', '\n']);
                    if let Some(data) = trimmed.strip_prefix("data: ") {
                        return Some(Ok(data.to_string()));
                    }
                }
                Err(error) => return Some(Err(error.into())),
            }
        }
    }
}

fn parse_response(response: Response) -> Result<Value> {
    let status = response.status();
    if status.as_u16() == 413 {
        bail!(
            "File too large: the Air Code server rejected the payload (413). Try reducing the file size."
        );
    }
    let response = response.error_for_status()?;
    Ok(response.json()?)
}

fn parse_string_array(value: &Value) -> Result<Vec<String>> {
    value
        .as_array()
        .ok_or_else(|| anyhow!("Expected array"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("Expected string item"))
        })
        .collect()
}

pub fn maybe_base64_text(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => general_purpose::STANDARD.encode(bytes),
    }
}
