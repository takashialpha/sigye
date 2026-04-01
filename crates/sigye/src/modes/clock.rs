//! Clock display mode.

use std::any::Any;

use chrono::{Datelike, Local, Timelike};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::Color,
};
use sigye_core::{ClockDisplayFormat, DisplayMode, TimeFormat};

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// Clock display mode — shows the current time as big ASCII art.
pub struct ClockMode {
    pub last_second: u32,
    pub last_minute: u32,
    pub last_hour: u32,
    pub display_format: ClockDisplayFormat,
}

impl ClockMode {
    /// Create a new `ClockMode` initialized from the current local time.
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            last_second: now.format("%S").to_string().parse().unwrap_or(0),
            last_minute: now.format("%M").to_string().parse().unwrap_or(0),
            last_hour: now.format("%H").to_string().parse().unwrap_or(0),
            display_format: ClockDisplayFormat::HumanReadable,
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

/// Render a progress bar line with colored filled/unfilled portions directly to the buffer.
fn render_progress_line(frame: &mut Frame, area: Rect, text: &str, accent: Color) {
    let text_width = text.len() as u16;
    let start_x = area.x + (area.width.saturating_sub(text_width)) / 2;
    let y = area.y;

    let buf = frame.buffer_mut();
    for (char_idx, ch) in text.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        let x_pos = start_x + char_idx as u16;
        if x_pos >= area.x + area.width {
            continue;
        }
        let color = if ch == '━' { accent } else { Color::DarkGray };
        if let Some(cell) = buf.cell_mut(Position::new(x_pos, y)) {
            cell.set_char(ch);
            cell.set_fg(color);
        }
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

        let has_sun_info = ctx.sunrise_sunset.is_some();

        // Layout: Fill(1), font height, Length(2) gap, Length(1) date, [optional Length(1) sun],
        //         Length(1) format_info, Length(1) progress_bar, Fill(1), Length(1) hints
        let sun_height = if has_sun_info { 1 } else { 0 };
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(font_height),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(sun_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

        let now = Local::now();

        // Build the text to display as big ASCII art based on display_format
        let big_text = match self.display_format {
            ClockDisplayFormat::HumanReadable => match ctx.time_format {
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
            },
            ClockDisplayFormat::UnixTimestamp => now.timestamp().to_string(),
            ClockDisplayFormat::Iso8601 => {
                // Big text is still the normal time; ISO string goes in the info line
                match ctx.time_format {
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
                }
            }
            ClockDisplayFormat::HexTime => {
                let h = now.hour();
                let m = now.minute();
                let s = now.second();
                format!("{h:02X}:{m:02X}:{s:02X}")
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
        render::render_ascii_text(frame, chunks[1], font, &big_text, &params);

        // Render date string
        let date_str = now.format("%A, %B %-d, %Y").to_string();
        render::render_centered_text(frame, chunks[3], &date_str, Color::DarkGray);

        // Render sunrise/sunset info if available
        if let Some((ref sunrise, ref sunset)) = ctx.sunrise_sunset {
            let sun_str = format!("Sunrise {}  Sunset {}", sunrise, sunset);
            render::render_centered_text(frame, chunks[4], &sun_str, Color::DarkGray);
        }

        // Render format info line
        match self.display_format {
            ClockDisplayFormat::HumanReadable => {}
            ClockDisplayFormat::UnixTimestamp => {
                render::render_centered_text(frame, chunks[5], "Unix Timestamp", Color::DarkGray);
            }
            ClockDisplayFormat::Iso8601 => {
                let iso = now.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
                render::render_centered_text(frame, chunks[5], &iso, Color::Gray);
            }
            ClockDisplayFormat::HexTime => {
                render::render_centered_text(frame, chunks[5], "Hex Time", Color::DarkGray);
            }
        }

        // Render day/year progress bar
        let day_progress = {
            let h = now.hour();
            let m = now.minute();
            let s = now.second();
            (h * 3600 + m * 60 + s) as f64 / 86400.0
        };
        let year_progress = {
            let day_of_year = now.ordinal() as f64;
            let year = now.year();
            let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                366.0
            } else {
                365.0
            };
            day_of_year / days_in_year
        };

        let day_pct = (day_progress * 100.0) as u32;
        let year_pct = (year_progress * 100.0) as u32;

        let bar_width = 20usize;
        let day_filled = (day_progress * bar_width as f64) as usize;
        let year_filled = (year_progress * bar_width as f64) as usize;

        let day_bar: String = "━".repeat(day_filled) + &"╌".repeat(bar_width - day_filled);
        let year_bar: String = "━".repeat(year_filled) + &"╌".repeat(bar_width - year_filled);

        let progress_text = format!("Day {day_bar} {day_pct}%  Year {year_bar} {year_pct}%");
        render_progress_line(frame, chunks[6], &progress_text, ctx.color());

        // Render key hints
        let hints = self.key_hints();
        let hint_str: String = hints
            .iter()
            .map(|(k, v)| format!("[{k}] {v}"))
            .collect::<Vec<_>>()
            .join("  ");
        render::render_centered_text(frame, chunks[8], &hint_str, Color::DarkGray);
    }

    fn handle_key(&mut self, key: KeyEvent, ctx: &mut RenderContext) -> bool {
        match key.code {
            KeyCode::Char('f') => {
                self.display_format = self.display_format.next();
                true
            }
            KeyCode::Char('u') => {
                let ts = Local::now().timestamp().to_string();
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(ts);
                }
                ctx.trigger_flash(0.5);
                true
            }
            KeyCode::Char('i') => {
                let iso = Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string();
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(iso);
                }
                ctx.trigger_flash(0.5);
                true
            }
            _ => false,
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("f", "format"),
            ("u", "copy unix"),
            ("i", "copy iso"),
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
