use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::tui::app::{App, TuiMessageKind};
use crate::tui::theme::SwissTheme;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // 1. Vertical Split: Header (3 lines) -> Main Body (flex) -> Input (3 lines) -> Footer (1 line)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Body (Stream + Sidebar)
            Constraint::Length(3), // Input bar
            Constraint::Length(1), // Footer status line
        ])
        .split(size);

    draw_header(f, app, main_chunks[0]);
    draw_body(f, app, main_chunks[1]);
    draw_input(f, app, main_chunks[2]);
    draw_footer(f, app, main_chunks[3]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // [+] APEX//CLI
            Constraint::Min(20),   // Model Router Badge
            Constraint::Length(24), // Tokens & Cost
            Constraint::Length(16), // Git Branch
        ])
        .split(area);

    // Brand Block
    let brand = Paragraph::new(Line::from(vec![
        Span::styled(" + ", SwissTheme::badge_red()),
        Span::styled(" APEX//CLI ", SwissTheme::title()),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(SwissTheme::BORDER_LINE)));
    f.render_widget(brand, header_chunks[0]);

    // Model Router Block
    let router_text = format!(" ROUTING: {} ", app.active_model);
    let router = Paragraph::new(Line::from(vec![
        Span::styled("● ", Style::default().fg(SwissTheme::EMERALD)),
        Span::styled(router_text, Style::default().fg(SwissTheme::STARK_WHITE).add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(SwissTheme::BORDER_LINE)));
    f.render_widget(router, header_chunks[1]);

    // Tokens & Cost Block
    let tokens_text = format!(" USAGE: {} / ${:.4} ", app.token_count, app.cost);
    let tokens = Paragraph::new(Line::from(Span::styled(tokens_text, Style::default().fg(SwissTheme::MUTED_TEXT))))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(SwissTheme::BORDER_LINE)));
    f.render_widget(tokens, header_chunks[2]);

    // Git Branch Block
    let git_text = format!(" GIT: {} ", app.git_branch);
    let git = Paragraph::new(Line::from(Span::styled(git_text, Style::default().fg(SwissTheme::STARK_WHITE).add_modifier(Modifier::BOLD))))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(SwissTheme::BORDER_LINE)));
    f.render_widget(git, header_chunks[3]);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    // 70% Execution Stream, 30% Telemetry Sidebar
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

    draw_stream(f, app, body_chunks[0]);
    draw_sidebar(f, app, body_chunks[1]);
}

fn draw_stream(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();

    for msg in &app.messages {
        match &msg.kind {
            TuiMessageKind::User => {
                lines.push(Line::from(vec![
                    Span::styled("[01] USER INTENT // ", Style::default().fg(SwissTheme::SWISS_RED).add_modifier(Modifier::BOLD)),
                    Span::styled(&msg.content, Style::default().fg(SwissTheme::STARK_WHITE).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(""));
            }
            TuiMessageKind::Assistant => {
                lines.push(Line::from(vec![
                    Span::styled("[02] REASONING // ", Style::default().fg(SwissTheme::MUTED_TEXT).add_modifier(Modifier::BOLD)),
                ]));
                for l in msg.content.lines() {
                    lines.push(Line::from(format!("  {}", l)));
                }
                lines.push(Line::from(""));
            }
            TuiMessageKind::ToolCall { name, args } => {
                lines.push(Line::from(vec![
                    Span::styled("▶ TOOL.EXEC: ", Style::default().fg(SwissTheme::AMBER).add_modifier(Modifier::BOLD)),
                    Span::styled(name, Style::default().fg(SwissTheme::STARK_WHITE).add_modifier(Modifier::BOLD)),
                    Span::raw(format!(" args={}", args)),
                ]));
            }
            TuiMessageKind::ToolResult { name, duration_ms } => {
                lines.push(Line::from(vec![
                    Span::styled("✔ COMPLETED: ", Style::default().fg(SwissTheme::EMERALD).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} ({}ms)", name, duration_ms), Style::default().fg(SwissTheme::MUTED_TEXT)),
                ]));
                for l in msg.content.lines().take(6) {
                    lines.push(Line::from(vec![
                        Span::styled("  | ", Style::default().fg(SwissTheme::BORDER_LINE)),
                        Span::styled(l, Style::default().fg(Color::Rgb(170, 175, 185))),
                    ]));
                }
                if msg.content.lines().count() > 6 {
                    lines.push(Line::from(Span::styled("  | ... [truncated]", Style::default().fg(SwissTheme::SUBTLE_TEXT))));
                }
                lines.push(Line::from(""));
            }
            TuiMessageKind::System => {
                lines.push(Line::from(Span::styled(&msg.content, Style::default().fg(SwissTheme::SUBTLE_TEXT))));
                lines.push(Line::from(""));
            }
            TuiMessageKind::Error => {
                lines.push(Line::from(vec![
                    Span::styled("✖ ERROR: ", Style::default().fg(SwissTheme::SWISS_RED).add_modifier(Modifier::BOLD)),
                    Span::styled(&msg.content, Style::default().fg(SwissTheme::SWISS_RED)),
                ]));
                lines.push(Line::from(""));
            }
        }
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" // EXECUTION STREAM ")
                .title_style(Style::default().fg(SwissTheme::MUTED_TEXT).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SwissTheme::BORDER_LINE)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll as u16, 0));

    f.render_widget(p, area);
}

fn draw_sidebar(f: &mut Frame, _app: &App, area: Rect) {
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Diagnostics Matrix
            Constraint::Length(8), // Token Density & Scope
            Constraint::Min(5),    // Workers / Instructions
        ])
        .split(area);

    // 1. Compiler / LSP Health
    let lsp_lines = vec![
        Line::from(vec![
            Span::styled("ERRORS:   ", Style::default().fg(SwissTheme::MUTED_TEXT)),
            Span::styled("00", Style::default().fg(SwissTheme::EMERALD).add_modifier(Modifier::BOLD)),
            Span::styled("   WARNINGS: ", Style::default().fg(SwissTheme::MUTED_TEXT)),
            Span::styled("01", Style::default().fg(SwissTheme::AMBER).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("SYMBOLS:  ", Style::default().fg(SwissTheme::MUTED_TEXT)),
            Span::styled("412", Style::default().fg(SwissTheme::STARK_WHITE)),
            Span::styled("  LSP RESP: ", Style::default().fg(SwissTheme::MUTED_TEXT)),
            Span::styled("12MS", Style::default().fg(SwissTheme::STARK_WHITE)),
        ]),
        Line::from(vec![
            Span::styled("STATUS:   ", Style::default().fg(SwissTheme::MUTED_TEXT)),
            Span::styled("RUST-ANALYZER READY", Style::default().fg(SwissTheme::EMERALD)),
        ]),
    ];

    let lsp_box = Paragraph::new(lsp_lines)
        .block(
            Block::default()
                .title(" // 01. COMPILER TELEMETRY ")
                .title_style(Style::default().fg(SwissTheme::MUTED_TEXT).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SwissTheme::BORDER_LINE)),
        );
    f.render_widget(lsp_box, sidebar_chunks[0]);

    // 2. Token Gauge & Scope
    let token_lines = vec![
        Line::from(vec![
            Span::styled("RATIO: ", Style::default().fg(SwissTheme::MUTED_TEXT)),
            Span::styled("■■□□□□□□□□□□□□□□□□□□", Style::default().fg(SwissTheme::STARK_WHITE)),
        ]),
        Line::from(vec![
            Span::styled("IN-SCOPE ASSETS: ", Style::default().fg(SwissTheme::MUTED_TEXT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled(" • Cargo.toml (180 tkn)", Style::default().fg(SwissTheme::STARK_WHITE))),
        Line::from(Span::styled(" • src/main.rs (420 tkn)", Style::default().fg(SwissTheme::STARK_WHITE))),
    ];

    let token_box = Paragraph::new(token_lines)
        .block(
            Block::default()
                .title(" // 02. CONTEXT ENGINE ")
                .title_style(Style::default().fg(SwissTheme::MUTED_TEXT).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SwissTheme::BORDER_LINE)),
        );
    f.render_widget(token_box, sidebar_chunks[1]);

    // 3. Command Keybindings
    let keymap_lines = vec![
        Line::from(vec![
            Span::styled("ENTER ", SwissTheme::badge_white()),
            Span::raw(" Send prompt to agent"),
        ]),
        Line::from(vec![
            Span::styled("TAB   ", SwissTheme::badge_white()),
            Span::raw(" Cycle sidebar view"),
        ]),
        Line::from(vec![
            Span::styled("UP/DN ", SwissTheme::badge_white()),
            Span::raw(" Scroll activity stream"),
        ]),
        Line::from(vec![
            Span::styled("ESC   ", SwissTheme::badge_red()),
            Span::raw(" Stop / Exit"),
        ]),
    ];

    let keymap_box = Paragraph::new(keymap_lines)
        .block(
            Block::default()
                .title(" // 03. KEYMAP ")
                .title_style(Style::default().fg(SwissTheme::MUTED_TEXT).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SwissTheme::BORDER_LINE)),
        );
    f.render_widget(keymap_box, sidebar_chunks[2]);
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let input_line = Line::from(vec![
        Span::styled("PROMPT> ", Style::default().fg(SwissTheme::SWISS_RED).add_modifier(Modifier::BOLD)),
        Span::styled(&app.input, Style::default().fg(SwissTheme::STARK_WHITE)),
        Span::styled("█", Style::default().fg(SwissTheme::SWISS_RED)),
    ]);

    let input_widget = Paragraph::new(input_line)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SwissTheme::BORDER_FOCUSED)),
        );
    f.render_widget(input_widget, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_text = format!(" APEX STATE: {} | DIR: {} ", app.status_text, app.workspace.display());
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(SwissTheme::SUBTLE_TEXT));
    f.render_widget(footer, area);
}
