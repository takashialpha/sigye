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
            Self::PomodoroSound => Self::TimerDuration,
            Self::TimerDuration => Self::Font,
        }
    }

    /// Move to the previous field.
    pub fn prev(self) -> Self {
        match self {
            Self::Font => Self::TimerDuration,
            Self::TimerDuration => Self::PomodoroSound,
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

/// Settings dialog state.
#[derive(Debug)]
pub struct SettingsDialog {
    /// Whether the dialog is visible.
    pub visible: bool,
    /// Currently selected field.
    pub selected_field: SettingsField,
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
    /// Original timer duration (for cancel/revert).
    original_timer_duration_mins: u32,
}

impl SettingsDialog {
    /// Create a new settings dialog.
    pub fn new(available_fonts: Vec<String>) -> Self {
        Self {
            visible: false,
            selected_field: SettingsField::default(),
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
        timer_duration_mins: u32,
    ) {
        self.visible = true;
        self.selected_field = SettingsField::default();
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

    /// Get original timer duration (for reverting on cancel).
    pub fn original_timer_duration_mins(&self) -> u32 {
        self.original_timer_duration_mins
    }

    /// Move to next field.
    pub fn next_field(&mut self) {
        self.selected_field = self.selected_field.next();
    }

    /// Move to previous field.
    pub fn prev_field(&mut self) {
        self.selected_field = self.selected_field.prev();
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
    pub fn render(&self, frame: &mut Frame, area: Rect, accent_color: Color) {
        if !self.visible {
            return;
        }

        // Calculate centered dialog area
        let dialog_width = 40.min(area.width.saturating_sub(4));
        let dialog_height = 31.min(area.height.saturating_sub(2));

        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

        // Clear the area behind the dialog
        frame.render_widget(Clear, dialog_area);

        // Create block with border
        let block = Block::default()
            .title(" Settings ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent_color));

        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout for settings fields
        let chunks = Layout::vertical([
            Constraint::Length(1), // 0: Top padding
            Constraint::Length(1), // 1: Font
            Constraint::Length(1), // 2: Spacing
            Constraint::Length(1), // 3: Color
            Constraint::Length(1), // 4: Spacing
            Constraint::Length(1), // 5: Time Format
            Constraint::Length(1), // 6: Spacing
            Constraint::Length(1), // 7: Show Seconds
            Constraint::Length(1), // 8: Spacing
            Constraint::Length(1), // 9: Animation
            Constraint::Length(1), // 10: Spacing
            Constraint::Length(1), // 11: Speed
            Constraint::Length(1), // 12: Spacing
            Constraint::Length(1), // 13: Background
            Constraint::Length(1), // 14: Spacing
            Constraint::Length(1), // 15: Colon Blink
            Constraint::Length(1), // 16: Spacing
            Constraint::Length(1), // 17: Pomodoro Work
            Constraint::Length(1), // 18: Spacing
            Constraint::Length(1), // 19: Pomodoro Break
            Constraint::Length(1), // 20: Spacing
            Constraint::Length(1), // 21: Pomodoro Long Break
            Constraint::Length(1), // 22: Spacing
            Constraint::Length(1), // 23: Pomodoro Sound
            Constraint::Length(1), // 24: Spacing
            Constraint::Length(1), // 25: Timer Duration
            Constraint::Fill(1),   // 26: Bottom space
            Constraint::Length(1), // 27: Help text
        ])
        .split(inner_area);

        // Render font field
        let font_line = self.render_field(
            "Font",
            self.selected_font(),
            self.selected_field == SettingsField::Font,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(font_line).alignment(Alignment::Center),
            chunks[1],
        );

        // Render color field
        let color_line = self.render_field(
            "Color",
            self.color_theme.display_name(),
            self.selected_field == SettingsField::Color,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(color_line).alignment(Alignment::Center),
            chunks[3],
        );

        // Render time format field
        let time_format_name = match self.time_format {
            TimeFormat::TwentyFourHour => "24-hour",
            TimeFormat::TwelveHour => "12-hour",
        };
        let time_line = self.render_field(
            "Format",
            time_format_name,
            self.selected_field == SettingsField::TimeFormat,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(time_line).alignment(Alignment::Center),
            chunks[5],
        );

        // Render show seconds field
        let seconds_value = if self.show_seconds { "On" } else { "Off" };
        let seconds_line = self.render_field(
            "Seconds",
            seconds_value,
            self.selected_field == SettingsField::ShowSeconds,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(seconds_line).alignment(Alignment::Center),
            chunks[7],
        );

        // Render animation field
        let animation_line = self.render_field(
            "Animation",
            self.animation_style.display_name(),
            self.selected_field == SettingsField::Animation,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(animation_line).alignment(Alignment::Center),
            chunks[9],
        );

        // Render speed field (grayed out when Animation is None)
        let speed_line = self.render_field_with_style(
            "Speed",
            self.animation_speed.display_name(),
            self.selected_field == SettingsField::Speed,
            accent_color,
            self.animation_style != AnimationStyle::None,
        );
        frame.render_widget(
            Paragraph::new(speed_line).alignment(Alignment::Center),
            chunks[11],
        );

        // Render background field
        let background_line = self.render_field(
            "Background",
            self.background_style.display_name(),
            self.selected_field == SettingsField::Background,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(background_line).alignment(Alignment::Center),
            chunks[13],
        );

        // Render colon blink field
        let blink_value = if self.colon_blink { "On" } else { "Off" };
        let blink_line = self.render_field(
            "Colon Blink",
            blink_value,
            self.selected_field == SettingsField::ColonBlink,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(blink_line).alignment(Alignment::Center),
            chunks[15],
        );

        // Render pomodoro work duration field
        let work_value = format!("{} min", self.pomodoro_work_mins);
        let work_line = self.render_field(
            "Pomo Work",
            &work_value,
            self.selected_field == SettingsField::PomodoroWork,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(work_line).alignment(Alignment::Center),
            chunks[17],
        );

        // Render pomodoro break duration field
        let break_value = format!("{} min", self.pomodoro_break_mins);
        let break_line = self.render_field(
            "Pomo Break",
            &break_value,
            self.selected_field == SettingsField::PomodoroBreak,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(break_line).alignment(Alignment::Center),
            chunks[19],
        );

        // Render pomodoro long break duration field
        let long_break_value = format!("{} min", self.pomodoro_long_break_mins);
        let long_break_line = self.render_field(
            "Pomo Long",
            &long_break_value,
            self.selected_field == SettingsField::PomodoroLongBreak,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(long_break_line).alignment(Alignment::Center),
            chunks[21],
        );

        // Render pomodoro sound field
        let sound_value = if self.pomodoro_sound { "On" } else { "Off" };
        let sound_line = self.render_field(
            "Pomo Sound",
            sound_value,
            self.selected_field == SettingsField::PomodoroSound,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(sound_line).alignment(Alignment::Center),
            chunks[23],
        );

        // Render timer duration field
        let timer_value = format!("{} min", self.timer_duration_mins);
        let timer_line = self.render_field(
            "Timer",
            &timer_value,
            self.selected_field == SettingsField::TimerDuration,
            accent_color,
        );
        frame.render_widget(
            Paragraph::new(timer_line).alignment(Alignment::Center),
            chunks[25],
        );

        // Render help text
        let help = Line::from(vec![
            Span::styled("↑↓", Style::default().fg(accent_color).bold()),
            Span::styled(" nav  ", Style::default().dark_gray()),
            Span::styled("←→", Style::default().fg(accent_color).bold()),
            Span::styled(" change  ", Style::default().dark_gray()),
            Span::styled("Enter", Style::default().fg(accent_color).bold()),
            Span::styled(" save  ", Style::default().dark_gray()),
            Span::styled("Esc", Style::default().fg(accent_color).bold()),
            Span::styled(" cancel", Style::default().dark_gray()),
        ]);
        frame.render_widget(
            Paragraph::new(help).alignment(Alignment::Center),
            chunks[27],
        );
    }

    /// Render a single settings field line.
    fn render_field(
        &self,
        label: &str,
        value: &str,
        selected: bool,
        accent_color: Color,
    ) -> Line<'static> {
        let arrow_style = if selected {
            Style::default().fg(accent_color).bold()
        } else {
            Style::default().dark_gray()
        };

        let value_style = if selected {
            Style::default().fg(accent_color).bold()
        } else {
            Style::default()
        };

        let label_style = Style::default().dark_gray();

        Line::from(vec![
            Span::styled(format!("{label}: "), label_style),
            Span::styled(String::from("◀ "), arrow_style),
            Span::styled(value.to_string(), value_style),
            Span::styled(String::from(" ▶"), arrow_style),
        ])
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
            // Grayed out when disabled
            let gray = Style::default().dark_gray();
            return Line::from(vec![
                Span::styled(format!("{label}: "), gray),
                Span::styled(String::from("◀ "), gray),
                Span::styled(value.to_string(), gray),
                Span::styled(String::from(" ▶"), gray),
            ]);
        }

        self.render_field(label, value, selected, accent_color)
    }
}
