// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.

use super::io::ChildStderrResource;
use super::io::ChildStdinResource;
use super::io::ChildStdoutResource;
use super::io::StdFileResource;
use crate::permissions::Permissions;
use deno_core::error::AnyError;
use deno_core::op;

use deno_core::serde_json;
use deno_core::AsyncMutFuture;
use deno_core::AsyncRefCell;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::RcRef;
use deno_core::Resource;
use deno_core::ResourceId;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::process::Command;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

pub fn init() -> Extension {
  Extension::builder()
    .ops(vec![op_run::decl(), op_run_status::decl(), op_kill::decl()])
    .build()
}

#[derive(Copy, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Stdio {
  Inherit,
  Piped,
  Null,
}

impl Stdio {
  pub fn as_stdio(&self) -> std::process::Stdio {
    match &self {
      Stdio::Inherit => std::process::Stdio::inherit(),
      Stdio::Piped => std::process::Stdio::piped(),
      Stdio::Null => std::process::Stdio::null(),
    }
  }
}

#[derive(Copy, Clone, PartialEq)]
pub enum StdioOrRid {
  Stdio(Stdio),
  Rid(ResourceId),
}

impl<'de> Deserialize<'de> for StdioOrRid {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    use serde_json::Value;
    let value = Value::deserialize(deserializer)?;
    match value {
      Value::String(val) => match val.as_str() {
        "inherit" => Ok(StdioOrRid::Stdio(Stdio::Inherit)),
        "piped" => Ok(StdioOrRid::Stdio(Stdio::Piped)),
        "null" => Ok(StdioOrRid::Stdio(Stdio::Null)),
        val => Err(serde::de::Error::unknown_variant(
          val,
          &["inherit", "piped", "null"],
        )),
      },
      Value::Number(val) => match val.as_u64() {
        Some(val) if val <= ResourceId::MAX as u64 => {
          Ok(StdioOrRid::Rid(val as ResourceId))
        }
        _ => Err(serde::de::Error::custom("Expected a positive integer")),
      },
      _ => Err(serde::de::Error::custom(
        r#"Expected a resource id, "inherit", "piped", or "null""#,
      )),
    }
  }
}

impl StdioOrRid {
  pub fn as_stdio(
    &self,
    state: &mut OpState,
  ) -> Result<std::process::Stdio, AnyError> {
    match &self {
      StdioOrRid::Stdio(val) => Ok(val.as_stdio()),
      StdioOrRid::Rid(rid) => StdFileResource::as_stdio(state, *rid),
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunArgs {
  cmd: Vec<String>,
  cwd: Option<String>,
  clear_env: bool,
  env: Vec<(String, String)>,
  #[cfg(unix)]
  gid: Option<u32>,
  #[cfg(unix)]
  uid: Option<u32>,
  stdin: StdioOrRid,
  stdout: StdioOrRid,
  stderr: StdioOrRid,
}

struct ChildResource {
  child: AsyncRefCell<tokio::process::Child>,
}

impl Resource for ChildResource {
  fn name(&self) -> Cow<str> {
    "child".into()
  }
}

impl ChildResource {
  fn borrow_mut(self: Rc<Self>) -> AsyncMutFuture<tokio::process::Child> {
    RcRef::map(self, |r| &r.child).borrow_mut()
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
// TODO(@AaronO): maybe find a more descriptive name or a convention for return structs
struct RunInfo {
  rid: ResourceId,
  pid: Option<u32>,
  stdin_rid: Option<ResourceId>,
  stdout_rid: Option<ResourceId>,
  stderr_rid: Option<ResourceId>,
}

#[op]
fn op_run(state: &mut OpState, run_args: RunArgs) -> Result<RunInfo, AnyError> {
  let args = run_args.cmd;
  state.borrow_mut::<Permissions>().run.check(&args[0])?;
  let env = run_args.env;
  let cwd = run_args.cwd;

  let mut c = Command::new(args.get(0).unwrap());
  (1..args.len()).for_each(|i| {
    let arg = args.get(i).unwrap();
    c.arg(arg);
  });
  cwd.map(|d| c.current_dir(d));

  if run_args.clear_env {
    super::check_unstable(state, "Deno.run.clearEnv");
    c.env_clear();
  }
  for (key, value) in &env {
    c.env(key, value);
  }

  #[cfg(unix)]
  if let Some(gid) = run_args.gid {
    super::check_unstable(state, "Deno.run.gid");
    c.gid(gid);
  }
  #[cfg(unix)]
  if let Some(uid) = run_args.uid {
    super::check_unstable(state, "Deno.run.uid");
    c.uid(uid);
  }
  #[cfg(unix)]
  // TODO(bartlomieju):
  #[allow(clippy::undocumented_unsafe_blocks)]
  unsafe {
    c.pre_exec(|| {
      libc::setgroups(0, std::ptr::null());
      Ok(())
    });
  }

  // TODO: make this work with other resources, eg. sockets
  c.stdin(run_args.stdin.as_stdio(state)?);
  c.stdout(
    match run_args.stdout {
      StdioOrRid::Stdio(Stdio::Inherit) => StdioOrRid::Rid(1),
      value => value,
    }
    .as_stdio(state)?,
  );
  c.stderr(
    match run_args.stderr {
      StdioOrRid::Stdio(Stdio::Inherit) => StdioOrRid::Rid(2),
      value => value,
    }
    .as_stdio(state)?,
  );

  // We want to kill child when it's closed
  c.kill_on_drop(true);

  // Spawn the command.
  let mut child = c.spawn()?;
  let pid = child.id();

  let stdin_rid = match child.stdin.take() {
    Some(child_stdin) => {
      let rid = state
        .resource_table
        .add(ChildStdinResource::from(child_stdin));
      Some(rid)
    }
    None => None,
  };

  let stdout_rid = match child.stdout.take() {
    Some(child_stdout) => {
      let rid = state
        .resource_table
        .add(ChildStdoutResource::from(child_stdout));
      Some(rid)
    }
    None => None,
  };

  let stderr_rid = match child.stderr.take() {
    Some(child_stderr) => {
      let rid = state
        .resource_table
        .add(ChildStderrResource::from(child_stderr));
      Some(rid)
    }
    None => None,
  };

  let child_resource = ChildResource {
    child: AsyncRefCell::new(child),
  };
  let child_rid = state.resource_table.add(child_resource);

  Ok(RunInfo {
    rid: child_rid,
    pid,
    stdin_rid,
    stdout_rid,
    stderr_rid,
  })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessStatus {
  got_signal: bool,
  exit_code: i32,
  exit_signal: i32,
}

#[op]
async fn op_run_status(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
) -> Result<ProcessStatus, AnyError> {
  let resource = state
    .borrow_mut()
    .resource_table
    .get::<ChildResource>(rid)?;
  let mut child = resource.borrow_mut().await;
  let run_status = child.wait().await?;
  let code = run_status.code();

  #[cfg(unix)]
  let signal = run_status.signal();
  #[cfg(not(unix))]
  let signal = None;

  code
    .or(signal)
    .expect("Should have either an exit code or a signal.");
  let got_signal = signal.is_some();

  Ok(ProcessStatus {
    got_signal,
    exit_code: code.unwrap_or(-1),
    exit_signal: signal.unwrap_or(-1),
  })
}

#[cfg(unix)]
pub fn kill(pid: i32, signal: &str) -> Result<(), AnyError> {
  let signo = super::signal::signal_str_to_int(signal)?;
  use nix::sys::signal::{kill as unix_kill, Signal};
  use nix::unistd::Pid;
  let sig = Signal::try_from(signo)?;
  unix_kill(Pid::from_raw(pid), Option::Some(sig)).map_err(AnyError::from)
}

#[cfg(not(unix))]
pub fn kill(pid: i32, signal: &str) -> Result<(), AnyError> {
  use deno_core::error::type_error;
  use std::io::Error;
  use std::io::ErrorKind::NotFound;
  use winapi::shared::minwindef::DWORD;
  use winapi::shared::minwindef::FALSE;
  use winapi::shared::minwindef::TRUE;
  use winapi::shared::winerror::ERROR_INVALID_PARAMETER;
  use winapi::um::errhandlingapi::GetLastError;
  use winapi::um::handleapi::CloseHandle;
  use winapi::um::processthreadsapi::OpenProcess;
  use winapi::um::processthreadsapi::TerminateProcess;
  use winapi::um::winnt::PROCESS_TERMINATE;

  if !matches!(signal, "SIGKILL" | "SIGTERM") {
    Err(type_error(format!("Invalid signal: {}", signal)))
  } else if pid <= 0 {
    Err(type_error("Invalid pid"))
  } else {
    // SAFETY: winapi call
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, FALSE, pid as DWORD) };

    if handle.is_null() {
      // SAFETY: winapi call
      let err = match unsafe { GetLastError() } {
        ERROR_INVALID_PARAMETER => Error::from(NotFound), // Invalid `pid`.
        errno => Error::from_raw_os_error(errno as i32),
      };
      Err(err.into())
    } else {
      // SAFETY: winapi calls
      unsafe {
        let is_terminated = TerminateProcess(handle, 1);
        CloseHandle(handle);
        match is_terminated {
          FALSE => Err(Error::last_os_error().into()),
          TRUE => Ok(()),
          _ => unreachable!(),
        }
      }
    }
  }
}

#[op]
fn op_kill(
  state: &mut OpState,
  pid: i32,
  signal: String,
) -> Result<(), AnyError> {
  state.borrow_mut::<Permissions>().run.check_all()?;
  kill(pid, &signal)?;
  Ok(())
}
