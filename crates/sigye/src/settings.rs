//! Settings dialog widget for configuring the clock.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use sigye_core::{AnimationSpeed, AnimationStyle, BackgroundStyle, ColorTheme, TimeFormat};

/// The settings field currently being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsField {
    #[default]
    Font,
    Color,
    TimeFormat,
    ShowSeconds,
    Animation,
    Speed,
    Background,
    ColonBlink,
    PomodoroWork,
    PomodoroBreak,
    PomodoroLongBreak,
    PomodoroSound,
    DesktopNotifications,
    TimerDuration,
}

impl SettingsField {
    /// Move to the next field.
    pub fn next(self) -> Self {
        match self {
            Self::Font => Self::Color,
            Self::Color => Self::TimeFormat,
            Self::TimeFormat => Self::ShowSeconds,
            Self::ShowSeconds => Self::Animation,
            Self::Animation => Self::Speed,
            Self::Speed => Self::Background,
            Self::Background => Self::ColonBlink,
            Self::ColonBlink => Self::PomodoroWork,
            Self::PomodoroWork => Self::PomodoroBreak,
            Self::PomodoroBreak => Self::PomodoroLongBreak,
            Self::PomodoroLongBreak => Self::PomodoroSound,
            Self::PomodoroSound => Self::DesktopNotifications,
            Self::DesktopNotifications => Self::TimerDuration,
            Self::TimerDuration => Self::Font,
        }
    }

    /// Move to the previous field.
    pub fn prev(self) -> Self {
        match self {
            Self::Font => Self::TimerDuration,
            Self::TimerDuration => Self::DesktopNotifications,
            Self::DesktopNotifications => Self::PomodoroSound,
            Self::PomodoroSound => Self::PomodoroLongBreak,
            Self::Color => Self::Font,
            Self::TimeFormat => Self::Color,
            Self::ShowSeconds => Self::TimeFormat,
            Self::Animation => Self::ShowSeconds,
            Self::Speed => Self::Animation,
            Self::Background => Self::Speed,
            Self::ColonBlink => Self::Background,
            Self::PomodoroWork => Self::ColonBlink,
            Self::PomodoroBreak => Self::PomodoroWork,
            Self::PomodoroLongBreak => Self::PomodoroBreak,
        }
    }
}

/// A row in the settings dialog layout.
enum RowKind {
    Header(&'static str),
    Field(SettingsField),
    Spacer,
}

/// Settings dialog state.
#[derive(Debug)]
pub struct SettingsDialog {
    /// Whether the dialog is visible.
    pub visible: bool,
    /// Currently selected field.
    pub selected_field: SettingsField,
    /// Scroll offset for vertical scrolling.
    scroll_offset: u16,
    /// Index into available fonts list.
    pub font_index: usize,
    /// List of available font names.
    pub available_fonts: Vec<String>,
    /// Current color theme selection.
    pub color_theme: ColorTheme,
    /// Current time format selection.
    pub time_format: TimeFormat,
    /// Current animation style selection.
    pub animation_style: AnimationStyle,
    /// Current animation speed selection.
    pub animation_speed: AnimationSpeed,
    /// Current background style selection.
    pub background_style: BackgroundStyle,
    /// Current colon blink setting.
    pub colon_blink: bool,
    /// Current show seconds setting.
    pub show_seconds: bool,
    /// Pomodoro work duration in minutes.
    pub pomodoro_work_mins: u32,
    /// Pomodoro break duration in minutes.
    pub pomodoro_break_mins: u32,
    /// Pomodoro long break duration in minutes.
    pub pomodoro_long_break_mins: u32,
    /// Pomodoro sound notification setting.
    pub pomodoro_sound: bool,
    /// Desktop notifications setting.
    pub desktop_notifications: bool,
    /// Timer countdown duration in minutes.
    pub timer_duration_mins: u32,
    /// Original font index (for cancel/revert).
    original_font_index: usize,
    /// Original color theme (for cancel/revert).
    original_color_theme: ColorTheme,
    /// Original time format (for cancel/revert).
    original_time_format: TimeFormat,
    /// Original animation style (for cancel/revert).
    original_animation_style: AnimationStyle,
    /// Original animation speed (for cancel/revert).
    original_animation_speed: AnimationSpeed,
    /// Original background style (for cancel/revert).
    original_background_style: BackgroundStyle,
    /// Original colon blink (for cancel/revert).
    original_colon_blink: bool,
    /// Original show seconds (for cancel/revert).
    original_show_seconds: bool,
    /// Original pomodoro work duration (for cancel/revert).
    original_pomodoro_work_mins: u32,
    /// Original pomodoro break duration (for cancel/revert).
    original_pomodoro_break_mins: u32,
    /// Original pomodoro long break duration (for cancel/revert).
    original_pomodoro_long_break_mins: u32,
    /// Original pomodoro sound (for cancel/revert).
    original_pomodoro_sound: bool,
    /// Original desktop notifications (for cancel/revert).
    original_desktop_notifications: bool,
    /// Original timer duration (for cancel/revert).
    original_timer_duration_mins: u32,
}

impl SettingsDialog {
    /// Create a new settings dialog.
    pub fn new(available_fonts: Vec<String>) -> Self {
        Self {
            visible: false,
            selected_field: SettingsField::default(),
            scroll_offset: 0,
            font_index: 0,
            available_fonts,
            color_theme: ColorTheme::default(),
            time_format: TimeFormat::default(),
            animation_style: AnimationStyle::default(),
            animation_speed: AnimationSpeed::default(),
            background_style: BackgroundStyle::default(),
            colon_blink: false,
            show_seconds: true,
            pomodoro_work_mins: 25,
            pomodoro_break_mins: 5,
            pomodoro_long_break_mins: 15,
            pomodoro_sound: true,
            desktop_notifications: true,
            timer_duration_mins: 5,
            original_font_index: 0,
            original_color_theme: ColorTheme::default(),
            original_time_format: TimeFormat::default(),
            original_animation_style: AnimationStyle::default(),
            original_animation_speed: AnimationSpeed::default(),
            original_background_style: BackgroundStyle::default(),
            original_colon_blink: false,
            original_show_seconds: true,
            original_pomodoro_work_mins: 25,
            original_pomodoro_break_mins: 5,
            original_pomodoro_long_break_mins: 15,
            original_pomodoro_sound: true,
            original_desktop_notifications: true,
            original_timer_duration_mins: 5,
        }
    }

    /// Open dialog with current settings.
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        &mut self,
        font_name: &str,
        color_theme: ColorTheme,
        time_format: TimeFormat,
        animation_style: AnimationStyle,
        animation_speed: AnimationSpeed,
        colon_blink: bool,
        show_seconds: bool,
        background_style: BackgroundStyle,
        pomodoro_work_mins: u32,
        pomodoro_break_mins: u32,
        pomodoro_long_break_mins: u32,
        pomodoro_sound: bool,
        desktop_notifications: bool,
        timer_duration_mins: u32,
    ) {
        self.visible = true;
        self.selected_field = SettingsField::default();
        self.scroll_offset = 0;
        self.color_theme = color_theme;
        self.time_format = time_format;
        self.animation_style = animation_style;
        self.animation_speed = animation_speed;
        self.background_style = background_style;
        self.colon_blink = colon_blink;
        self.show_seconds = show_seconds;
        self.pomodoro_work_mins = pomodoro_work_mins;
        self.pomodoro_break_mins = pomodoro_break_mins;
        self.pomodoro_long_break_mins = pomodoro_long_break_mins;
        self.pomodoro_sound = pomodoro_sound;
        self.desktop_notifications = desktop_notifications;
        self.timer_duration_mins = timer_duration_mins;

        // Find font index
        self.font_index = self
            .available_fonts
            .iter()
            .position(|f| f == font_name)
            .unwrap_or(0);

        // Store original values for cancel/revert
        self.original_font_index = self.font_index;
        self.original_color_theme = color_theme;
        self.original_time_format = time_format;
        self.original_animation_style = animation_style;
        self.original_animation_speed = animation_speed;
        self.original_background_style = background_style;
        self.original_colon_blink = colon_blink;
        self.original_show_seconds = show_seconds;
        self.original_pomodoro_work_mins = pomodoro_work_mins;
        self.original_pomodoro_break_mins = pomodoro_break_mins;
        self.original_pomodoro_long_break_mins = pomodoro_long_break_mins;
        self.original_pomodoro_sound = pomodoro_sound;
        self.original_desktop_notifications = desktop_notifications;
        self.original_timer_duration_mins = timer_duration_mins;
    }

    /// Close without saving.
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Get original font name (for reverting on cancel).
    pub fn original_font(&self) -> &str {
        self.available_fonts
            .get(self.original_font_index)
            .map(String::as_str)
            .unwrap_or("Standard")
    }

    /// Get original color theme (for reverting on cancel).
    pub fn original_color_theme(&self) -> ColorTheme {
        self.original_color_theme
    }

    /// Get original time format (for reverting on cancel).
    pub fn original_time_format(&self) -> TimeFormat {
        self.original_time_format
    }

    /// Get original animation style (for reverting on cancel).
    pub fn original_animation_style(&self) -> AnimationStyle {
        self.original_animation_style
    }

    /// Get original animation speed (for reverting on cancel).
    pub fn original_animation_speed(&self) -> AnimationSpeed {
        self.original_animation_speed
    }

    /// Get original colon blink (for reverting on cancel).
    pub fn original_colon_blink(&self) -> bool {
        self.original_colon_blink
    }

    /// Get original show seconds (for reverting on cancel).
    pub fn original_show_seconds(&self) -> bool {
        self.original_show_seconds
    }

    /// Get original background style (for reverting on cancel).
    pub fn original_background_style(&self) -> BackgroundStyle {
        self.original_background_style
    }

    /// Get original pomodoro work duration (for reverting on cancel).
    pub fn original_pomodoro_work_mins(&self) -> u32 {
        self.original_pomodoro_work_mins
    }

    /// Get original pomodoro break duration (for reverting on cancel).
    pub fn original_pomodoro_break_mins(&self) -> u32 {
        self.original_pomodoro_break_mins
    }

    /// Get original pomodoro long break duration (for reverting on cancel).
    pub fn original_pomodoro_long_break_mins(&self) -> u32 {
        self.original_pomodoro_long_break_mins
    }

    /// Get original pomodoro sound (for reverting on cancel).
    pub fn original_pomodoro_sound(&self) -> bool {
        self.original_pomodoro_sound
    }

    /// Get original desktop notifications (for reverting on cancel).
    pub fn original_desktop_notifications(&self) -> bool {
        self.original_desktop_notifications
    }

    /// Get original timer duration (for reverting on cancel).
    pub fn original_timer_duration_mins(&self) -> u32 {
        self.original_timer_duration_mins
    }

    /// Move to next field and ensure it's visible.
    pub fn next_field(&mut self) {
        self.selected_field = self.selected_field.next();
    }

    /// Move to previous field and ensure it's visible.
    pub fn prev_field(&mut self) {
        self.selected_field = self.selected_field.prev();
    }

    /// Get the section layout: ordered list of rows (headers, fields, spacers).
    fn section_layout() -> Vec<RowKind> {
        vec![
            RowKind::Header("Clock"),
            RowKind::Field(SettingsField::Font),
            RowKind::Field(SettingsField::Color),
            RowKind::Field(SettingsField::TimeFormat),
            RowKind::Field(SettingsField::ShowSeconds),
            RowKind::Field(SettingsField::ColonBlink),
            RowKind::Spacer,
            RowKind::Header("Animation"),
            RowKind::Field(SettingsField::Animation),
            RowKind::Field(SettingsField::Speed),
            RowKind::Field(SettingsField::Background),
            RowKind::Spacer,
            RowKind::Header("Pomodoro"),
            RowKind::Field(SettingsField::PomodoroWork),
            RowKind::Field(SettingsField::PomodoroBreak),
            RowKind::Field(SettingsField::PomodoroLongBreak),
            RowKind::Field(SettingsField::PomodoroSound),
            RowKind::Field(SettingsField::DesktopNotifications),
            RowKind::Spacer,
            RowKind::Header("Timer"),
            RowKind::Field(SettingsField::TimerDuration),
        ]
    }

    /// Get the row index of the currently selected field in the section layout.
    fn selected_field_row_index(&self) -> usize {
        Self::section_layout()
            .iter()
            .position(|row| matches!(row, RowKind::Field(f) if *f == self.selected_field))
            .unwrap_or(0)
    }

    /// Adjust scroll_offset so the selected field is within the visible window.
    fn ensure_visible(&mut self, visible_rows: u16) {
        let row_idx = self.selected_field_row_index() as u16;
        // Scroll up if selected field is above the visible window
        if row_idx < self.scroll_offset {
            self.scroll_offset = row_idx;
        }
        // Scroll down if selected field is below the visible window
        if row_idx >= self.scroll_offset + visible_rows {
            self.scroll_offset = row_idx.saturating_sub(visible_rows - 1);
        }
    }

    /// Select next value for current field.
    pub fn next_value(&mut self) {
        match self.selected_field {
            SettingsField::Font => {
                if !self.available_fonts.is_empty() {
                    self.font_index = (self.font_index + 1) % self.available_fonts.len();
                }
            }
            SettingsField::Color => {
                self.color_theme = self.color_theme.next();
            }
            SettingsField::TimeFormat => {
                self.time_format = self.time_format.toggle();
            }
            SettingsField::ShowSeconds => {
                self.show_seconds = !self.show_seconds;
            }
            SettingsField::Animation => {
                self.animation_style = self.animation_style.next();
            }
            SettingsField::Speed => {
                self.animation_speed = self.animation_speed.next();
            }
            SettingsField::Background => {
                self.background_style = self.background_style.next();
            }
            SettingsField::ColonBlink => {
                self.colon_blink = !self.colon_blink;
            }
            SettingsField::PomodoroWork => {
                // Cycle through common work durations: 15, 20, 25, 30, 45, 50, 60
                self.pomodoro_work_mins = match self.pomodoro_work_mins {
                    15 => 20,
                    20 => 25,
                    25 => 30,
                    30 => 45,
                    45 => 50,
                    50 => 60,
                    _ => 15,
                };
            }
            SettingsField::PomodoroBreak => {
                // Cycle through common break durations: 3, 5, 10, 15
                self.pomodoro_break_mins = match self.pomodoro_break_mins {
                    3 => 5,
                    5 => 10,
                    10 => 15,
                    _ => 3,
                };
            }
            SettingsField::PomodoroLongBreak => {
                // Cycle through common long break durations: 10, 15, 20, 30
                self.pomodoro_long_break_mins = match self.pomodoro_long_break_mins {
                    10 => 15,
                    15 => 20,
                    20 => 30,
                    _ => 10,
                };
            }
            SettingsField::PomodoroSound => {
                self.pomodoro_sound = !self.pomodoro_sound;
            }
            SettingsField::DesktopNotifications => {
                self.desktop_notifications = !self.desktop_notifications;
            }
            SettingsField::TimerDuration => {
                self.timer_duration_mins = (self.timer_duration_mins + 1).min(99);
            }
        }
    }

    /// Select previous value for current field.
    pub fn prev_value(&mut self) {
        match self.selected_field {
            SettingsField::Font => {
                if !self.available_fonts.is_empty() {
                    self.font_index = if self.font_index == 0 {
                        self.available_fonts.len() - 1
                    } else {
                        self.font_index - 1
                    };
                }
            }
            SettingsField::Color => {
                self.color_theme = self.color_theme.prev();
            }
            SettingsField::TimeFormat => {
                self.time_format = self.time_format.toggle();
            }
            SettingsField::ShowSeconds => {
                self.show_seconds = !self.show_seconds;
            }
            SettingsField::Animation => {
                self.animation_style = self.animation_style.prev();
            }
            SettingsField::Speed => {
                self.animation_speed = self.animation_speed.prev();
            }
            SettingsField::Background => {
                self.background_style = self.background_style.prev();
            }
            SettingsField::ColonBlink => {
                self.colon_blink = !self.colon_blink;
            }
            SettingsField::PomodoroWork => {
                // Cycle through common work durations (reverse): 60, 50, 45, 30, 25, 20, 15
                self.pomodoro_work_mins = match self.pomodoro_work_mins {
                    15 => 60,
                    20 => 15,
                    25 => 20,
                    30 => 25,
                    45 => 30,
                    50 => 45,
                    60 => 50,
                    _ => 25,
                };
            }
            SettingsField::PomodoroBreak => {
                // Cycle through common break durations (reverse): 15, 10, 5, 3
                self.pomodoro_break_mins = match self.pomodoro_break_mins {
                    3 => 15,
                    5 => 3,
                    10 => 5,
                    15 => 10,
                    _ => 5,
                };
            }
            SettingsField::PomodoroLongBreak => {
                // Cycle through common long break durations (reverse): 30, 20, 15, 10
                self.pomodoro_long_break_mins = match self.pomodoro_long_break_mins {
                    10 => 30,
                    15 => 10,
                    20 => 15,
                    30 => 20,
                    _ => 15,
                };
            }
            SettingsField::PomodoroSound => {
                self.pomodoro_sound = !self.pomodoro_sound;
            }
            SettingsField::DesktopNotifications => {
                self.desktop_notifications = !self.desktop_notifications;
            }
            SettingsField::TimerDuration => {
                self.timer_duration_mins = (self.timer_duration_mins.saturating_sub(1)).max(1);
            }
        }
    }

    /// Get currently selected font name.
    pub fn selected_font(&self) -> &str {
        self.available_fonts
            .get(self.font_index)
            .map(String::as_str)
            .unwrap_or("Standard")
    }

    /// Render the settings dialog.
    pub fn render(&mut self, frame: &mut Frame, area: Rect, accent_color: Color) {
        if !self.visible {
            return;
        }

        let layout = Self::section_layout();
        let total_content_rows = layout.len() as u16;

        // Proportional dialog sizing
        let dialog_width = 50.min(area.width.saturating_sub(4));
        // +5 for: border top/bottom (2) + help row (1) + top padding (1) + bottom padding (1)
        let dialog_height = (total_content_rows + 5).min(area.height.saturating_sub(2));

        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Create block with border and explicit colors for light theme support
        let block = Block::default()
            .title(" Settings ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent_color))
            .style(Style::default().fg(Color::White).bg(Color::Black));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Split inner area with padding around content
        let chunks = Layout::vertical([
            Constraint::Length(1), // Top padding
            Constraint::Fill(1),   // Scrollable content area
            Constraint::Length(1), // Bottom padding
            Constraint::Length(1), // Help text
        ])
        .split(inner_area);

        // Add horizontal padding (2 chars each side)
        let content_area = Rect::new(
            chunks[1].x + 2,
            chunks[1].y,
            chunks[1].width.saturating_sub(4),
            chunks[1].height,
        );
        let visible_rows = content_area.height;

        // Ensure selected field is visible (adjust scroll)
        self.ensure_visible(visible_rows);

        // Determine if we need scroll indicators
        let can_scroll_up = self.scroll_offset > 0;
        let can_scroll_down = total_content_rows > self.scroll_offset + visible_rows;

        // Render content rows within the visible scroll window
        for (row_idx, row) in layout.iter().enumerate() {
            let row_idx = row_idx as u16;
            if row_idx < self.scroll_offset {
                continue;
            }
            let visible_y = row_idx - self.scroll_offset;
            if visible_y >= visible_rows {
                break;
            }

            let row_area = Rect::new(
                content_area.x,
                content_area.y + visible_y,
                content_area.width,
                1,
            );

            match row {
                RowKind::Header(name) => {
                    let header = self.render_section_header(name, accent_color);
                    frame.render_widget(
                        Paragraph::new(header).alignment(Alignment::Center),
                        row_area,
                    );
                }
                RowKind::Field(field) => {
                    let line = self.render_field_for(*field, accent_color);
                    frame
                        .render_widget(Paragraph::new(line).alignment(Alignment::Center), row_area);
                }
                RowKind::Spacer => {} // Empty row
            }
        }

        // Render scroll indicators over first/last content rows
        if can_scroll_up {
            let indicator = Line::from(Span::styled("  ▲  ", Style::default().fg(accent_color)));
            let indicator_area = Rect::new(
                content_area.x + content_area.width.saturating_sub(6),
                content_area.y,
                6,
                1,
            );
            frame.render_widget(
                Paragraph::new(indicator).alignment(Alignment::Right),
                indicator_area,
            );
        }
        if can_scroll_down {
            let indicator = Line::from(Span::styled("  ▼  ", Style::default().fg(accent_color)));
            let indicator_area = Rect::new(
                content_area.x + content_area.width.saturating_sub(6),
                content_area.y + visible_rows.saturating_sub(1),
                6,
                1,
            );
            frame.render_widget(
                Paragraph::new(indicator).alignment(Alignment::Right),
                indicator_area,
            );
        }

        // Render help text
        let help = Line::from(vec![
            Span::styled("↑↓", Style::default().fg(accent_color).bold()),
            Span::styled(" nav  ", Style::default().fg(Color::Gray)),
            Span::styled("←→", Style::default().fg(accent_color).bold()),
            Span::styled(" change  ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(accent_color).bold()),
            Span::styled(" save  ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(accent_color).bold()),
            Span::styled(" cancel", Style::default().fg(Color::Gray)),
        ]);
        frame.render_widget(Paragraph::new(help).alignment(Alignment::Center), chunks[3]);
    }

    /// Render a section header line.
    fn render_section_header(&self, name: &str, accent_color: Color) -> Line<'static> {
        Line::from(Span::styled(
            format!("── {name} ──"),
            Style::default().fg(accent_color).bold(),
        ))
    }

    /// Render the appropriate field line for a given SettingsField.
    fn render_field_for(&self, field: SettingsField, accent_color: Color) -> Line<'static> {
        let selected = self.selected_field == field;
        match field {
            SettingsField::Font => {
                self.render_field("Font", self.selected_font(), selected, accent_color)
            }
            SettingsField::Color => self.render_field(
                "Color",
                self.color_theme.display_name(),
                selected,
                accent_color,
            ),
            SettingsField::TimeFormat => {
                let name = match self.time_format {
                    TimeFormat::TwentyFourHour => "24-hour",
                    TimeFormat::TwelveHour => "12-hour",
                };
                self.render_field("Format", name, selected, accent_color)
            }
            SettingsField::ShowSeconds => {
                let v = if self.show_seconds { "On" } else { "Off" };
                self.render_field("Seconds", v, selected, accent_color)
            }
            SettingsField::ColonBlink => {
                let v = if self.colon_blink { "On" } else { "Off" };
                self.render_field("Colon Blink", v, selected, accent_color)
            }
            SettingsField::Animation => self.render_field(
                "Animation",
                self.animation_style.display_name(),
                selected,
                accent_color,
            ),
            SettingsField::Speed => self.render_field_with_style(
                "Speed",
                self.animation_speed.display_name(),
                selected,
                accent_color,
                self.animation_style != AnimationStyle::None,
            ),
            SettingsField::Background => self.render_field(
                "Background",
                self.background_style.display_name(),
                selected,
                accent_color,
            ),
            SettingsField::PomodoroWork => {
                let v = format!("{} min", self.pomodoro_work_mins);
                self.render_field("Work", &v, selected, accent_color)
            }
            SettingsField::PomodoroBreak => {
                let v = format!("{} min", self.pomodoro_break_mins);
                self.render_field("Break", &v, selected, accent_color)
            }
            SettingsField::PomodoroLongBreak => {
                let v = format!("{} min", self.pomodoro_long_break_mins);
                self.render_field("Long Break", &v, selected, accent_color)
            }
            SettingsField::PomodoroSound => {
                let v = if self.pomodoro_sound { "On" } else { "Off" };
                self.render_field("Sound", v, selected, accent_color)
            }
            SettingsField::DesktopNotifications => {
                let v = if self.desktop_notifications {
                    "On"
                } else {
                    "Off"
                };
                self.render_field("Notifications", v, selected, accent_color)
            }
            SettingsField::TimerDuration => {
                let v = format!("{} min", self.timer_duration_mins);
                self.render_field("Duration", &v, selected, accent_color)
            }
        }
    }

    /// Render a single settings field line.
    fn render_field(
        &self,
        label: &str,
        value: &str,
        selected: bool,
        accent_color: Color,
    ) -> Line<'static> {
        if selected {
            let arrow_style = Style::default().fg(accent_color).bold();
            let value_style = Style::default().fg(accent_color).bold();
            let label_style = Style::default().fg(accent_color);
            Line::from(vec![
                Span::styled(String::from("► "), arrow_style),
                Span::styled(format!("{label}: "), label_style),
                Span::styled(String::from("◀ "), arrow_style),
                Span::styled(value.to_string(), value_style),
                Span::styled(String::from(" ▶"), arrow_style),
            ])
        } else {
            let label_style = Style::default().fg(Color::Gray);
            let value_style = Style::default().fg(Color::White);
            Line::from(vec![
                Span::styled(String::from("  "), Style::default()),
                Span::styled(format!("{label}: "), label_style),
                Span::styled(value.to_string(), value_style),
            ])
        }
    }

    /// Render a single settings field line with enabled/disabled state.
    fn render_field_with_style(
        &self,
        label: &str,
        value: &str,
        selected: bool,
        accent_color: Color,
        enabled: bool,
    ) -> Line<'static> {
        if !enabled {
            // Grayed out when disabled - no arrows
            let gray = Style::default().fg(Color::DarkGray);
            return Line::from(vec![
                Span::styled(String::from("  "), Style::default()),
                Span::styled(format!("{label}: "), gray),
                Span::styled(value.to_string(), gray),
            ]);
        }

        self.render_field(label, value, selected, accent_color)
    }
}
