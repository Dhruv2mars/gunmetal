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
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn run(paths: &AppPaths) -> Result<()> {
    let snapshot = DashboardSnapshot::load(paths)?;
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
    let sections = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(7),
        Constraint::Min(8),
    ])
    .split(frame.area());

    let hero = Paragraph::new(vec![
        Line::from(Span::styled(
            "Gunmetal",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("Local-first AI switchboard"),
        Line::from("Press q or Esc to quit"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Overview"));
    frame.render_widget(hero, sections[0]);

    let summary = Paragraph::new(vec![
        Line::from(format!("Home: {}", snapshot.root)),
        Line::from(format!(
            "Keys: {}   Profiles: {}   Models: {}   Logs: {}",
            snapshot.key_count, snapshot.profile_count, snapshot.model_count, snapshot.log_count
        )),
        Line::from(format!(
            "Priority: {}",
            snapshot.provider_priority.join(" -> ")
        )),
    ])
    .block(Block::default().borders(Borders::ALL).title("State"));
    frame.render_widget(summary, sections[1]);

    let details = Paragraph::new(
        snapshot
            .provider_breakdown
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect::<Vec<_>>(),
    )
    .block(Block::default().borders(Borders::ALL).title("Providers"));
    frame.render_widget(details, sections[2]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSnapshot {
    pub root: String,
    pub key_count: usize,
    pub profile_count: usize,
    pub model_count: usize,
    pub log_count: usize,
    pub provider_priority: Vec<String>,
    pub provider_breakdown: Vec<String>,
}

impl DashboardSnapshot {
    pub fn load(paths: &AppPaths) -> Result<Self> {
        let storage = paths.storage_handle()?;
        let provider_catalog = builtin_providers();

        Ok(Self {
            root: paths.root.display().to_string(),
            key_count: storage.list_keys()?.len(),
            profile_count: storage.list_profiles()?.len(),
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
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::DashboardSnapshot;

    #[test]
    fn snapshot_reads_counts_from_local_state() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let snapshot = DashboardSnapshot::load(&paths).unwrap();

        assert_eq!(snapshot.key_count, 0);
        assert_eq!(snapshot.profile_count, 0);
        assert_eq!(snapshot.model_count, 0);
        assert!(!snapshot.provider_priority.is_empty());
    }
}
