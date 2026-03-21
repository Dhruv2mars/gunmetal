use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use gunmetal_providers::builtin_providers;
use gunmetal_storage::AppPaths;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn run(paths: &AppPaths, service: ServiceSnapshot) -> Result<()> {
    let snapshot = DashboardSnapshot::load(paths, service)?;
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, snapshot);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    snapshot: DashboardSnapshot,
) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, &snapshot))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            break;
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, snapshot: &DashboardSnapshot) {
    let rows = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(9),
        Constraint::Min(10),
    ])
    .split(frame.area());

    let overview = Paragraph::new(vec![
        Line::from(Span::styled(
            "Gunmetal",
            Style::default()
                .fg(Color::Rgb(226, 232, 240))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("Local-first AI switchboard"),
        Line::from("Press q or Esc to quit"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Overview"));
    frame.render_widget(overview, rows[0]);

    let service_color = if snapshot.service_running {
        Color::Rgb(134, 239, 172)
    } else {
        Color::Rgb(251, 191, 36)
    };
    let state = Paragraph::new(vec![
        Line::from(format!("Home: {}", snapshot.root)),
        Line::from(vec![
            Span::raw("Service: "),
            Span::styled(
                snapshot.service_state.clone(),
                Style::default().fg(service_color),
            ),
            Span::raw(format!("   URL: {}", snapshot.service_url)),
            Span::raw(match snapshot.service_pid {
                Some(pid) => format!("   PID: {pid}"),
                None => String::new(),
            }),
        ]),
        Line::from(format!(
            "Keys: {}   Profiles: {}   Models: {}   Logs: {}",
            snapshot.key_count, snapshot.profile_count, snapshot.model_count, snapshot.log_count
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title("State"));
    frame.render_widget(state, rows[1]);

    let top =
        Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)]).split(rows[2]);

    let actions = Paragraph::new(vec![
        Line::from(if snapshot.profile_count == 0 {
            "gunmetal setup"
        } else {
            "gunmetal start"
        }),
        Line::from("gunmetal stop"),
        Line::from("gunmetal status"),
        Line::from("gunmetal logs list --limit 20"),
        Line::from("gunmetal models sync <profile-name>"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Operator"));
    frame.render_widget(actions, top[0]);

    let recents = Paragraph::new(
        snapshot
            .recent_profiles
            .iter()
            .chain(snapshot.recent_keys.iter())
            .map(|line| Line::from(line.clone()))
            .collect::<Vec<_>>(),
    )
    .block(Block::default().borders(Borders::ALL).title("Recent"));
    frame.render_widget(recents, top[1]);

    let bottom =
        Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)]).split(rows[3]);

    let providers = Paragraph::new(
        snapshot
            .provider_breakdown
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect::<Vec<_>>(),
    )
    .block(Block::default().borders(Borders::ALL).title("Providers"));
    frame.render_widget(providers, bottom[0]);

    let logs = Paragraph::new(
        snapshot
            .recent_logs
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect::<Vec<_>>(),
    )
    .block(Block::default().borders(Borders::ALL).title("Request Logs"));
    frame.render_widget(logs, bottom[1]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSnapshot {
    pub root: String,
    pub service_state: String,
    pub service_running: bool,
    pub service_url: String,
    pub service_pid: Option<u32>,
    pub key_count: usize,
    pub profile_count: usize,
    pub model_count: usize,
    pub log_count: usize,
    pub provider_priority: Vec<String>,
    pub provider_breakdown: Vec<String>,
    pub recent_profiles: Vec<String>,
    pub recent_keys: Vec<String>,
    pub recent_logs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSnapshot {
    pub state: String,
    pub running: bool,
    pub url: String,
    pub pid: Option<u32>,
}

impl DashboardSnapshot {
    pub fn load(paths: &AppPaths, service: ServiceSnapshot) -> Result<Self> {
        let storage = paths.storage_handle()?;
        let provider_catalog = builtin_providers();
        let profiles = storage.list_profiles()?;
        let keys = storage.list_keys()?;
        let logs = storage.list_request_logs(6)?;

        Ok(Self {
            root: paths.root.display().to_string(),
            service_state: service.state,
            service_running: service.running,
            service_url: service.url,
            service_pid: service.pid,
            key_count: keys.len(),
            profile_count: profiles.len(),
            model_count: storage.list_models()?.len(),
            log_count: storage.list_request_logs(250)?.len(),
            provider_priority: provider_catalog
                .iter()
                .map(|item| item.kind.to_string())
                .collect(),
            provider_breakdown: provider_catalog
                .iter()
                .map(|item| format!("{} {:?} priority={}", item.kind, item.class, item.priority))
                .collect(),
            recent_profiles: profiles
                .iter()
                .take(3)
                .map(|item| {
                    format!(
                        "profile {} {} {}",
                        item.provider,
                        item.name,
                        item.base_url.as_deref().unwrap_or("default")
                    )
                })
                .chain(
                    profiles
                        .is_empty()
                        .then(|| "run `gunmetal setup` to connect your first provider".to_owned()),
                )
                .collect(),
            recent_keys: keys
                .iter()
                .take(3)
                .map(|item| format!("key {} {} {}", item.prefix, item.name, item.state))
                .chain(
                    keys.is_empty()
                        .then(|| "create a key in setup so apps can call Gunmetal".to_owned()),
                )
                .collect(),
            recent_logs: logs
                .iter()
                .take(6)
                .map(|item| {
                    format!(
                        "{} {} {} {} {}",
                        item.provider,
                        item.model,
                        item.endpoint,
                        item.status_code.unwrap_or_default(),
                        item.usage.total_tokens.unwrap_or_default()
                    )
                })
                .chain(
                    logs.is_empty().then(|| {
                        "request history shows up here after the first API call".to_owned()
                    }),
                )
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{DashboardSnapshot, ServiceSnapshot};

    #[test]
    fn snapshot_reads_counts_from_local_state() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let snapshot = DashboardSnapshot::load(
            &paths,
            ServiceSnapshot {
                state: "stopped".to_owned(),
                running: false,
                url: "http://127.0.0.1:4684".to_owned(),
                pid: None,
            },
        )
        .unwrap();

        assert_eq!(snapshot.key_count, 0);
        assert_eq!(snapshot.profile_count, 0);
        assert_eq!(snapshot.model_count, 0);
        assert!(!snapshot.provider_priority.is_empty());
        assert_eq!(snapshot.service_state, "stopped");
    }
}
