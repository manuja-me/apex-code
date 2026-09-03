#![allow(dead_code)]
use ratatui::style::{Color, Modifier, Style};

pub struct SwissTheme;

impl SwissTheme {
    pub const BG_BASE: Color = Color::Rgb(13, 14, 18);
    pub const BG_PANEL: Color = Color::Rgb(17, 19, 24);
    pub const BG_SUBTLE: Color = Color::Rgb(22, 24, 33);
    pub const BORDER_LINE: Color = Color::Rgb(43, 47, 56);
    pub const BORDER_FOCUSED: Color = Color::Rgb(235, 0, 41); // Swiss Red

    pub const SWISS_RED: Color = Color::Rgb(235, 0, 41);
    pub const STARK_WHITE: Color = Color::Rgb(245, 247, 250);
    pub const MUTED_TEXT: Color = Color::Rgb(120, 125, 138);
    pub const SUBTLE_TEXT: Color = Color::Rgb(80, 84, 96);
    pub const EMERALD: Color = Color::Rgb(52, 211, 153);
    pub const AMBER: Color = Color::Rgb(251, 191, 36);

    pub fn title() -> Style {
        Style::default()
            .fg(Self::STARK_WHITE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn badge_red() -> Style {
        Style::default()
            .bg(Self::SWISS_RED)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    pub fn badge_white() -> Style {
        Style::default()
            .bg(Self::STARK_WHITE)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_label() -> Style {
        Style::default()
            .fg(Self::MUTED_TEXT)
            .add_modifier(Modifier::BOLD)
    }

    pub fn code_diff_add() -> Style {
        Style::default()
            .fg(Self::EMERALD)
            .bg(Color::Rgb(9, 26, 19))
    }

    pub fn code_diff_remove() -> Style {
        Style::default()
            .fg(Color::Rgb(253, 164, 175))
            .bg(Color::Rgb(31, 9, 13))
    }
}
