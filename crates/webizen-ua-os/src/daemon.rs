use crate::session::{LoopbackHost, Sku};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DaemonError {
    Io(String),
    NotFound(String),
    RefusedShellBinary,
    Health(String),
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonError::Io(s) | DaemonError::NotFound(s) | DaemonError::Health(s) => {
                f.write_str(s)
            }
            DaemonError::RefusedShellBinary => {
                f.write_str("refused to exec the OS-shell as the Qualia daemon")
            }
        }
    }
}

impl std::error::Error for DaemonError {}

#[derive(Clone, Debug)]
pub struct DaemonHandle {
    pub status: DaemonStatus,
}

#[derive(Clone, Debug)]
pub struct DaemonStatus {
    pub reachable: bool,
    pub port: u16,
    pub binary: Option<PathBuf>,
    pub detail: String,
}

pub fn ping_daemon(port: u16) -> Result<String, DaemonError> {
    let addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .map_err(|e| DaemonError::Health(format!("parse: {e}")))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))
        .map_err(|e| DaemonError::Health(format!("connect: {e}")))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| DaemonError::Io(e.to_string()))?;
    let request =
        format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| DaemonError::Health(format!("write: {e}")))?;
    let mut body = String::new();
    stream
        .read_to_string(&mut body)
        .map_err(|e| DaemonError::Health(format!("read: {e}")))?;
    if body.contains('{') || body.contains("OK") || body.contains("200") {
        Ok(body)
    } else {
        Err(DaemonError::Health("empty /health".into()))
    }
}

pub fn ensure_daemon(host: &LoopbackHost, sku: Sku) -> Result<DaemonHandle, DaemonError> {
    host.export_vault_env();
    host.ensure_runtime_dir()?;

    if ping_daemon(host.port).is_ok() {
        return Ok(DaemonHandle {
            status: DaemonStatus {
                reachable: true,
                port: host.port,
                binary: resolve_qualia_cli(host).ok(),
                detail: "already reachable".into(),
            },
        });
    }

    let binary = resolve_qualia_cli(host)?;
    refuse_if_shell(&binary)?;

    let mut cmd = Command::new(&binary);
    cmd.arg("daemon").arg("start");
    if sku == Sku::Guest {
        cmd.arg("--dev");
    }
    cmd.arg("--port").arg(host.port.to_string());
    cmd.env("QUALIA_STORAGE_PATH", &host.storage);
    cmd.env("QUALIA_DATA_DIR", &host.storage);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    let status = cmd
        .status()
        .map_err(|e| DaemonError::Io(format!("spawn {}: {e}", binary.display())))?;
    if !status.success() {
        return Err(DaemonError::Io(format!(
            "{} daemon start exited {}",
            binary.display(),
            status
        )));
    }

    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(150));
        if ping_daemon(host.port).is_ok() {
            return Ok(DaemonHandle {
                status: DaemonStatus {
                    reachable: true,
                    port: host.port,
                    binary: Some(binary),
                    detail: "started".into(),
                },
            });
        }
    }

    Ok(DaemonHandle {
        status: DaemonStatus {
            reachable: false,
            port: host.port,
            binary: Some(binary),
            detail: format!(
                "started; waiting on GET /health and {}",
                host.pid_path().display()
            ),
        },
    })
}

pub fn probe_only(host: &LoopbackHost) -> DaemonStatus {
    let binary = resolve_qualia_cli(host).ok();
    match ping_daemon(host.port) {
        Ok(_) => DaemonStatus {
            reachable: true,
            port: host.port,
            binary,
            detail: "reachable".into(),
        },
        Err(e) => DaemonStatus {
            reachable: false,
            port: host.port,
            binary,
            detail: e.to_string(),
        },
    }
}

/// Result returned by `POST http://127.0.0.1:4242/invoke` on the QualiaDB sidecar.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpInvokeResult {
    pub ok: bool,
    pub value: String,
    pub diagnostic: Option<String>,
    pub revision: u64,
    pub committed: usize,
    pub honesty: String,
}

/// Invoke a QualiaDB capability (`GraphDatabase.*`, `WebizenSentinel.*`, `Wellfair.*`, `Llm.*`)
/// over loopback HTTP `POST /invoke` on `127.0.0.1:{port}`.
pub fn http_invoke(port: u16, id: &str, args: JsonValue) -> Result<HttpInvokeResult, DaemonError> {
    let addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .map_err(|e| DaemonError::Health(format!("parse: {e}")))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
        .map_err(|e| DaemonError::Health(format!("connect: {e}")))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(6)))
        .map_err(|e| DaemonError::Io(e.to_string()))?;

    let payload = serde_json::json!({ "id": id, "args": args }).to_string();
    let req = format!(
        "POST /invoke HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| DaemonError::Io(format!("write /invoke: {e}")))?;

    let mut raw = String::new();
    stream
        .read_to_string(&mut raw)
        .map_err(|e| DaemonError::Io(format!("read /invoke: {e}")))?;

    let json_start = raw
        .find("\r\n\r\n")
        .map(|i| &raw[i + 4..])
        .and_then(|b| b.find('{').map(|j| &b[j..]))
        .ok_or_else(|| DaemonError::Health("invalid /invoke HTTP response".into()))?;

    let v: JsonValue = serde_json::from_str(json_start)
        .map_err(|e| DaemonError::Health(format!("parse /invoke JSON: {e}")))?;

    let value = match v.get("value") {
        Some(JsonValue::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    };
    Ok(HttpInvokeResult {
        ok: v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false),
        value,
        diagnostic: v
            .get("diagnostic")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        revision: v.get("revision").and_then(|x| x.as_u64()).unwrap_or(0),
        committed: v.get("committed").and_then(|x| x.as_u64()).unwrap_or(0) as usize,
        honesty: v
            .get("honesty")
            .and_then(|x| x.as_str())
            .unwrap_or("live")
            .to_string(),
    })
}

fn resolve_qualia_cli(host: &LoopbackHost) -> Result<PathBuf, DaemonError> {
    if let Ok(p) = std::env::var("WEBIZEN_QUALIA_CLI") {
        let path = PathBuf::from(p.trim());
        if path.is_file() {
            return Ok(path);
        }
        return Err(DaemonError::NotFound(format!(
            "WEBIZEN_QUALIA_CLI not a file: {}",
            path.display()
        )));
    }
    let names: &[&str] = if cfg!(windows) {
        &["qualia-cli.exe", "qualia-cli"]
    } else {
        &["qualia-cli"]
    };
    for name in names {
        let candidate = host.install_root.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    for extra in sibling_qualia_cli_candidates() {
        if extra.is_file() {
            return Ok(extra);
        }
    }
    which("qualia-cli").or_else(|_| {
        Err(DaemonError::NotFound(
            "qualia-cli not in install_root or PATH (set WEBIZEN_QUALIA_CLI)".into(),
        ))
    })
}

/// Guest-dev search: Qualia tree next to webizenai, isolated sidecar target first.
fn sibling_qualia_cli_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(root) = std::env::var("WEBIZEN_QUALIA_ROOT") {
        push_cli_under(&mut out, Path::new(&root));
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    // crates/webizen-ua-os → webizen-os-shell → webizenai → github
    if let Some(github) = manifest.ancestors().nth(4) {
        push_cli_under(&mut out, &github.join("qualiaDB"));
    }
    out
}

fn push_cli_under(out: &mut Vec<PathBuf>, root: &Path) {
    let names: &[&str] = if cfg!(windows) {
        &["qualia-cli.exe", "qualia-cli"]
    } else {
        &["qualia-cli"]
    };
    let dirs = [
        root.join("target-sidecar/debug"),
        root.join("target-sidecar/release"),
        root.join("target/debug"),
        root.join("target/release"),
    ];
    for dir in dirs {
        for name in names {
            out.push(dir.join(name));
        }
    }
}

fn refuse_if_shell(binary: &Path) -> Result<(), DaemonError> {
    let name = binary
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name.contains("webizen-os-shell") || name == "webizen-os-shell" {
        return Err(DaemonError::RefusedShellBinary);
    }
    Ok(())
}

fn which(cmd: &str) -> Result<PathBuf, DaemonError> {
    let exe = if cfg!(windows) && !cmd.ends_with(".exe") {
        format!("{cmd}.exe")
    } else {
        cmd.to_string()
    };
    let paths = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(&exe);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(DaemonError::NotFound(format!("{cmd} not on PATH")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_DAEMON_PORT;

    #[test]
    fn refuse_shell_binary() {
        let p = PathBuf::from("C:/github/webizen-os-shell.exe");
        assert!(matches!(
            refuse_if_shell(&p),
            Err(DaemonError::RefusedShellBinary)
        ));
        let ok = PathBuf::from("C:/opt/webizen/qualia-cli.exe");
        assert!(refuse_if_shell(&ok).is_ok());
    }

    #[test]
    fn default_port_matches_qualia() {
        assert_eq!(DEFAULT_DAEMON_PORT, 4242);
    }
}
