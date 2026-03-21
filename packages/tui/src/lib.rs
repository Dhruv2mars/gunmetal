use std::io;

use anyhow::{Result, anyhow, bail};
use arboard::Clipboard;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use gunmetal_core::{KeyScope, NewGunmetalKey, NewProviderProfile, ProviderKind, ProviderProfile};
use gunmetal_providers::ProviderHub;
use gunmetal_storage::AppPaths;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};
use tokio::runtime::Runtime;

pub fn run(paths: &AppPaths, service: ServiceSnapshot) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut app = DashboardApp::load(paths, service)?;
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &runtime, &mut app, paths);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    runtime: &Runtime,
    app: &mut DashboardApp,
    paths: &AppPaths,
) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app))?;

        if let Event::Key(key) = event::read()?
            && app.handle_key(key, runtime, paths)?
        {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame, app: &DashboardApp) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(14),
            Constraint::Length(2),
        ])
        .split(area);

    let service_color = if app.service.running {
        Color::Green
    } else {
        Color::Yellow
    };
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            "Gunmetal",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "{}  {}  {}",
            app.service.state, app.service.url, app.snapshot.root
        )),
        Line::from(vec![
            Span::raw("Service "),
            Span::styled(
                app.service.state.clone(),
                Style::default().fg(service_color),
            ),
            Span::raw(format!(
                "   Profiles {}   Keys {}   Logs {}",
                app.snapshot.profiles.len(),
                app.snapshot.keys.len(),
                app.snapshot.logs.len()
            )),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Overview"));
    frame.render_widget(header, rows[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
        .split(rows[1]);

    let tabs = Tabs::new(Tab::titles())
        .select(app.tab.index())
        .block(Block::default().borders(Borders::ALL).title("Mode"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, body[0]);

    let left = inner_area(body[0], 1, 3);
    match app.tab {
        Tab::Profiles => render_profiles_list(frame, app, left),
        Tab::Keys => render_keys_list(frame, app, left),
        Tab::Logs => render_logs_list(frame, app, left),
        Tab::Snippets => render_snippets_list(frame, app, left),
    }

    let detail = Paragraph::new(app.detail_text())
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.tab.detail_title()),
        );
    frame.render_widget(detail, body[1]);

    let footer = Paragraph::new(vec![
        Line::from(app.hints()),
        Line::from(app.status_message.clone()),
    ])
    .block(Block::default().borders(Borders::ALL).title("Operator"));
    frame.render_widget(footer, rows[2]);

    if let Some(prompt) = &app.prompt {
        let area = centered_rect(64, 40, frame.area());
        frame.render_widget(Clear, area);
        let body = vec![
            Line::from(format!(
                "{} ({}/{})",
                prompt.current_label(),
                prompt.index + 1,
                prompt.fields.len()
            )),
            Line::from(""),
            Line::from(prompt.current_value()),
            Line::from(""),
            Line::from("Enter next field   Esc cancel   Backspace delete"),
        ];
        let prompt = Paragraph::new(body)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("New Profile"));
        frame.render_widget(prompt, area);
    }
}

fn render_profiles_list(frame: &mut Frame, app: &DashboardApp, area: Rect) {
    let items = if app.snapshot.profiles.is_empty() {
        vec![ListItem::new("No profiles. Press n to create one.")]
    } else {
        app.snapshot
            .profiles
            .iter()
            .map(|item| {
                ListItem::new(format!(
                    "{}  {}  models={}",
                    item.profile.provider, item.profile.name, item.model_count
                ))
            })
            .collect()
    };
    let mut state = ListState::default();
    state.select(app.selected_profile());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Profiles"))
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_keys_list(frame: &mut Frame, app: &DashboardApp, area: Rect) {
    let items = if app.snapshot.keys.is_empty() {
        vec![ListItem::new(
            "No keys. Press k on a profile to create one.",
        )]
    } else {
        app.snapshot
            .keys
            .iter()
            .map(|item| ListItem::new(format!("{}  {}  {}", item.prefix, item.name, item.state)))
            .collect()
    };
    let mut state = ListState::default();
    state.select(app.selected_key());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Keys"))
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_logs_list(frame: &mut Frame, app: &DashboardApp, area: Rect) {
    let items = if app.snapshot.logs.is_empty() {
        vec![ListItem::new(
            "No logs yet. Make an API call to populate this view.",
        )]
    } else {
        app.snapshot
            .logs
            .iter()
            .map(|item| {
                ListItem::new(format!(
                    "{}  {}  {}",
                    item.provider,
                    item.endpoint,
                    item.status_code.unwrap_or_default()
                ))
            })
            .collect()
    };
    let mut state = ListState::default();
    state.select(app.selected_log());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Requests"))
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_snippets_list(frame: &mut Frame, app: &DashboardApp, area: Rect) {
    let items = app
        .snippet_titles()
        .into_iter()
        .map(ListItem::new)
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(app.snippet_index));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Snippets"))
        .highlight_style(Style::default().bg(Color::DarkGray));
    frame.render_stateful_widget(list, area, &mut state);
}

#[derive(Debug, Clone)]
struct DashboardApp {
    snapshot: DashboardSnapshot,
    service: ServiceSnapshot,
    tab: Tab,
    profile_index: usize,
    key_index: usize,
    log_index: usize,
    snippet_index: usize,
    status_message: String,
    last_secret: Option<String>,
    prompt: Option<NewProfilePrompt>,
}

impl DashboardApp {
    fn load(paths: &AppPaths, service: ServiceSnapshot) -> Result<Self> {
        Ok(Self {
            snapshot: DashboardSnapshot::load(paths)?,
            service,
            tab: Tab::Profiles,
            profile_index: 0,
            key_index: 0,
            log_index: 0,
            snippet_index: 0,
            status_message: "Ready".to_owned(),
            last_secret: None,
            prompt: None,
        })
    }

    fn reload(&mut self, paths: &AppPaths) -> Result<()> {
        self.snapshot = DashboardSnapshot::load(paths)?;
        self.profile_index = clamp_index(self.profile_index, self.snapshot.profiles.len());
        self.key_index = clamp_index(self.key_index, self.snapshot.keys.len());
        self.log_index = clamp_index(self.log_index, self.snapshot.logs.len());
        self.snippet_index = clamp_index(self.snippet_index, self.snippet_titles().len());
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, runtime: &Runtime, paths: &AppPaths) -> Result<bool> {
        if self.prompt.is_some() {
            return self.handle_prompt_key(key, paths);
        }

        if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
            return Ok(true);
        }

        match key.code {
            KeyCode::Char('n') if self.tab == Tab::Profiles => {
                self.prompt = Some(NewProfilePrompt::default())
            }
            KeyCode::Char('a') if self.tab == Tab::Profiles => {
                self.auth_selected_profile(runtime, paths)?
            }
            KeyCode::Char('s') if self.tab == Tab::Profiles => {
                self.sync_selected_profile(runtime, paths)?
            }
            KeyCode::Char('o') if self.tab == Tab::Profiles => {
                self.logout_selected_profile(runtime, paths)?
            }
            KeyCode::Char('k') if self.tab == Tab::Profiles => {
                self.create_key_for_selected_profile(paths)?
            }
            KeyCode::Char('d') if self.tab == Tab::Keys => {
                self.set_selected_key_state(paths, gunmetal_core::KeyState::Disabled)?
            }
            KeyCode::Char('r') if self.tab == Tab::Keys => {
                self.set_selected_key_state(paths, gunmetal_core::KeyState::Revoked)?
            }
            KeyCode::Char('x') if self.tab == Tab::Keys => self.delete_selected_key(paths)?,
            KeyCode::Char('c')
                if self.tab == Tab::Snippets && key.modifiers == KeyModifiers::CONTROL =>
            {
                match copy_to_clipboard(&self.selected_snippet_text()) {
                    Ok(()) => {
                        self.status_message = "Copied snippet to clipboard.".to_owned();
                    }
                    Err(error) => {
                        self.status_message = format!("Clipboard unavailable: {error}");
                    }
                }
            }
            KeyCode::Tab | KeyCode::Right => self.tab = self.tab.next(),
            KeyCode::BackTab | KeyCode::Left => self.tab = self.tab.previous(),
            KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => self.select_previous(),
            KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => self.select_next(),
            _ => {}
        }

        Ok(false)
    }

    fn handle_prompt_key(&mut self, key: KeyEvent, paths: &AppPaths) -> Result<bool> {
        let Some(prompt) = &mut self.prompt else {
            return Ok(false);
        };

        match key.code {
            KeyCode::Esc => {
                self.prompt = None;
                self.status_message = "Cancelled profile creation.".to_owned();
            }
            KeyCode::Backspace => {
                prompt.current_value_mut().pop();
            }
            KeyCode::Enter => {
                if prompt.advance() {
                    self.create_profile_from_prompt(paths)?;
                }
            }
            KeyCode::Char(ch)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                prompt.current_value_mut().push(ch);
            }
            _ => {}
        }

        Ok(false)
    }

    fn create_profile_from_prompt(&mut self, paths: &AppPaths) -> Result<()> {
        let prompt = self
            .prompt
            .take()
            .ok_or_else(|| anyhow!("prompt missing"))?;
        let provider = prompt.provider()?;
        let name = prompt.name()?;
        let base_url = prompt.optional(2);
        let api_key = prompt.optional(3);
        let credentials = api_key.map(|value| serde_json::json!({ "api_key": value }));
        let profile = paths.storage_handle()?.create_profile(NewProviderProfile {
            provider,
            name: name.clone(),
            base_url,
            enabled: true,
            credentials,
        })?;
        self.reload(paths)?;
        self.profile_index = self
            .snapshot
            .profiles
            .iter()
            .position(|item| item.profile.id == profile.id)
            .unwrap_or_default();
        self.status_message = format!("Created profile {}", profile.name);
        Ok(())
    }

    fn auth_selected_profile(&mut self, runtime: &Runtime, paths: &AppPaths) -> Result<()> {
        let Some(profile) = self.selected_profile_row().map(|item| item.profile.clone()) else {
            self.status_message = "No profile selected.".to_owned();
            return Ok(());
        };
        let providers = ProviderHub::new(paths.clone());
        let message = runtime.block_on(async move {
            match profile.provider {
                ProviderKind::Codex | ProviderKind::Copilot => {
                    let session = providers.login(&profile, true).await?;
                    Ok::<String, anyhow::Error>(format!(
                        "Opened auth in browser: {}",
                        session.auth_url
                    ))
                }
                _ => {
                    let status = providers.auth_status(&profile).await?;
                    Ok(format!("Auth {:?}: {}", status.state, status.label))
                }
            }
        })?;
        self.reload(paths)?;
        self.status_message = message;
        Ok(())
    }

    fn sync_selected_profile(&mut self, runtime: &Runtime, paths: &AppPaths) -> Result<()> {
        let Some(profile) = self.selected_profile_row().map(|item| item.profile.clone()) else {
            self.status_message = "No profile selected.".to_owned();
            return Ok(());
        };
        let providers = ProviderHub::new(paths.clone());
        let storage = paths.storage_handle()?;
        let models = runtime.block_on(async { providers.sync_models(&profile).await })?;
        let count = models.len();
        storage.replace_models_for_profile(&profile.provider, Some(profile.id), &models)?;
        self.reload(paths)?;
        self.status_message = format!("Synced {count} models for {}", profile.name);
        Ok(())
    }

    fn logout_selected_profile(&mut self, runtime: &Runtime, paths: &AppPaths) -> Result<()> {
        let Some(profile) = self.selected_profile_row().map(|item| item.profile.clone()) else {
            self.status_message = "No profile selected.".to_owned();
            return Ok(());
        };
        let providers = ProviderHub::new(paths.clone());
        runtime.block_on(async { providers.logout(&profile).await })?;
        self.reload(paths)?;
        self.status_message = format!("Logged out {}", profile.name);
        Ok(())
    }

    fn create_key_for_selected_profile(&mut self, paths: &AppPaths) -> Result<()> {
        let Some(profile) = self.selected_profile_row().map(|item| item.profile.clone()) else {
            self.status_message = "No profile selected.".to_owned();
            return Ok(());
        };
        let created = paths.storage_handle()?.create_key(NewGunmetalKey {
            name: format!("{}-key", profile.name),
            scopes: vec![KeyScope::Inference, KeyScope::ModelsRead],
            allowed_providers: vec![profile.provider.clone()],
            expires_at: None,
        })?;
        self.last_secret = Some(created.secret);
        self.reload(paths)?;
        self.tab = Tab::Snippets;
        self.snippet_index = 0;
        self.status_message = format!("Created key {}", created.record.name);
        Ok(())
    }

    fn set_selected_key_state(
        &mut self,
        paths: &AppPaths,
        state: gunmetal_core::KeyState,
    ) -> Result<()> {
        let Some(key) = self.selected_key_row().cloned() else {
            self.status_message = "No key selected.".to_owned();
            return Ok(());
        };
        paths
            .storage_handle()?
            .set_key_state(key.id, state.clone())?;
        self.reload(paths)?;
        self.status_message = format!("Set {} to {}", key.name, state);
        Ok(())
    }

    fn delete_selected_key(&mut self, paths: &AppPaths) -> Result<()> {
        let Some(key) = self.selected_key_row().cloned() else {
            self.status_message = "No key selected.".to_owned();
            return Ok(());
        };
        paths.storage_handle()?.delete_key(key.id)?;
        self.reload(paths)?;
        self.status_message = format!("Deleted key {}", key.name);
        Ok(())
    }

    fn hints(&self) -> String {
        match self.tab {
            Tab::Profiles => {
                "Tab switch  n new profile  a auth/status  s sync  o logout  k create key  q quit"
                    .to_owned()
            }
            Tab::Keys => "Tab switch  d disable  r revoke  x delete  q quit".to_owned(),
            Tab::Logs => "Tab switch  up/down inspect request drill-down  q quit".to_owned(),
            Tab::Snippets => "Tab switch  Ctrl-C copy snippet  q quit".to_owned(),
        }
    }

    fn detail_text(&self) -> Text<'static> {
        match self.tab {
            Tab::Profiles => Text::from(self.profile_detail_lines()),
            Tab::Keys => Text::from(self.key_detail_lines()),
            Tab::Logs => Text::from(self.log_detail_lines()),
            Tab::Snippets => Text::from(self.snippet_detail_lines()),
        }
    }

    fn profile_detail_lines(&self) -> Vec<Line<'static>> {
        let Some(item) = self.selected_profile_row() else {
            return vec![
                Line::from("Create a profile with n."),
                Line::from("Then auth, sync, and create a key here."),
            ];
        };
        let models = self
            .snapshot
            .models
            .iter()
            .filter(|model| model.profile_id == Some(item.profile.id))
            .collect::<Vec<_>>();
        let first_model = models
            .first()
            .map(|item| item.id.as_str())
            .unwrap_or("<sync models>");
        vec![
            Line::from(format!("Provider: {}", item.profile.provider)),
            Line::from(format!("Name: {}", item.profile.name)),
            Line::from(format!(
                "Base URL: {}",
                item.profile.base_url.as_deref().unwrap_or("default")
            )),
            Line::from(format!("Models synced: {}", item.model_count)),
            Line::from(format!("First model: {first_model}")),
            Line::from(""),
            Line::from("Actions"),
            Line::from("a  browser auth for codex/copilot; auth check for key-based providers"),
            Line::from("s  sync models from upstream and enrich from models.dev"),
            Line::from("o  clear provider auth"),
            Line::from("k  create a provider-scoped Gunmetal key"),
        ]
    }

    fn key_detail_lines(&self) -> Vec<Line<'static>> {
        let Some(key) = self.selected_key_row() else {
            return vec![Line::from("Create a key from the Profiles tab with k.")];
        };
        let providers = if key.allowed_providers.is_empty() {
            "all providers".to_owned()
        } else {
            key.allowed_providers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        };
        let secret = self
            .last_secret
            .as_deref()
            .filter(|value| value.starts_with(&key.prefix))
            .unwrap_or("<secret hidden>");
        vec![
            Line::from(format!("Name: {}", key.name)),
            Line::from(format!("Prefix: {}", key.prefix)),
            Line::from(format!("State: {}", key.state)),
            Line::from(format!(
                "Scopes: {}",
                key.scopes
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Line::from(format!("Providers: {providers}")),
            Line::from(format!(
                "Last used: {}",
                key.last_used_at
                    .map(|value| value.to_rfc3339())
                    .unwrap_or_else(|| "never".to_owned())
            )),
            Line::from(""),
            Line::from("Config"),
            Line::from("Base URL: http://127.0.0.1:4684/v1"),
            Line::from(format!("API Key: {secret}")),
        ]
    }

    fn log_detail_lines(&self) -> Vec<Line<'static>> {
        let Some(log) = self.selected_log_row() else {
            return vec![Line::from(
                "Request drill-down appears here after the first API call.",
            )];
        };
        vec![
            Line::from(format!("When: {}", log.started_at.to_rfc3339())),
            Line::from(format!("Provider: {}", log.provider)),
            Line::from(format!("Model: {}", log.model)),
            Line::from(format!("Endpoint: {}", log.endpoint)),
            Line::from(format!(
                "Status: {}",
                log.status_code
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unknown".to_owned())
            )),
            Line::from(format!("Latency: {} ms", log.duration_ms)),
            Line::from(format!(
                "Usage: in={} out={} total={}",
                log.usage.input_tokens.unwrap_or_default(),
                log.usage.output_tokens.unwrap_or_default(),
                log.usage.total_tokens.unwrap_or_default()
            )),
            Line::from(format!(
                "Error: {}",
                log.error_message
                    .clone()
                    .unwrap_or_else(|| "none".to_owned())
            )),
        ]
    }

    fn snippet_titles(&self) -> Vec<String> {
        vec![
            "API Key".to_owned(),
            "Base URL".to_owned(),
            "cURL models".to_owned(),
            "cURL chat".to_owned(),
            "cURL responses".to_owned(),
        ]
    }

    fn snippet_detail_lines(&self) -> Vec<Line<'static>> {
        self.selected_snippet_text()
            .lines()
            .map(|line| Line::from(line.to_owned()))
            .collect()
    }

    fn selected_snippet_text(&self) -> String {
        let key = self
            .last_secret
            .clone()
            .unwrap_or_else(|| "<GUNMETAL_API_KEY>".to_owned());
        let model = self
            .selected_profile_row()
            .and_then(|item| {
                self.snapshot
                    .models
                    .iter()
                    .find(|model| model.profile_id == Some(item.profile.id))
                    .map(|model| model.id.clone())
            })
            .unwrap_or_else(|| "<provider/model>".to_owned());

        match self.snippet_index {
            0 => key,
            1 => "http://127.0.0.1:4684/v1".to_owned(),
            2 => format!("curl -H 'Authorization: Bearer {key}' http://127.0.0.1:4684/v1/models"),
            3 => format!(
                "curl -H 'Authorization: Bearer {key}' -H 'Content-Type: application/json' \\\n  http://127.0.0.1:4684/v1/chat/completions \\\n  -d '{{\"model\":\"{model}\",\"messages\":[{{\"role\":\"user\",\"content\":\"hello\"}}]}}'"
            ),
            _ => format!(
                "curl -H 'Authorization: Bearer {key}' -H 'Content-Type: application/json' \\\n  http://127.0.0.1:4684/v1/responses \\\n  -d '{{\"model\":\"{model}\",\"input\":\"hello\"}}'"
            ),
        }
    }

    fn selected_profile(&self) -> Option<usize> {
        (!self.snapshot.profiles.is_empty()).then_some(self.profile_index)
    }

    fn selected_profile_row(&self) -> Option<&ProfileRow> {
        self.snapshot.profiles.get(self.profile_index)
    }

    fn selected_key(&self) -> Option<usize> {
        (!self.snapshot.keys.is_empty()).then_some(self.key_index)
    }

    fn selected_key_row(&self) -> Option<&gunmetal_core::GunmetalKey> {
        self.snapshot.keys.get(self.key_index)
    }

    fn selected_log(&self) -> Option<usize> {
        (!self.snapshot.logs.is_empty()).then_some(self.log_index)
    }

    fn selected_log_row(&self) -> Option<&gunmetal_core::RequestLogEntry> {
        self.snapshot.logs.get(self.log_index)
    }

    fn select_previous(&mut self) {
        match self.tab {
            Tab::Profiles => self.profile_index = self.profile_index.saturating_sub(1),
            Tab::Keys => self.key_index = self.key_index.saturating_sub(1),
            Tab::Logs => self.log_index = self.log_index.saturating_sub(1),
            Tab::Snippets => self.snippet_index = self.snippet_index.saturating_sub(1),
        }
    }

    fn select_next(&mut self) {
        match self.tab {
            Tab::Profiles => {
                self.profile_index = next_index(self.profile_index, self.snapshot.profiles.len())
            }
            Tab::Keys => self.key_index = next_index(self.key_index, self.snapshot.keys.len()),
            Tab::Logs => self.log_index = next_index(self.log_index, self.snapshot.logs.len()),
            Tab::Snippets => {
                self.snippet_index = next_index(self.snippet_index, self.snippet_titles().len())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DashboardSnapshot {
    root: String,
    profiles: Vec<ProfileRow>,
    keys: Vec<gunmetal_core::GunmetalKey>,
    logs: Vec<gunmetal_core::RequestLogEntry>,
    models: Vec<gunmetal_core::ModelDescriptor>,
}

impl DashboardSnapshot {
    fn load(paths: &AppPaths) -> Result<Self> {
        let storage = paths.storage_handle()?;
        let profiles = storage.list_profiles()?;
        let models = storage.list_models()?;
        let profile_counts =
            models
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, model| {
                    if let Some(profile_id) = model.profile_id {
                        *acc.entry(profile_id).or_insert(0usize) += 1;
                    }
                    acc
                });

        Ok(Self {
            root: paths.root.display().to_string(),
            profiles: profiles
                .into_iter()
                .map(|profile| ProfileRow {
                    model_count: profile_counts.get(&profile.id).copied().unwrap_or_default(),
                    profile,
                })
                .collect(),
            keys: storage.list_keys()?,
            logs: storage.list_request_logs(50)?,
            models,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ProfileRow {
    profile: ProviderProfile,
    model_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSnapshot {
    pub state: String,
    pub running: bool,
    pub url: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Profiles,
    Keys,
    Logs,
    Snippets,
}

impl Tab {
    fn titles() -> Vec<Line<'static>> {
        vec![
            Line::from("Profiles"),
            Line::from("Keys"),
            Line::from("Logs"),
            Line::from("Snippets"),
        ]
    }

    fn index(self) -> usize {
        match self {
            Self::Profiles => 0,
            Self::Keys => 1,
            Self::Logs => 2,
            Self::Snippets => 3,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Profiles => Self::Keys,
            Self::Keys => Self::Logs,
            Self::Logs => Self::Snippets,
            Self::Snippets => Self::Profiles,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Profiles => Self::Snippets,
            Self::Keys => Self::Profiles,
            Self::Logs => Self::Keys,
            Self::Snippets => Self::Logs,
        }
    }

    fn detail_title(self) -> &'static str {
        match self {
            Self::Profiles => "Profile Detail",
            Self::Keys => "Key Detail",
            Self::Logs => "Request Drill-Down",
            Self::Snippets => "Copy-Ready Snippet",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NewProfilePrompt {
    index: usize,
    fields: Vec<PromptField>,
}

impl Default for NewProfilePrompt {
    fn default() -> Self {
        Self {
            index: 0,
            fields: vec![
                PromptField::new("Provider", "openai"),
                PromptField::new("Profile name", ""),
                PromptField::new("Base URL (optional)", ""),
                PromptField::new("API key (optional)", ""),
            ],
        }
    }
}

impl NewProfilePrompt {
    fn current_label(&self) -> &str {
        &self.fields[self.index].label
    }

    fn current_value(&self) -> &str {
        &self.fields[self.index].value
    }

    fn current_value_mut(&mut self) -> &mut String {
        &mut self.fields[self.index].value
    }

    fn advance(&mut self) -> bool {
        if self.index + 1 >= self.fields.len() {
            true
        } else {
            self.index += 1;
            false
        }
    }

    fn provider(&self) -> Result<ProviderKind> {
        self.fields[0]
            .value
            .trim()
            .parse::<ProviderKind>()
            .map_err(|error| anyhow!(error))
    }

    fn name(&self) -> Result<String> {
        let value = self.fields[1].value.trim();
        if value.is_empty() {
            bail!("profile name cannot be empty");
        }
        Ok(value.to_owned())
    }

    fn optional(&self, index: usize) -> Option<String> {
        let value = self.fields[index].value.trim();
        (!value.is_empty()).then(|| value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptField {
    label: String,
    value: String,
}

impl PromptField {
    fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_owned(),
            value: value.to_owned(),
        }
    }
}

fn next_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        usize::min(current + 1, len.saturating_sub(1))
    }
}

fn clamp_index(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        usize::min(current, len.saturating_sub(1))
    }
}

fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(popup[1])[1]
}

fn inner_area(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    Rect {
        x: area.x + horizontal,
        y: area.y + vertical,
        width: area.width.saturating_sub(horizontal * 2),
        height: area.height.saturating_sub(vertical + 1),
    }
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text.to_owned())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use gunmetal_core::{NewProviderProfile, ProviderKind};
    use tempfile::TempDir;
    use tokio::runtime::Runtime;

    use super::{DashboardApp, DashboardSnapshot, ServiceSnapshot};

    #[test]
    fn snapshot_reads_counts_from_local_state() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let snapshot = DashboardSnapshot::load(&paths).unwrap();

        assert!(snapshot.profiles.is_empty());
        assert!(snapshot.keys.is_empty());
        assert!(snapshot.logs.is_empty());
    }

    #[test]
    fn creating_provider_key_scopes_it_to_selected_profile() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let storage = paths.storage_handle().unwrap();
        storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::OpenAi,
                name: "openai".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();

        let mut app = DashboardApp::load(
            &paths,
            ServiceSnapshot {
                state: "running".to_owned(),
                running: true,
                url: "http://127.0.0.1:4684".to_owned(),
                pid: Some(1),
            },
        )
        .unwrap();
        app.create_key_for_selected_profile(&paths).unwrap();

        let keys = storage.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].allowed_providers, vec![ProviderKind::OpenAi]);
        assert!(
            app.last_secret
                .as_deref()
                .is_some_and(|value| value.starts_with("gm_"))
        );
    }

    #[test]
    fn snippets_show_last_created_secret_when_available() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let storage = paths.storage_handle().unwrap();
        let profile = storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::OpenAi,
                name: "openai".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();
        storage
            .replace_models_for_profile(
                &ProviderKind::OpenAi,
                Some(profile.id),
                &[gunmetal_core::ModelDescriptor {
                    id: "openai/gpt-5.1".to_owned(),
                    provider: ProviderKind::OpenAi,
                    profile_id: Some(profile.id),
                    upstream_name: "gpt-5.1".to_owned(),
                    display_name: "GPT-5.1".to_owned(),
                    metadata: None,
                }],
            )
            .unwrap();

        let mut app = DashboardApp::load(
            &paths,
            ServiceSnapshot {
                state: "running".to_owned(),
                running: true,
                url: "http://127.0.0.1:4684".to_owned(),
                pid: Some(1),
            },
        )
        .unwrap();
        app.last_secret = Some("gm_test_secret".to_owned());
        app.snippet_index = 3;
        let detail = app.snippet_detail_lines();
        let rendered = detail
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("gm_test_secret"));
        assert!(rendered.contains("openai/gpt-5.1"));
    }

    #[test]
    fn pressing_k_on_profiles_creates_key_instead_of_navigating() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let storage = paths.storage_handle().unwrap();
        storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::OpenAi,
                name: "one".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();
        storage
            .create_profile(NewProviderProfile {
                provider: ProviderKind::OpenRouter,
                name: "two".to_owned(),
                base_url: None,
                enabled: true,
                credentials: None,
            })
            .unwrap();

        let mut app = DashboardApp::load(
            &paths,
            ServiceSnapshot {
                state: "running".to_owned(),
                running: true,
                url: "http://127.0.0.1:4684".to_owned(),
                pid: Some(1),
            },
        )
        .unwrap();
        app.profile_index = app
            .snapshot
            .profiles
            .iter()
            .position(|row| row.profile.provider == ProviderKind::OpenRouter)
            .unwrap();

        let runtime = Runtime::new().unwrap();
        app.handle_key(
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
            &runtime,
            &paths,
        )
        .unwrap();

        let keys = storage.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].allowed_providers, vec![ProviderKind::OpenRouter]);
        assert_eq!(app.tab, super::Tab::Snippets);
    }
}
