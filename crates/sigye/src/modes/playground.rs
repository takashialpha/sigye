//! FIGlet text playground mode — type text and see it rendered in ASCII art.

use std::any::Any;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Color,
};
use sigye_core::DisplayMode;

use crate::context::RenderContext;
use crate::mode::Mode;
use crate::render::{self, AsciiTextParams};

/// Playground mode — type text and see it rendered in the current FIGlet font.
pub struct PlaygroundMode {
    input_text: String,
}

impl PlaygroundMode {
    pub fn new() -> Self {
        Self {
            input_text: String::from("Hello"),
        }
    }
}

impl Default for PlaygroundMode {
    fn default() -> Self {
        Self::new()
    }
}

impl Mode for PlaygroundMode {
    fn display_mode(&self) -> DisplayMode {
        DisplayMode::Playground
    }

    fn update(&mut self, _ctx: &mut RenderContext) {}

    fn render(&self, frame: &mut Frame, ctx: &RenderContext) {
        let area = frame.area();
        let font = ctx.font_registry.get_or_default(&ctx.current_font);
        let font_height = font.height as u16;

        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(font_height),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

        let display_text = if self.input_text.is_empty() {
            "_"
        } else {
            &self.input_text
        };

        let params = AsciiTextParams {
            color_theme: ctx.color_theme,
            static_color: ctx.color(),
            animation_style: ctx.animation_style,
            animation_speed: ctx.animation_speed,
            elapsed_ms: ctx.elapsed_ms(),
            flash_intensity: ctx.flash_intensity,
            colon_blink: false,
        };

        render::render_ascii_text(frame, chunks[1], font, display_text, &params);

        // Show input prompt
        let prompt = format!("Type: {}_", self.input_text);
        render::render_centered_text(frame, chunks[3], &prompt, Color::Gray);

        // Show font name
        let font_info = format!("Font: {}", ctx.current_font);
        render::render_centered_text(frame, chunks[4], &font_info, Color::DarkGray);

        // Key hints
        let hints = self.key_hints();
        let hint_str: String = hints
            .iter()
            .map(|(k, v)| format!("[{k}] {v}"))
            .collect::<Vec<_>>()
            .join("  ");
        render::render_centered_text(frame, chunks[6], &hint_str, Color::DarkGray);
    }

    fn handle_key(&mut self, key: KeyEvent, _ctx: &mut RenderContext) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return false;
                }
                // Pass through global keybindings
                match c {
                    'q' | 'm' | 't' | 'c' | 'a' | 'b' | 's' => return false,
                    '?' => return false,
                    _ => {}
                }
                if self.input_text.len() < 40 {
                    self.input_text.push(c);
                }
                true
            }
            KeyCode::Backspace => {
                self.input_text.pop();
                true
            }
            KeyCode::Delete => {
                self.input_text.clear();
                true
            }
            _ => false,
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("type", "enter text"),
            ("BS", "delete"),
            ("Del", "clear"),
            ("m", "mode"),
            ("?", "help"),
        ]
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
