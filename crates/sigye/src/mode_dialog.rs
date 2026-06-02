//! Mode selection dialog — pick a display mode from a list.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use sigye_core::DisplayMode;

/// All modes shown in the picker, in display order.
const MODES: &[DisplayMode] = &[
    DisplayMode::Clock,
    DisplayMode::Pomodoro,
    DisplayMode::Timer,
    DisplayMode::Stopwatch,
    DisplayMode::WorldClock,
    DisplayMode::Countdown,
];

/// Outcome of a key event in the dialog.
pub enum ModeAction {
    /// Stay open.
    Continue,
    /// User picked a mode (Enter).
    Select(DisplayMode),
    /// User dismissed (Esc).
    Cancel,
}

/// Mode selection dialog state.
#[derive(Debug, Default)]
pub struct ModeDialog {
    pub visible: bool,
    /// Index into `MODES` of the highlighted entry.
    selected: usize,
}

impl ModeDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the dialog with the cursor on `current`.
    pub fn open(&mut self, current: DisplayMode) {
        self.visible = true;
        self.selected = MODES.iter().position(|m| *m == current).unwrap_or(0);
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Process a key event. Caller acts on the returned action.
    pub fn handle_key(&mut self, key: KeyEvent) -> ModeAction {
        match key.code {
            KeyCode::Esc => {
                self.close();
                ModeAction::Cancel
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let picked = MODES[self.selected];
                self.close();
                ModeAction::Select(picked)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = if self.selected == 0 {
                    MODES.len() - 1
                } else {
                    self.selected - 1
                };
                ModeAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % MODES.len();
                ModeAction::Continue
            }
            // Number-key shortcuts: 1-6 jumps directly.
            KeyCode::Char(c @ '1'..='9') => {
                let idx = (c as usize) - ('1' as usize);
                if idx < MODES.len() {
                    let picked = MODES[idx];
                    self.close();
                    return ModeAction::Select(picked);
                }
                ModeAction::Continue
            }
            _ => ModeAction::Continue,
        }
    }

    /// Render the dialog. `current` marks the active mode with a dot.
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        accent: Color,
        _dim: Color,
        muted: Color,
        current: DisplayMode,
    ) {
        if !self.visible {
            return;
        }

        let dialog_width = 36u16.min(area.width.saturating_sub(4));
        // border(2) + padding(2) + entries + help(1) + spacer(1)
        let dialog_height = (MODES.len() as u16 + 6).min(area.height.saturating_sub(2));

        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(" Mode ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent))
            .style(Style::default().fg(Color::White).bg(Color::Black));
        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

        // Build list lines
        let mut lines: Vec<Line> = Vec::with_capacity(MODES.len());
        for (i, mode) in MODES.iter().enumerate() {
            let is_selected = i == self.selected;
            let is_current = *mode == current;
            let marker = if is_selected { "►" } else { " " };
            let active = if is_current { " ●" } else { "  " };
            let number = format!(" {} ", i + 1);
            let label = format!("{}{}{}", marker, number, mode.display_name());

            let style = if is_selected {
                Style::default().fg(accent).bold()
            } else if is_current {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(muted)
            };

            lines.push(Line::from(vec![
                Span::styled(label, style),
                Span::styled(active.to_string(), Style::default().fg(accent)),
            ]));
        }

        let list_area = Rect::new(
            chunks[1].x + 2,
            chunks[1].y,
            chunks[1].width.saturating_sub(4),
            chunks[1].height,
        );
        frame.render_widget(Paragraph::new(lines), list_area);

        let help = Line::from(vec![
            Span::styled("↑↓", Style::default().fg(accent).bold()),
            Span::styled(" nav  ", Style::default().fg(muted)),
            Span::styled("1-6", Style::default().fg(accent).bold()),
            Span::styled(" jump  ", Style::default().fg(muted)),
            Span::styled("Enter", Style::default().fg(accent).bold()),
            Span::styled(" pick  ", Style::default().fg(muted)),
            Span::styled("Esc", Style::default().fg(accent).bold()),
            Span::styled(" cancel", Style::default().fg(muted)),
        ]);
        frame.render_widget(Paragraph::new(help).alignment(Alignment::Center), chunks[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};
    use ratatui::{Terminal, backend::TestBackend};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn opens_with_cursor_on_current() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Timer);
        assert!(d.visible);
        assert_eq!(MODES[d.selected], DisplayMode::Timer);
    }

    #[test]
    fn enter_selects_and_closes() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let action = d.handle_key(key(KeyCode::Down));
        assert!(matches!(action, ModeAction::Continue));
        let action = d.handle_key(key(KeyCode::Enter));
        match action {
            ModeAction::Select(m) => assert_eq!(m, DisplayMode::Pomodoro),
            _ => panic!("expected Select"),
        }
        assert!(!d.visible);
    }

    #[test]
    fn esc_cancels() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let action = d.handle_key(key(KeyCode::Esc));
        assert!(matches!(action, ModeAction::Cancel));
        assert!(!d.visible);
    }

    #[test]
    fn number_shortcut_jumps() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let action = d.handle_key(key(KeyCode::Char('6')));
        match action {
            ModeAction::Select(m) => assert_eq!(m, DisplayMode::Countdown),
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn up_wraps() {
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        d.handle_key(key(KeyCode::Up));
        assert_eq!(MODES[d.selected], DisplayMode::Countdown);
    }

    #[test]
    fn render_uses_muted_for_secondary_help_text() {
        let mut terminal = Terminal::new(TestBackend::new(50, 16)).unwrap();
        let mut d = ModeDialog::new();
        d.open(DisplayMode::Clock);
        let accent = Color::Red;
        let dim = Color::Rgb(10, 20, 30);
        let muted = Color::Rgb(40, 50, 60);

        terminal
            .draw(|frame| d.render(frame, frame.area(), accent, dim, muted, DisplayMode::Clock))
            .unwrap();

        assert_eq!(color_of_text(terminal.backend(), " nav  "), Some(muted));
    }

    fn color_of_text(backend: &TestBackend, text: &str) -> Option<Color> {
        let buffer = backend.buffer();
        let area = buffer.area;
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if text_matches_at(backend, x, y, text) {
                    return buffer.cell((x, y)).map(|cell| cell.fg);
                }
            }
        }
        None
    }

    fn text_matches_at(backend: &TestBackend, x: u16, y: u16, text: &str) -> bool {
        let buffer = backend.buffer();
        text.chars().enumerate().all(|(offset, ch)| {
            buffer
                .cell((x + offset as u16, y))
                .is_some_and(|cell| cell.symbol() == ch.to_string())
        })
    }
}
