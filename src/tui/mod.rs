pub mod theme;
pub mod app;
pub mod ui;

use std::io::stdout;
use std::path::Path;
use std::time::Duration;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;
use crate::agent::engine::{AgentEngine, AgentEvent};
use crate::config::ApexConfig;
use self::app::{App, TuiMessage, TuiMessageKind};

pub async fn run_tui(workspace: impl AsRef<Path>, config: ApexConfig) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&workspace, config.clone());

    // Main TUI Event Loop
    let res = run_loop(&mut terminal, &mut app, workspace.as_ref(), config).await;

    // Restore terminal safely
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    workspace: &Path,
    config: ApexConfig,
) -> Result<()> {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let (prompt_tx, mut prompt_rx) = mpsc::unbounded_channel::<String>();

    // Background Agent Task
    let ws = workspace.to_path_buf();
    tokio::spawn(async move {
        let mut engine = match AgentEngine::new(&ws, config) {
            Ok(e) => e,
            Err(err) => {
                let _ = event_tx.send(AgentEvent::Error(err.to_string()));
                return;
            }
        };

        while let Some(prompt) = prompt_rx.recv().await {
            let tx = event_tx.clone();
            let _ = engine.run(&prompt, move |ev| {
                let _ = tx.send(ev);
            }).await;
        }
    });

    loop {
        // Drain any pending agent events
        while let Ok(event) = event_rx.try_recv() {
            app.append_event(event);
            app.is_running = false;
            app.status_text = "NORMAL // READY".to_string();
        }

        terminal.draw(|f| ui::draw(f, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if key.code == KeyCode::Enter && !app.input.trim().is_empty() && !app.is_running {
                    let prompt = app.input.trim().to_string();
                    app.input.clear();
                    app.cursor_pos = 0;
                    app.prompt_history.push(prompt.clone());
                    app.history_idx = None;

                    // Intercept slash commands immediately
                    if prompt.starts_with('/') {
                        if app.handle_slash_command(&prompt).await {
                            continue;
                        }
                    }

                    app.is_running = true;
                    app.status_text = "AGENT RUNNING // THINKING".to_string();

                    app.messages.push(TuiMessage {
                        kind: TuiMessageKind::User,
                        content: prompt.clone(),
                    });

                    let _ = prompt_tx.send(prompt);
                } else {
                    app.handle_key(key);
                }
            }
        }
    }

    Ok(())
}
