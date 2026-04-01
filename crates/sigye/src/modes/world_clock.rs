//! World Clock display mode.

use std::any::Any;
use std::str::FromStr;

use chrono::Utc;
use chrono_tz::Tz;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Color,
};
use sigye_core::{DisplayMode, TimeFormat};

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// World Clock display mode — shows multiple timezone clocks.
pub struct WorldClockMode {
    /// (label, timezone string) pairs.
    pub entries: Vec<(String, String)>,
}

impl WorldClockMode {
    /// Create a new `WorldClockMode` by parsing "Label=Timezone" entries.
    pub fn new(zones: &[String]) -> Self {
        Self {
            entries: parse_zones(zones),
        }
    }

    /// Re-parse entries when config changes.
    pub fn update_entries(&mut self, zones: &[String]) {
        self.entries = parse_zones(zones);
    }
}

/// Parse "Label=Timezone" strings into (label, tz) pairs. Invalid entries are skipped.
fn parse_zones(zones: &[String]) -> Vec<(String, String)> {
    zones
        .iter()
        .filter_map(|entry| {
            let parts: Vec<&str> = entry.splitn(2, '=').collect();
            if parts.len() == 2 {
                let label = parts[0].trim().to_string();
                let tz_str = parts[1].trim().to_string();
                // Validate timezone
                if Tz::from_str(&tz_str).is_ok() {
                    Some((label, tz_str))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

impl Mode for WorldClockMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::WorldClock
    }

    fn update(&mut self, _ctx: &mut RenderContext) {
        // No-op: time is read fresh each frame
    }

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();

        if self.entries.is_empty() {
            render::render_centered_text(
                frame,
                area,
                "No world clock zones configured",
                Color::DarkGray,
            );
            return;
        }

        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        // Each entry gets: Length(1) label + Length(font_height) time
        let entry_height = 1 + font_height;
        let n = self.entries.len() as u16;

        // Build constraints: fill top, then entries, fill bottom, hints
        let mut constraints = vec![Constraint::Fill(1)];
        for _ in 0..n {
            constraints.push(Constraint::Length(entry_height));
        }
        constraints.push(Constraint::Fill(1));
        constraints.push(Constraint::Length(1));

        let chunks = Layout::vertical(constraints).split(area);

        let now_utc = Utc::now();

        let params = AsciiTextParams {
            color_theme: ctx.color_theme,
            static_color: ctx.color(),
            animation_style: ctx.animation_style,
            animation_speed: ctx.animation_speed,
            elapsed_ms: ctx.elapsed_ms(),
            flash_intensity: ctx.flash_intensity,
            colon_blink: ctx.colon_blink,
        };

        for (i, (label, tz_str)) in self.entries.iter().enumerate() {
            let chunk_idx = 1 + i; // offset by the leading Fill(1)

            let entry_area = chunks[chunk_idx];

            // Split entry area into label row + clock rows
            let entry_chunks =
                Layout::vertical([Constraint::Length(1), Constraint::Length(font_height)])
                    .split(entry_area);

            // Render label
            render::render_centered_text(frame, entry_chunks[0], label, Color::DarkGray);

            // Render time in FIGlet font
            let time_str = if let Ok(tz) = Tz::from_str(tz_str) {
                let local_time = now_utc.with_timezone(&tz);
                match ctx.time_format {
                    TimeFormat::TwelveHour => local_time.format("%I:%M %p").to_string(),
                    TimeFormat::TwentyFourHour => local_time.format("%H:%M").to_string(),
                }
            } else {
                "??:??".to_string()
            };

            render::render_ascii_text(frame, entry_chunks[1], font, &time_str, &params);
        }

        // Key hints (last chunk)
        let hints_area = chunks[chunks.len() - 1];
        let hints = self.key_hints();
        let hint_str: String = hints
            .iter()
            .map(|(k, v)| format!("[{k}] {v}"))
            .collect::<Vec<_>>()
            .join("  ");
        render::render_centered_text(frame, hints_area, &hint_str, Color::DarkGray);
    }

    fn handle_key(&mut self, _key: KeyEvent, _ctx: &mut RenderContext) -> bool {
        false
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("t", "12/24h"),
            ("c", "color"),
            ("s", "settings"),
            ("?", "help"),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
