use std::{
    fs,
    net::IpAddr,
    path::PathBuf,
    process::{Command as ProcessCommand, Stdio},
    thread,
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// Manages the lifecycle of the Gunmetal daemon process.
pub struct DaemonManager {
    home: PathBuf,
    pid_file: PathBuf,
    stdout_log: PathBuf,
    stderr_log: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub state: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub url: String,
    pub health: Option<String>,
    pub home: Option<String>,
    pub note: Option<String>,
}

impl DaemonManager {
    pub fn new(home: impl Into<PathBuf>) -> Result<Self> {
        let home = home.into();
        let runtime_dir = home.join("runtime");
        let logs_dir = home.join("logs");
        std::fs::create_dir_all(&runtime_dir)?;
        std::fs::create_dir_all(&logs_dir)?;
        Ok(Self {
            pid_file: runtime_dir.join("daemon.pid"),
            stdout_log: logs_dir.join("daemon.stdout.log"),
            stderr_log: logs_dir.join("daemon.stderr.log"),
            home,
        })
    }

    pub fn from_env() -> Result<Self> {
        if let Ok(home) = std::env::var("GUNMETAL_HOME") {
            return Self::new(home);
        }
        let Some(home) = dirs::home_dir() else {
            bail!("could not resolve user home directory");
        };
        Self::new(home.join(".gunmetal"))
    }

    pub async fn status(&self, host: IpAddr, port: u16) -> Result<ServiceStatus> {
        let url = format!("http://{host}:{port}");
        let health_url = format!("{url}/health");
        let pid = self.managed_pid()?;
        match reqwest::get(&health_url).await {
            Ok(response) => {
                let health = response.text().await.ok();
                let home = self.daemon_home(&url).await;
                Ok(ServiceStatus {
                    state: "running".to_owned(),
                    running: true,
                    pid,
                    url,
                    health,
                    home,
                    note: None,
                })
            }
            Err(_) => {
                if let Some(pid) = pid {
                    if self.process_exists(pid) {
                        return Ok(ServiceStatus {
                            state: "starting".to_owned(),
                            running: false,
                            pid: Some(pid),
                            url,
                            health: None,
                            home: None,
                            note: Some("Gunmetal is still starting.".to_owned()),
                        });
                    }
                    let _ = fs::remove_file(&self.pid_file);
                    return Ok(ServiceStatus {
                        state: "stopped".to_owned(),
                        running: false,
                        pid: None,
                        url,
                        health: None,
                        home: None,
                        note: Some("Removed stale daemon state.".to_owned()),
                    });
                }
                Ok(ServiceStatus {
                    state: "stopped".to_owned(),
                    running: false,
                    pid: None,
                    url,
                    health: None,
                    home: None,
                    note: None,
                })
            }
        }
    }

    pub async fn start(&self, host: IpAddr, port: u16) -> Result<ServiceStatus> {
        let current = self.status(host, port).await?;
        if current.running {
            self.ensure_home_matches(&current)?;
            return Ok(ServiceStatus {
                note: Some("Gunmetal was already running.".to_owned()),
                ..current
            });
        }
        if current.state == "starting" {
            return self.wait_for_health(host, port, 20).await;
        }

        self.spawn_process(host, port)?;
        let status = self.wait_for_health(host, port, 20).await?;
        self.ensure_home_matches(&status)?;
        if status.running {
            if let Ok(Some(pid)) = self.pid_from_port(port) {
                let _ = fs::write(&self.pid_file, pid.to_string());
            }
            return Ok(ServiceStatus {
                note: Some("Gunmetal started.".to_owned()),
                ..status
            });
        }

        anyhow::bail!("{}", self.diagnose_start_failure(port))
    }

    pub async fn stop(&self, host: IpAddr, port: u16) -> Result<ServiceStatus> {
        let pid = if let Some(pid) = self.managed_pid()? {
            Some(pid)
        } else {
            let status = self.status(host, port).await?;
            if status.running {
                self.pid_from_port(port)?
            } else {
                return Ok(ServiceStatus {
                    state: "stopped".to_owned(),
                    running: false,
                    pid: None,
                    url: status.url,
                    health: None,
                    home: None,
                    note: Some("Gunmetal was not running.".to_owned()),
                });
            }
        };

        let Some(pid) = pid else {
            return self.status(host, port).await;
        };

        self.terminate_pid(pid)?;
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(150));
            let status = self.status(host, port).await?;
            if !status.running {
                let _ = fs::remove_file(&self.pid_file);
                return Ok(ServiceStatus {
                    state: "stopped".to_owned(),
                    note: Some("Gunmetal stopped.".to_owned()),
                    ..status
                });
            }
        }

        let status = self.status(host, port).await?;
        Ok(ServiceStatus {
            state: "stopping".to_owned(),
            note: Some("Gunmetal is still shutting down. Run `gunmetal status` again.".to_owned()),
            ..status
        })
    }

    pub fn managed_pid(&self) -> Result<Option<u32>> {
        let Some(pid) = self.read_pid(&self.pid_file)? else {
            return Ok(None);
        };
        if self.process_exists(pid) {
            return Ok(Some(pid));
        }
        let _ = fs::remove_file(&self.pid_file);
        Ok(None)
    }

    pub fn diagnose_start_failure(&self, port: u16) -> String {
        let mut lines = vec!["Gunmetal failed to start.".to_owned()];

        if let Ok(stderr) = fs::read_to_string(&self.stderr_log) {
            if !stderr.is_empty() {
                lines.push("Recent daemon stderr:".to_owned());
                for line in stderr
                    .lines()
                    .rev()
                    .take(6)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    lines.push(format!("  {line}"));
                }
            }
        }

        match self.pid_from_port(port) {
            Ok(Some(pid)) => {
                lines.push(format!(
                    "Port {port} is already in use by process {pid}. Run `gunmetal stop --port {port}` or choose a different port."
                ));
            }
            Ok(None) => {
                lines.push(format!(
                    "Port {port} appears free. Check the daemon logs above for the real error."
                ));
            }
            Err(error) => {
                lines.push(format!("Could not inspect port {port}: {error}"));
            }
        }

        lines.join("\n")
    }

    fn spawn_process(&self, host: IpAddr, port: u16) -> Result<()> {
        use std::fs::OpenOptions;

        let stdout = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.stdout_log)?;
        let stderr = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.stderr_log)?;
        let mut command = ProcessCommand::new(std::env::current_exe()?);
        #[cfg(unix)]
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        #[cfg(windows)]
        command.creation_flags(0x00000008);
        command
            .arg("serve")
            .arg("--host")
            .arg(host.to_string())
            .arg("--port")
            .arg(port.to_string())
            .env("GUNMETAL_HOME", &self.home)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        command.spawn()?;
        Ok(())
    }

    async fn wait_for_health(
        &self,
        host: IpAddr,
        port: u16,
        attempts: usize,
    ) -> Result<ServiceStatus> {
        for _ in 0..attempts {
            let status = self.status(host, port).await?;
            if status.running {
                return Ok(status);
            }
            thread::sleep(Duration::from_millis(150));
        }
        self.status(host, port).await
    }

    fn read_pid(&self, path: &std::path::Path) -> Result<Option<u32>> {
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(path)?;
        Ok(raw.trim().parse::<u32>().ok())
    }

    fn process_exists(&self, pid: u32) -> bool {
        #[cfg(windows)]
        {
            return ProcessCommand::new("tasklist")
                .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
                .output()
                .ok()
                .map(|output| {
                    let text = String::from_utf8_lossy(&output.stdout);
                    text.contains(&format!(",\"{pid}\"")) || text.starts_with('"')
                })
                .unwrap_or(false);
        }

        #[cfg(unix)]
        {
            unsafe {
                let result = libc::kill(pid as i32, 0);
                if result == 0 {
                    return true;
                }
                std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
            }
        }
    }

    fn terminate_pid(&self, pid: u32) -> Result<()> {
        #[cfg(windows)]
        {
            let status = ProcessCommand::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .status()?;
            if !status.success() {
                anyhow::bail!("failed to stop daemon pid {pid}");
            }
        }

        #[cfg(not(windows))]
        {
            let status = ProcessCommand::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status()?;
            if !status.success() {
                anyhow::bail!("failed to stop daemon pid {pid}");
            }
        }

        Ok(())
    }

    pub fn pid_from_port(&self, port: u16) -> Result<Option<u32>> {
        #[cfg(unix)]
        {
            if let Ok(output) = ProcessCommand::new("lsof")
                .args(["-iTCP", &format!(":{port}"), "-sTCP:LISTEN", "-t"])
                .output()
                && output.status.success()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if let Ok(pid) = line.trim().parse::<u32>() {
                        return Ok(Some(pid));
                    }
                }
            }
            if let Ok(output) = ProcessCommand::new("lsof")
                .args(["-ti", &format!(":{port}")])
                .output()
                && output.status.success()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if let Ok(pid) = line.trim().parse::<u32>() {
                        return Ok(Some(pid));
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            if let Ok(output) = ProcessCommand::new("netstat").args(["-ano"]).output() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.contains(&format!(":{port}")) && line.contains("LISTENING") {
                        if let Some(pid_str) = line.split_whitespace().last()
                            && let Ok(pid) = pid_str.parse::<u32>()
                        {
                            return Ok(Some(pid));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn daemon_home(&self, url: &str) -> Option<String> {
        let response = reqwest::get(format!("{url}/webui/api/state")).await.ok()?;
        let body = response.json::<serde_json::Value>().await.ok()?;
        body.get("service")
            .and_then(|service| service.get("home"))
            .and_then(|home| home.as_str())
            .map(ToOwned::to_owned)
    }

    fn ensure_home_matches(&self, status: &ServiceStatus) -> Result<()> {
        let expected = self.home.display().to_string();
        if let Some(home) = status.home.as_deref()
            && home != expected
        {
            let port = status
                .url
                .rsplit(':')
                .next()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8080);
            anyhow::bail!(
                "Port {port} is already used by Gunmetal with home {home}.\nRun `gunmetal stop --port {port}` to stop it, or use `gunmetal start --port <different-port>`.",
            );
        }
        Ok(())
    }
}
