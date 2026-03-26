//! Clock display mode.

use std::any::Any;

use chrono::Local;
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

/// Clock display mode — shows the current time as big ASCII art.
pub struct ClockMode {
    pub last_second: u32,
    pub last_minute: u32,
    pub last_hour: u32,
}

impl ClockMode {
    /// Create a new `ClockMode` initialized from the current local time.
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            last_second: now.format("%S").to_string().parse().unwrap_or(0),
            last_minute: now.format("%M").to_string().parse().unwrap_or(0),
            last_hour: now.format("%H").to_string().parse().unwrap_or(0),
        }
    }

    /// Detect time changes and trigger flash effects with appropriate intensity.
    fn update_flash(&mut self, ctx: &mut RenderContext) {
        let now = Local::now();
        let second: u32 = now.format("%S").to_string().parse().unwrap_or(0);
        let minute: u32 = now.format("%M").to_string().parse().unwrap_or(0);
        let hour: u32 = now.format("%H").to_string().parse().unwrap_or(0);

        if hour != self.last_hour {
            ctx.trigger_flash(1.0);
            self.last_hour = hour;
        } else if minute != self.last_minute {
            ctx.trigger_flash(0.7);
            self.last_minute = minute;
        } else if second != self.last_second {
            ctx.trigger_flash(0.3);
            self.last_second = second;
        }

        ctx.decay_flash();
    }
}

impl Default for ClockMode {
    fn default() -> Self {
        Self::new()
    }
}

impl Mode for ClockMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::Clock
    }

    fn update(&mut self, ctx: &mut RenderContext) {
        self.update_flash(ctx);
    }

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();

        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        // Layout: Fill(1), font height, Length(2) gap, Length(1) date, Fill(1), Length(1) hints
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(font_height),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

        let now = Local::now();

        // Build time string according to format
        let time_str = match ctx.time_format {
            TimeFormat::TwelveHour => {
                if ctx.show_seconds {
                    now.format("%I:%M:%S %p").to_string()
                } else {
                    now.format("%I:%M %p").to_string()
                }
            }
            TimeFormat::TwentyFourHour => {
                if ctx.show_seconds {
                    now.format("%H:%M:%S").to_string()
                } else {
                    now.format("%H:%M").to_string()
                }
            }
        };

        let params = AsciiTextParams {
            color_theme: ctx.color_theme,
            static_color: ctx.color(),
            animation_style: ctx.animation_style,
            animation_speed: ctx.animation_speed,
            elapsed_ms: ctx.elapsed_ms(),
            flash_intensity: ctx.flash_intensity,
            colon_blink: ctx.colon_blink,
        };

        // Render big ASCII time
        render::render_ascii_text(frame, chunks[1], font, &time_str, &params);

        // Render date string
        let date_str = now.format("%A, %B %-d, %Y").to_string();
        render::render_centered_text(frame, chunks[3], &date_str, Color::DarkGray);

        // Render key hints
        let hints = self.key_hints();
        let hint_str: String = hints
            .iter()
            .map(|(k, v)| format!("[{k}] {v}"))
            .collect::<Vec<_>>()
            .join("  ");
        render::render_centered_text(frame, chunks[5], &hint_str, Color::DarkGray);
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
