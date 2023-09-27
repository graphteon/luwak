// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.

use crate::deno_exe_path;
use crate::new_deno_dir;
use crate::npm_registry_url;
use crate::PathRef;
use crate::TestContext;
use crate::TestContextBuilder;

use super::TempDir;

use anyhow::Result;
use lsp_types as lsp;
use lsp_types::ClientCapabilities;
use lsp_types::ClientInfo;
use lsp_types::CodeActionCapabilityResolveSupport;
use lsp_types::CodeActionClientCapabilities;
use lsp_types::CodeActionKindLiteralSupport;
use lsp_types::CodeActionLiteralSupport;
use lsp_types::CompletionClientCapabilities;
use lsp_types::CompletionItemCapability;
use lsp_types::FoldingRangeClientCapabilities;
use lsp_types::InitializeParams;
use lsp_types::TextDocumentClientCapabilities;
use lsp_types::TextDocumentSyncClientCapabilities;
use lsp_types::Url;
use lsp_types::WorkspaceClientCapabilities;
use once_cell::sync::Lazy;
use parking_lot::Condvar;
use parking_lot::Mutex;
use regex::Regex;
use serde::de;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::to_value;
use serde_json::Value;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

static CONTENT_TYPE_REG: Lazy<Regex> =
  lazy_regex::lazy_regex!(r"(?i)^content-length:\s+(\d+)");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LspResponseError {
  code: i32,
  message: String,
  data: Option<Value>,
}

#[derive(Clone, Debug)]
pub enum LspMessage {
  Notification(String, Option<Value>),
  Request(u64, String, Option<Value>),
  Response(u64, Option<Value>, Option<LspResponseError>),
}

impl<'a> From<&'a [u8]> for LspMessage {
  fn from(s: &'a [u8]) -> Self {
    let value: Value = serde_json::from_slice(s).unwrap();
    let obj = value.as_object().unwrap();
    if obj.contains_key("id") && obj.contains_key("method") {
      let id = obj.get("id").unwrap().as_u64().unwrap();
      let method = obj.get("method").unwrap().as_str().unwrap().to_string();
      Self::Request(id, method, obj.get("params").cloned())
    } else if obj.contains_key("id") {
      let id = obj.get("id").unwrap().as_u64().unwrap();
      let maybe_error: Option<LspResponseError> = obj
        .get("error")
        .map(|v| serde_json::from_value(v.clone()).unwrap());
      Self::Response(id, obj.get("result").cloned(), maybe_error)
    } else {
      assert!(obj.contains_key("method"));
      let method = obj.get("method").unwrap().as_str().unwrap().to_string();
      Self::Notification(method, obj.get("params").cloned())
    }
  }
}

#[derive(Debug, Deserialize)]
struct DiagnosticBatchNotificationParams {
  batch_index: usize,
  messages_len: usize,
}

fn read_message<R>(reader: &mut R) -> Result<Option<Vec<u8>>>
where
  R: io::Read + io::BufRead,
{
  let mut content_length = 0_usize;
  loop {
    let mut buf = String::new();
    if reader.read_line(&mut buf)? == 0 {
      return Ok(None);
    }
    if let Some(captures) = CONTENT_TYPE_REG.captures(&buf) {
      let content_length_match = captures
        .get(1)
        .ok_or_else(|| anyhow::anyhow!("missing capture"))?;
      content_length = content_length_match.as_str().parse::<usize>()?;
    }
    if &buf == "\r\n" {
      break;
    }
  }

  let mut msg_buf = vec![0_u8; content_length];
  reader.read_exact(&mut msg_buf)?;
  Ok(Some(msg_buf))
}

struct LspStdoutReader {
  pending_messages: Arc<(Mutex<Vec<LspMessage>>, Condvar)>,
  read_messages: Vec<LspMessage>,
}

impl LspStdoutReader {
  pub fn new(mut buf_reader: io::BufReader<ChildStdout>) -> Self {
    let messages: Arc<(Mutex<Vec<LspMessage>>, Condvar)> = Default::default();
    std::thread::spawn({
      let messages = messages.clone();
      move || {
        while let Ok(Some(msg_buf)) = read_message(&mut buf_reader) {
          let msg = LspMessage::from(msg_buf.as_slice());
          let cvar = &messages.1;
          {
            let mut messages = messages.0.lock();
            messages.push(msg);
          }
          cvar.notify_all();
        }
      }
    });

    LspStdoutReader {
      pending_messages: messages,
      read_messages: Vec::new(),
    }
  }

  pub fn pending_len(&self) -> usize {
    self.pending_messages.0.lock().len()
  }

  pub fn output_pending_messages(&self) {
    let messages = self.pending_messages.0.lock();
    eprintln!("{:?}", messages);
  }

  pub fn had_message(&self, is_match: impl Fn(&LspMessage) -> bool) -> bool {
    self.read_messages.iter().any(&is_match)
      || self.pending_messages.0.lock().iter().any(&is_match)
  }

  pub fn read_message<R>(
    &mut self,
    mut get_match: impl FnMut(&LspMessage) -> Option<R>,
  ) -> R {
    let (msg_queue, cvar) = &*self.pending_messages;
    let mut msg_queue = msg_queue.lock();
    loop {
      for i in 0..msg_queue.len() {
        let msg = &msg_queue[i];
        if let Some(result) = get_match(msg) {
          let msg = msg_queue.remove(i);
          self.read_messages.push(msg);
          return result;
        }
      }
      cvar.wait(&mut msg_queue);
    }
  }

  pub fn read_latest_message<R>(
    &mut self,
    mut get_match: impl FnMut(&LspMessage) -> Option<R>,
  ) -> R {
    let (msg_queue, cvar) = &*self.pending_messages;
    let mut msg_queue = msg_queue.lock();
    loop {
      for i in (0..msg_queue.len()).rev() {
        let msg = &msg_queue[i];
        if let Some(result) = get_match(msg) {
          let msg = msg_queue.remove(i);
          self.read_messages.push(msg);
          return result;
        }
      }
      cvar.wait(&mut msg_queue);
    }
  }
}

pub struct InitializeParamsBuilder {
  params: InitializeParams,
}

impl InitializeParamsBuilder {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self {
      params: InitializeParams {
        process_id: None,
        client_info: Some(ClientInfo {
          name: "test-harness".to_string(),
          version: Some("1.0.0".to_string()),
        }),
        root_uri: None,
        initialization_options: Some(json!({
          "enableBuiltinCommands": true,
          "enable": true,
          "cache": null,
          "certificateStores": null,
          "codeLens": {
            "implementations": true,
            "references": true,
            "test": true
          },
          "config": null,
          "importMap": null,
          "lint": true,
          "suggest": {
            "autoImports": true,
            "completeFunctionCalls": false,
            "names": true,
            "paths": true,
            "imports": {
              "hosts": {}
            }
          },
          "testing": {
            "args": [
              "--allow-all"
            ],
            "enable": true
          },
          "tlsCertificate": null,
          "unsafelyIgnoreCertificateErrors": null,
          "unstable": false
        })),
        capabilities: ClientCapabilities {
          text_document: Some(TextDocumentClientCapabilities {
            code_action: Some(CodeActionClientCapabilities {
              code_action_literal_support: Some(CodeActionLiteralSupport {
                code_action_kind: CodeActionKindLiteralSupport {
                  value_set: vec![
                    "quickfix".to_string(),
                    "refactor".to_string(),
                  ],
                },
              }),
              is_preferred_support: Some(true),
              data_support: Some(true),
              disabled_support: Some(true),
              resolve_support: Some(CodeActionCapabilityResolveSupport {
                properties: vec!["edit".to_string()],
              }),
              ..Default::default()
            }),
            completion: Some(CompletionClientCapabilities {
              completion_item: Some(CompletionItemCapability {
                snippet_support: Some(true),
                ..Default::default()
              }),
              ..Default::default()
            }),
            folding_range: Some(FoldingRangeClientCapabilities {
              line_folding_only: Some(true),
              ..Default::default()
            }),
            synchronization: Some(TextDocumentSyncClientCapabilities {
              dynamic_registration: Some(true),
              will_save: Some(true),
              will_save_wait_until: Some(true),
              did_save: Some(true),
            }),
            ..Default::default()
          }),
          workspace: Some(WorkspaceClientCapabilities {
            configuration: Some(true),
            workspace_folders: Some(true),
            ..Default::default()
          }),
          experimental: Some(json!({
            "testingApi": true
          })),
          ..Default::default()
        },
        ..Default::default()
      },
    }
  }

  pub fn set_maybe_root_uri(&mut self, value: Option<Url>) -> &mut Self {
    self.params.root_uri = value;
    self
  }

  pub fn set_root_uri(&mut self, value: Url) -> &mut Self {
    self.set_maybe_root_uri(Some(value))
  }

  pub fn set_workspace_folders(
    &mut self,
    folders: Vec<lsp_types::WorkspaceFolder>,
  ) -> &mut Self {
    self.params.workspace_folders = Some(folders);
    self
  }

  pub fn enable_inlay_hints(&mut self) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert(
      "inlayHints".to_string(),
      json!({
        "parameterNames": {
          "enabled": "all"
        },
        "parameterTypes": {
          "enabled": true
        },
        "variableTypes": {
          "enabled": true
        },
        "propertyDeclarationTypes": {
          "enabled": true
        },
        "functionLikeReturnTypes": {
          "enabled": true
        },
        "enumMemberValues": {
          "enabled": true
        }
      }),
    );
    self
  }

  pub fn disable_testing_api(&mut self) -> &mut Self {
    let obj = self
      .params
      .capabilities
      .experimental
      .as_mut()
      .unwrap()
      .as_object_mut()
      .unwrap();
    obj.insert("testingApi".to_string(), false.into());
    let options = self.initialization_options_mut();
    options.remove("testing");
    self
  }

  pub fn set_cache(&mut self, value: impl AsRef<str>) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert("cache".to_string(), value.as_ref().to_string().into());
    self
  }

  pub fn set_code_lens(
    &mut self,
    value: Option<serde_json::Value>,
  ) -> &mut Self {
    let options = self.initialization_options_mut();
    if let Some(value) = value {
      options.insert("codeLens".to_string(), value);
    } else {
      options.remove("codeLens");
    }
    self
  }

  pub fn set_config(&mut self, value: impl AsRef<str>) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert("config".to_string(), value.as_ref().to_string().into());
    self
  }

  pub fn set_disable_paths(&mut self, value: Vec<String>) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert("disablePaths".to_string(), value.into());
    self
  }

  pub fn set_enable_paths(&mut self, value: Vec<String>) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert("enablePaths".to_string(), value.into());
    self
  }

  pub fn set_deno_enable(&mut self, value: bool) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert("enable".to_string(), value.into());
    self
  }

  pub fn set_import_map(&mut self, value: impl AsRef<str>) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert("importMap".to_string(), value.as_ref().to_string().into());
    self
  }

  pub fn set_preload_limit(&mut self, arg: usize) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert("documentPreloadLimit".to_string(), arg.into());
    self
  }

  pub fn set_tls_certificate(&mut self, value: impl AsRef<str>) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert(
      "tlsCertificate".to_string(),
      value.as_ref().to_string().into(),
    );
    self
  }

  pub fn set_unstable(&mut self, value: bool) -> &mut Self {
    let options = self.initialization_options_mut();
    options.insert("unstable".to_string(), value.into());
    self
  }

  pub fn add_test_server_suggestions(&mut self) -> &mut Self {
    self.set_suggest_imports_hosts(vec![(
      "http://localhost:4545/".to_string(),
      true,
    )])
  }

  pub fn set_suggest_imports_hosts(
    &mut self,
    values: Vec<(String, bool)>,
  ) -> &mut Self {
    let options = self.initialization_options_mut();
    let suggest = options.get_mut("suggest").unwrap().as_object_mut().unwrap();
    let imports = suggest.get_mut("imports").unwrap().as_object_mut().unwrap();
    let hosts = imports.get_mut("hosts").unwrap().as_object_mut().unwrap();
    hosts.clear();
    for (key, value) in values {
      hosts.insert(key, value.into());
    }
    self
  }

  pub fn with_capabilities(
    &mut self,
    mut action: impl FnMut(&mut ClientCapabilities),
  ) -> &mut Self {
    action(&mut self.params.capabilities);
    self
  }

  fn initialization_options_mut(
    &mut self,
  ) -> &mut serde_json::Map<String, serde_json::Value> {
    let options = self.params.initialization_options.as_mut().unwrap();
    options.as_object_mut().unwrap()
  }

  pub fn build(&self) -> InitializeParams {
    self.params.clone()
  }
}

pub struct LspClientBuilder {
  print_stderr: bool,
  capture_stderr: bool,
  deno_exe: PathRef,
  context: Option<TestContext>,
  use_diagnostic_sync: bool,
}

impl LspClientBuilder {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self {
      print_stderr: false,
      capture_stderr: false,
      deno_exe: deno_exe_path(),
      context: None,
      use_diagnostic_sync: true,
    }
  }

  pub fn deno_exe(&mut self, exe_path: impl AsRef<Path>) -> &mut Self {
    self.deno_exe = PathRef::new(exe_path);
    self
  }

  // not deprecated, this is just here so you don't accidentally
  // commit code with this enabled
  #[deprecated]
  pub fn print_stderr(&mut self) -> &mut Self {
    self.print_stderr = true;
    self
  }

  pub fn capture_stderr(&mut self) -> &mut Self {
    self.capture_stderr = true;
    self
  }

  /// Whether to use the synchronization messages to better sync diagnostics
  /// between the test client and server.
  pub fn use_diagnostic_sync(&mut self, value: bool) -> &mut Self {
    self.use_diagnostic_sync = value;
    self
  }

  pub fn set_test_context(&mut self, test_context: &TestContext) -> &mut Self {
    self.context = Some(test_context.clone());
    self
  }

  pub fn build(&self) -> LspClient {
    self.build_result().unwrap()
  }

  pub fn build_result(&self) -> Result<LspClient> {
    let deno_dir = self
      .context
      .as_ref()
      .map(|c| c.deno_dir().clone())
      .unwrap_or_else(new_deno_dir);
    let mut command = Command::new(&self.deno_exe);
    command
      .env("DENO_DIR", deno_dir.path())
      .env("NPM_CONFIG_REGISTRY", npm_registry_url())
      // turn on diagnostic synchronization communication
      .env(
        "DENO_DONT_USE_INTERNAL_LSP_DIAGNOSTIC_SYNC_FLAG",
        if self.use_diagnostic_sync { "1" } else { "" },
      )
      .arg("lsp")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped());
    if self.capture_stderr {
      command.stderr(Stdio::piped());
    } else if !self.print_stderr {
      command.stderr(Stdio::null());
    }
    let mut child = command.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let buf_reader = io::BufReader::new(stdout);
    let reader = LspStdoutReader::new(buf_reader);

    let stdin = child.stdin.take().unwrap();
    let writer = io::BufWriter::new(stdin);

    let stderr_lines_rx = if self.capture_stderr {
      let stderr = child.stderr.take().unwrap();
      let print_stderr = self.print_stderr;
      let (tx, rx) = mpsc::channel::<String>();
      std::thread::spawn(move || {
        let stderr = BufReader::new(stderr);
        for line in stderr.lines() {
          match line {
            Ok(line) => {
              if print_stderr {
                eprintln!("{}", line);
              }
              tx.send(line).unwrap();
            }
            Err(err) => {
              panic!("failed to read line from stderr: {:#}", err);
            }
          }
        }
      });
      Some(rx)
    } else {
      None
    };

    Ok(LspClient {
      child,
      reader,
      request_id: 1,
      start: Instant::now(),
      context: self
        .context
        .clone()
        .unwrap_or_else(|| TestContextBuilder::new().build()),
      writer,
      deno_dir,
      stderr_lines_rx,
      config: json!("{}"),
      supports_workspace_configuration: false,
    })
  }
}

pub struct LspClient {
  child: Child,
  reader: LspStdoutReader,
  request_id: u64,
  start: Instant,
  writer: io::BufWriter<ChildStdin>,
  deno_dir: TempDir,
  context: TestContext,
  stderr_lines_rx: Option<mpsc::Receiver<String>>,
  config: serde_json::Value,
  supports_workspace_configuration: bool,
}

impl Drop for LspClient {
  fn drop(&mut self) {
    match self.child.try_wait() {
      Ok(None) => {
        self.child.kill().unwrap();
        let _ = self.child.wait();
      }
      Ok(Some(status)) => panic!("deno lsp exited unexpectedly {status}"),
      Err(e) => panic!("pebble error: {e}"),
    }
  }
}

impl LspClient {
  pub fn deno_dir(&self) -> &TempDir {
    &self.deno_dir
  }

  pub fn duration(&self) -> Duration {
    self.start.elapsed()
  }

  pub fn queue_is_empty(&self) -> bool {
    self.reader.pending_len() == 0
  }

  pub fn queue_len(&self) -> usize {
    self.reader.output_pending_messages();
    self.reader.pending_len()
  }

  #[track_caller]
  pub fn wait_until_stderr_line(&self, condition: impl Fn(&str) -> bool) {
    let timeout_time =
      Instant::now().checked_add(Duration::from_secs(5)).unwrap();
    let lines_rx = self
      .stderr_lines_rx
      .as_ref()
      .expect("must setup with client_builder.capture_stderr()");
    let mut found_lines = Vec::new();
    while Instant::now() < timeout_time {
      if let Ok(line) = lines_rx.try_recv() {
        if condition(&line) {
          return;
        }
        found_lines.push(line);
      }
      std::thread::sleep(Duration::from_millis(20));
    }

    eprintln!("==== STDERR OUTPUT ====");
    for line in found_lines {
      eprintln!("{}", line)
    }
    eprintln!("== END STDERR OUTPUT ==");

    panic!("Timed out waiting on condition.")
  }

  pub fn initialize_default(&mut self) {
    self.initialize(|_| {})
  }

  pub fn initialize(
    &mut self,
    do_build: impl Fn(&mut InitializeParamsBuilder),
  ) {
    self.initialize_with_config(
      do_build,
      json!({"deno":{
        "enable": true
      }}),
    )
  }

  pub fn initialize_with_config(
    &mut self,
    do_build: impl Fn(&mut InitializeParamsBuilder),
    config: Value,
  ) {
    let mut builder = InitializeParamsBuilder::new();
    builder.set_root_uri(self.context.temp_dir().uri());
    do_build(&mut builder);
    let params: InitializeParams = builder.build();
    self.supports_workspace_configuration = match &params.capabilities.workspace
    {
      Some(workspace) => workspace.configuration == Some(true),
      _ => false,
    };
    self.write_request("initialize", params);
    self.write_notification("initialized", json!({}));
    self.config = config;
    if self.supports_workspace_configuration {
      self.handle_configuration_request(&self.config.clone());
    }
  }

  pub fn did_open(&mut self, params: Value) -> CollectedDiagnostics {
    self.did_open_with_config(params, &self.config.clone())
  }

  pub fn did_open_with_config(
    &mut self,
    params: Value,
    config: &Value,
  ) -> CollectedDiagnostics {
    self.did_open_raw(params);
    if self.supports_workspace_configuration {
      self.handle_configuration_request(config);
    }
    self.read_diagnostics()
  }

  pub fn did_open_raw(&mut self, params: Value) {
    self.write_notification("textDocument/didOpen", params);
  }

  pub fn handle_configuration_request(&mut self, settings: &Value) {
    let (id, method, args) = self.read_request::<Value>();
    assert_eq!(method, "workspace/configuration");
    let params = args.as_ref().unwrap().as_object().unwrap();
    let items = params.get("items").unwrap().as_array().unwrap();
    let settings_object = settings.as_object().unwrap();
    let mut result = vec![];
    for item in items {
      let item = item.as_object().unwrap();
      let section = item.get("section").unwrap().as_str().unwrap();
      result.push(settings_object.get(section).cloned().unwrap_or_default());
    }
    self.write_response(id, result);
  }

  pub fn did_save(&mut self, params: Value) {
    self.write_notification("textDocument/didSave", params);
  }

  pub fn did_change_watched_files(&mut self, params: Value) {
    self.write_notification("workspace/didChangeWatchedFiles", params);
  }

  fn get_latest_diagnostic_batch_index(&mut self) -> usize {
    let result = self
      .write_request("deno/internalLatestDiagnosticBatchIndex", json!(null));
    result.as_u64().unwrap() as usize
  }

  /// Reads the latest diagnostics. It's assumed that
  pub fn read_diagnostics(&mut self) -> CollectedDiagnostics {
    // wait for three (deno, lint, and typescript diagnostics) batch
    // notification messages for that index
    let mut read = 0;
    let mut total_messages_len = 0;
    while read < 3 {
      let (method, response) =
        self.read_notification::<DiagnosticBatchNotificationParams>();
      assert_eq!(method, "deno/internalTestDiagnosticBatch");
      let response = response.unwrap();
      if response.batch_index == self.get_latest_diagnostic_batch_index() {
        read += 1;
        total_messages_len += response.messages_len;
      }
    }

    // now read the latest diagnostic messages
    let mut all_diagnostics = Vec::with_capacity(total_messages_len);
    let mut seen_files = HashSet::new();
    for _ in 0..total_messages_len {
      let (method, response) =
        self.read_latest_notification::<lsp::PublishDiagnosticsParams>();
      assert_eq!(method, "textDocument/publishDiagnostics");
      let response = response.unwrap();
      if seen_files.insert(response.uri.to_string()) {
        all_diagnostics.push(response);
      }
    }

    CollectedDiagnostics(all_diagnostics)
  }

  pub fn shutdown(&mut self) {
    self.write_request("shutdown", json!(null));
    self.write_notification("exit", json!(null));
  }

  // it's flaky to assert for a notification because a notification
  // might arrive a little later, so only provide a method for asserting
  // that there is no notification
  pub fn assert_no_notification(&mut self, searching_method: &str) {
    assert!(!self.reader.had_message(|message| match message {
      LspMessage::Notification(method, _) => method == searching_method,
      _ => false,
    }))
  }

  pub fn read_notification<R>(&mut self) -> (String, Option<R>)
  where
    R: de::DeserializeOwned,
  {
    self.reader.read_message(|msg| match msg {
      LspMessage::Notification(method, maybe_params) => {
        let params = serde_json::from_value(maybe_params.clone()?).ok()?;
        Some((method.to_string(), params))
      }
      _ => None,
    })
  }

  pub fn read_latest_notification<R>(&mut self) -> (String, Option<R>)
  where
    R: de::DeserializeOwned,
  {
    self.reader.read_latest_message(|msg| match msg {
      LspMessage::Notification(method, maybe_params) => {
        let params = serde_json::from_value(maybe_params.clone()?).ok()?;
        Some((method.to_string(), params))
      }
      _ => None,
    })
  }

  pub fn read_notification_with_method<R>(
    &mut self,
    expected_method: &str,
  ) -> Option<R>
  where
    R: de::DeserializeOwned,
  {
    self.reader.read_message(|msg| match msg {
      LspMessage::Notification(method, maybe_params) => {
        if method != expected_method {
          None
        } else {
          serde_json::from_value(maybe_params.clone()?).ok()
        }
      }
      _ => None,
    })
  }

  pub fn read_request<R>(&mut self) -> (u64, String, Option<R>)
  where
    R: de::DeserializeOwned,
  {
    self.reader.read_message(|msg| match msg {
      LspMessage::Request(id, method, maybe_params) => Some((
        *id,
        method.to_owned(),
        maybe_params
          .clone()
          .map(|p| serde_json::from_value(p).unwrap()),
      )),
      _ => None,
    })
  }

  fn write(&mut self, value: Value) {
    let value_str = value.to_string();
    let msg = format!(
      "Content-Length: {}\r\n\r\n{}",
      value_str.as_bytes().len(),
      value_str
    );
    self.writer.write_all(msg.as_bytes()).unwrap();
    self.writer.flush().unwrap();
  }

  pub fn get_completion(
    &mut self,
    uri: impl AsRef<str>,
    position: (usize, usize),
    context: Value,
  ) -> lsp::CompletionResponse {
    self.write_request_with_res_as::<lsp::CompletionResponse>(
      "textDocument/completion",
      json!({
        "textDocument": {
          "uri": uri.as_ref(),
        },
        "position": { "line": position.0, "character": position.1 },
        "context": context,
      }),
    )
  }

  pub fn get_completion_list(
    &mut self,
    uri: impl AsRef<str>,
    position: (usize, usize),
    context: Value,
  ) -> lsp::CompletionList {
    let res = self.get_completion(uri, position, context);
    if let lsp::CompletionResponse::List(list) = res {
      list
    } else {
      panic!("unexpected response");
    }
  }

  pub fn write_request_with_res_as<R>(
    &mut self,
    method: impl AsRef<str>,
    params: impl Serialize,
  ) -> R
  where
    R: de::DeserializeOwned,
  {
    let result = self.write_request(method, params);
    serde_json::from_value(result).unwrap()
  }

  pub fn write_request(
    &mut self,
    method: impl AsRef<str>,
    params: impl Serialize,
  ) -> Value {
    let value = if to_value(&params).unwrap().is_null() {
      json!({
        "jsonrpc": "2.0",
        "id": self.request_id,
        "method": method.as_ref(),
      })
    } else {
      json!({
        "jsonrpc": "2.0",
        "id": self.request_id,
        "method": method.as_ref(),
        "params": params,
      })
    };
    self.write(value);

    self.reader.read_message(|msg| match msg {
      LspMessage::Response(id, maybe_result, maybe_error) => {
        assert_eq!(*id, self.request_id);
        self.request_id += 1;
        if let Some(error) = maybe_error {
          panic!("LSP ERROR: {error:?}");
        }
        Some(maybe_result.clone().unwrap())
      }
      _ => None,
    })
  }

  pub fn write_response<V>(&mut self, id: u64, result: V)
  where
    V: Serialize,
  {
    let value = json!({
      "jsonrpc": "2.0",
      "id": id,
      "result": result
    });
    self.write(value);
  }

  pub fn write_notification<S, V>(&mut self, method: S, params: V)
  where
    S: AsRef<str>,
    V: Serialize,
  {
    let value = json!({
      "jsonrpc": "2.0",
      "method": method.as_ref(),
      "params": params,
    });
    self.write(value);
  }
}

#[derive(Debug, Clone)]
pub struct CollectedDiagnostics(Vec<lsp::PublishDiagnosticsParams>);

impl CollectedDiagnostics {
  /// Gets the diagnostics that the editor will see after all the publishes.
  pub fn all(&self) -> Vec<lsp::Diagnostic> {
    self
      .all_messages()
      .into_iter()
      .flat_map(|m| m.diagnostics)
      .collect()
  }

  /// Gets the messages that the editor will see after all the publishes.
  pub fn all_messages(&self) -> Vec<lsp::PublishDiagnosticsParams> {
    self.0.clone()
  }

  pub fn messages_with_source(
    &self,
    source: &str,
  ) -> lsp::PublishDiagnosticsParams {
    self
      .all_messages()
      .iter()
      .find(|p| {
        p.diagnostics
          .iter()
          .any(|d| d.source == Some(source.to_string()))
      })
      .map(ToOwned::to_owned)
      .unwrap()
  }

  #[track_caller]
  pub fn messages_with_file_and_source(
    &self,
    specifier: &str,
    source: &str,
  ) -> lsp::PublishDiagnosticsParams {
    let specifier = Url::parse(specifier).unwrap();
    self
      .all_messages()
      .iter()
      .find(|p| {
        p.uri == specifier
          && p
            .diagnostics
            .iter()
            .any(|d| d.source == Some(source.to_string()))
      })
      .map(ToOwned::to_owned)
      .unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_read_message() {
    let msg1 = b"content-length: 11\r\n\r\nhello world";
    let mut reader1 = std::io::Cursor::new(msg1);
    assert_eq!(read_message(&mut reader1).unwrap().unwrap(), b"hello world");

    let msg2 = b"content-length: 5\r\n\r\nhello world";
    let mut reader2 = std::io::Cursor::new(msg2);
    assert_eq!(read_message(&mut reader2).unwrap().unwrap(), b"hello");
  }

  #[test]
  #[should_panic(expected = "failed to fill whole buffer")]
  fn test_invalid_read_message() {
    let msg1 = b"content-length: 12\r\n\r\nhello world";
    let mut reader1 = std::io::Cursor::new(msg1);
    read_message(&mut reader1).unwrap();
  }
}
