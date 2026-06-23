//! Shared rendering helpers for display modes.

use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
};
use sigye_core::{AnimationSpeed, AnimationStyle, ColorTheme, apply_animation, is_colon_visible};
use sigye_fonts::Font;

use crate::context::RenderContext;

/// Parameters for rendering ASCII art text to the frame buffer.
pub struct AsciiTextParams {
    pub color_theme: ColorTheme,
    pub static_color: Color,
    pub animation_style: AnimationStyle,
    pub animation_speed: AnimationSpeed,
    pub elapsed_ms: u64,
    pub flash_intensity: f32,
    pub colon_blink: bool,
}

impl AsciiTextParams {
    pub fn from_ctx(ctx: &RenderContext, static_color: Color) -> Self {
        Self {
            color_theme: ctx.color_theme,
            static_color,
            animation_style: ctx.animation_style,
            animation_speed: ctx.animation_speed,
            elapsed_ms: ctx.elapsed_ms(),
            flash_intensity: ctx.flash_intensity,
            colon_blink: ctx.colon_blink,
        }
    }
}

/// Render FIGlet ASCII art text centered in the given area.
/// Writes directly to the frame buffer, skipping spaces to preserve background transparency.
/// Returns (width, height) of the rendered text in characters.
pub fn render_ascii_text(
    frame: &mut Frame,
    area: Rect,
    font: &Font,
    text: &str,
    params: &AsciiTextParams,
) -> (usize, usize) {
    let time_lines = font.render_text(text);
    let height = time_lines.len();
    let width = time_lines.first().map(|s| s.chars().count()).unwrap_or(0);

    let text_width = width as u16;
    let start_x = area.x + (area.width.saturating_sub(text_width)) / 2;

    let colon_positions: Vec<bool> = if params.colon_blink {
        let mut mask = vec![false; width];
        let mut x_pos = 0;
        for ch in text.chars() {
            let char_width = font.char_width(ch);
            if ch == ':' {
                for i in 0..char_width {
                    if x_pos + i < mask.len() {
                        mask[x_pos + i] = true;
                    }
                }
            }
            x_pos += char_width;
        }
        mask
    } else {
        vec![]
    };

    let buf = frame.buffer_mut();
    for (line_idx, line) in time_lines.iter().enumerate() {
        let y_pos = area.y + line_idx as u16;
        if y_pos >= area.y + area.height {
            break;
        }

        for (char_idx, ch) in line.chars().enumerate() {
            if ch == ' ' {
                continue;
            }

            let x_pos = start_x + char_idx as u16;
            if x_pos >= area.x + area.width {
                continue;
            }

            if params.colon_blink {
                let is_colon = colon_positions.get(char_idx).copied().unwrap_or(false);
                if is_colon && !is_colon_visible(params.elapsed_ms) {
                    continue;
                }
            }

            let base_color = if params.color_theme.is_dynamic() {
                params
                    .color_theme
                    .color_at_position(char_idx, line_idx, width, height)
            } else {
                params.static_color
            };

            let animated_color = apply_animation(
                base_color,
                params.animation_style,
                params.animation_speed,
                params.elapsed_ms,
                char_idx,
                width,
                params.flash_intensity,
            );

            if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                cell.set_char(ch);
                cell.set_fg(animated_color);
            }
        }
    }

    (width, height)
}

/// Render a single line of text centered in the given area, directly to buffer.
/// Skips spaces to preserve background transparency.
pub fn render_centered_text(frame: &mut Frame, area: Rect, text: &str, color: Color) {
    // Use char count (not byte len) so multi-byte characters like `─` (U+2500)
    // center correctly. Each rendered char is assumed to occupy one terminal cell.
    let text_width = text.chars().count() as u16;
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
        if let Some(cell) = buf.cell_mut(Position::new(x_pos, y)) {
            cell.set_char(ch);
            cell.set_fg(color);
        }
    }
}

/// Render a text-based progress bar.
pub fn render_progress_bar(progress: f64, width: u16, accent: Color) -> Line<'static> {
    let bar_width = width as usize;
    if bar_width == 0 {
        return Line::from("");
    }
    let filled = ((progress * bar_width as f64).round() as usize).min(bar_width);
    let empty = bar_width - filled;

    let filled_str: String = "\u{2501}".repeat(filled);
    let empty_str: String = "\u{2500}".repeat(empty);

    Line::from(vec![
        Span::styled(filled_str, Style::default().fg(accent)),
        Span::styled(empty_str, Style::default().dark_gray()),
    ])
}

/// Render mode key hints unless screensaver mode is hiding UI chrome.
pub fn render_key_hints(
    frame: &mut Frame,
    area: Rect,
    ctx: &RenderContext,
    hints: &[(&str, &str)],
) {
    if ctx.screensaver_mode {
        return;
    }

    let hint_str = hints
        .iter()
        .map(|(key, value)| format!("[{key}] {value}"))
        .collect::<Vec<_>>()
        .join("  ");
    render_centered_text(frame, area, &hint_str, ctx.dim_color());
}
