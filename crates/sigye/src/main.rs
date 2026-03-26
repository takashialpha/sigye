//! sigye - A terminal clock application with configurable fonts.

mod settings;
mod system_metrics;
mod weather;

use std::time::{Duration, Instant};

use chrono::Local;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Position},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};
use sigye_config::Config;
use sigye_core::{
    AnimationSpeed, AnimationStyle, BackgroundStyle, ColorTheme, DisplayMode, PomodoroPhase,
    TimeFormat, apply_animation, is_colon_visible,
};
use sigye_fonts::FontRegistry;

use settings::SettingsDialog;
use sigye_background::BackgroundState;
use system_metrics::SystemMonitor;
use weather::WeatherMonitor;

fn send_desktop_notification(title: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show();
}

/// A beautiful terminal clock with ASCII art fonts, animations, and backgrounds.
#[derive(Parser)]
#[command(name = "sigye", version, about)]
struct Cli {
    /// Start in screensaver mode (fullscreen, no UI chrome)
    #[arg(long)]
    screensaver: bool,

    /// Start in demo mode (auto-cycle themes and backgrounds)
    #[arg(long)]
    demo: bool,

    /// Set the font name
    #[arg(long)]
    font: Option<String>,

    /// Set the color theme
    #[arg(long)]
    theme: Option<String>,

    /// Set the background style
    #[arg(long, name = "bg")]
    background: Option<String>,

    /// Set the display mode (clock, pomodoro, timer, stopwatch, worldclock)
    #[arg(long)]
    mode: Option<String>,

    /// Add a world clock timezone (e.g., "Tokyo=Asia/Tokyo"). Can be repeated.
    #[arg(long = "tz", value_name = "LABEL=TIMEZONE")]
    timezones: Vec<String>,

    /// Shell command to execute on timer/pomodoro completion
    #[arg(long)]
    on_complete: Option<String>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let terminal = ratatui::init();
    let result = App::new_with_cli(cli).run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    /// Current time format (12h or 24h).
    time_format: TimeFormat,
    /// Current color theme.
    color_theme: ColorTheme,
    /// Current animation style.
    animation_style: AnimationStyle,
    /// Current animation speed.
    animation_speed: AnimationSpeed,
    /// Whether colon blinks.
    colon_blink: bool,
    /// Whether to show seconds in the clock display.
    show_seconds: bool,
    /// Current background style.
    background_style: BackgroundStyle,
    /// Current font name.
    current_font: String,
    /// Font registry containing all available fonts.
    font_registry: FontRegistry,
    /// Settings dialog state.
    settings_dialog: SettingsDialog,
    /// Configuration for persistence.
    config: Config,
    /// Animation start time.
    animation_start: Instant,
    /// Last recorded second (for reactive animation).
    last_second: u32,
    /// Last recorded minute (for reactive animation).
    last_minute: u32,
    /// Last recorded hour (for reactive animation).
    last_hour: u32,
    /// Current flash intensity (0.0 to 1.0).
    flash_intensity: f32,
    /// When the last flash started (for decay calculation).
    flash_start: Option<Instant>,
    /// Background animation state.
    background_state: BackgroundState,
    /// System monitor for reactive backgrounds (lazy initialized).
    system_monitor: Option<SystemMonitor>,
    /// Weather monitor for dynamic weather background (lazy initialized).
    weather_monitor: Option<WeatherMonitor>,
    /// Current display mode (Clock or Pomodoro).
    display_mode: DisplayMode,
    /// Current pomodoro phase.
    pomodoro_phase: PomodoroPhase,
    /// Remaining seconds in pomodoro timer.
    pomodoro_remaining_secs: u32,
    /// Number of completed work sessions.
    pomodoro_sessions_completed: u32,
    /// Total focus time in seconds (accumulated from completed work sessions).
    pomodoro_total_focus_secs: u64,
    /// When the current work session started (for tracking).
    pomodoro_work_start: Option<Instant>,
    /// Last tick time for pomodoro timer.
    pomodoro_last_tick: Instant,
    /// Whether pomodoro timer is running.
    pomodoro_running: bool,
    /// Timer countdown duration in seconds (configured value).
    timer_duration_secs: u32,
    /// Remaining seconds in timer countdown.
    timer_remaining_secs: u32,
    /// Whether timer is running.
    timer_running: bool,
    /// Last tick time for timer.
    timer_last_tick: Instant,
    /// Whether timer has completed (reached zero).
    timer_completed: bool,
    /// Stopwatch elapsed time in milliseconds (accumulated).
    stopwatch_elapsed_ms: u64,
    /// Whether stopwatch is running.
    stopwatch_running: bool,
    /// Last tick time for stopwatch.
    stopwatch_last_tick: Instant,
    /// Lap times (stored as elapsed ms at each lap).
    stopwatch_laps: Vec<u64>,
    /// Parsed world clock entries: (label, timezone_name).
    world_clock_entries: Vec<(String, String)>,
    /// Whether help overlay is visible.
    show_help: bool,
    /// Whether screensaver mode is active (hide UI chrome).
    screensaver_mode: bool,
    /// Whether demo mode is active (auto-cycle).
    demo_mode: bool,
    /// Timer for demo mode theme cycling.
    demo_theme_cycle: Instant,
    /// Timer for demo mode background cycling.
    demo_bg_cycle: Instant,
    /// Timer for demo mode font cycling.
    demo_font_cycle: Instant,
    /// Shell command to run on timer/pomodoro completion.
    on_complete_command: Option<String>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        // Load configuration
        let config = Config::load();

        // Initialize font registry with bundled fonts
        let mut font_registry = FontRegistry::new();

        // Load custom fonts from config directory
        font_registry.load_custom_fonts(&Config::fonts_dir());

        // Get list of available fonts for settings dialog
        let available_fonts: Vec<String> = font_registry
            .list_fonts()
            .into_iter()
            .map(String::from)
            .collect();

        // Create settings dialog
        let settings_dialog = SettingsDialog::new(available_fonts);

        // Get current time for initial state
        let now = chrono::Local::now();

        // Initialize system monitor if reactive background is selected
        let system_monitor = if config.background_style.is_reactive() {
            let monitor = SystemMonitor::new();
            monitor.start();
            Some(monitor)
        } else {
            None
        };

        // Initialize weather monitor if weather background is selected
        let weather_monitor = if config.background_style.requires_weather() {
            let monitor = WeatherMonitor::new(config.weather_location.clone());
            monitor.start();
            Some(monitor)
        } else {
            None
        };

        // Capture pomodoro work duration before config is moved
        // Parse world clock entries
        let world_clock_entries: Vec<(String, String)> = config
            .world_clock_zones
            .iter()
            .map(|entry| {
                if let Some((label, tz)) = entry.split_once('=') {
                    (label.trim().to_string(), tz.trim().to_string())
                } else {
                    (entry.clone(), entry.clone())
                }
            })
            .collect();

        let on_complete_command = config.on_complete.clone();
        let pomodoro_initial_secs = config.pomodoro_work_mins * 60;
        let timer_initial_secs = config.timer_duration_mins * 60;

        Self {
            running: false,
            time_format: config.time_format,
            color_theme: config.color_theme,
            animation_style: config.animation_style,
            animation_speed: config.animation_speed,
            colon_blink: config.colon_blink,
            show_seconds: config.show_seconds,
            background_style: config.background_style,
            current_font: config.font_name.clone(),
            font_registry,
            settings_dialog,
            config,
            animation_start: Instant::now(),
            last_second: now.format("%S").to_string().parse().unwrap_or(0),
            last_minute: now.format("%M").to_string().parse().unwrap_or(0),
            last_hour: now.format("%H").to_string().parse().unwrap_or(0),
            flash_intensity: 0.0,
            flash_start: None,
            background_state: BackgroundState::new(),
            system_monitor,
            weather_monitor,
            display_mode: DisplayMode::default(),
            pomodoro_phase: PomodoroPhase::default(),
            pomodoro_remaining_secs: pomodoro_initial_secs,
            pomodoro_sessions_completed: 0,
            pomodoro_total_focus_secs: 0,
            pomodoro_work_start: None,
            pomodoro_last_tick: Instant::now(),
            pomodoro_running: false,
            timer_duration_secs: timer_initial_secs,
            timer_remaining_secs: timer_initial_secs,
            timer_running: false,
            timer_last_tick: Instant::now(),
            timer_completed: false,
            stopwatch_elapsed_ms: 0,
            stopwatch_running: false,
            stopwatch_last_tick: Instant::now(),
            stopwatch_laps: Vec::new(),
            world_clock_entries,
            show_help: false,
            screensaver_mode: false,
            demo_mode: false,
            demo_theme_cycle: Instant::now(),
            demo_bg_cycle: Instant::now(),
            demo_font_cycle: Instant::now(),
            on_complete_command,
        }
    }

    /// Construct a new instance with CLI overrides applied.
    fn new_with_cli(cli: Cli) -> Self {
        let mut app = Self::new();

        if let Some(font_name) = cli.font {
            let fonts = app.font_registry.list_fonts();
            if let Some(matched) = fonts.iter().find(|f| f.eq_ignore_ascii_case(&font_name)) {
                app.current_font = matched.to_string();
            } else {
                app.current_font = font_name;
            }
        }

        if let Some(theme_str) = cli.theme
            && let Some(theme) = parse_color_theme(&theme_str) {
                app.color_theme = theme;
            }

        if let Some(bg_str) = cli.background
            && let Some(bg) = parse_background_style(&bg_str) {
                app.background_style = bg;
                app.update_background_monitors();
            }

        if let Some(mode_str) = cli.mode
            && let Some(mode) = parse_display_mode(&mode_str) {
                app.display_mode = mode;
            }

        if !cli.timezones.is_empty() {
            app.config.world_clock_zones = cli.timezones;
            app.world_clock_entries = app
                .config
                .world_clock_zones
                .iter()
                .map(|entry| {
                    if let Some((label, tz)) = entry.split_once('=') {
                        (label.trim().to_string(), tz.trim().to_string())
                    } else {
                        (entry.clone(), entry.clone())
                    }
                })
                .collect();
        }

        if cli.on_complete.is_some() {
            app.on_complete_command = cli.on_complete;
        }

        app.screensaver_mode = cli.screensaver;
        app.demo_mode = cli.demo;

        // In screensaver mode with no background, default to Aurora
        if app.screensaver_mode && app.background_style == BackgroundStyle::None {
            app.background_style = BackgroundStyle::Aurora;
            app.update_background_monitors();
        }

        app
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        // Demo mode auto-cycling
        if self.demo_mode {
            if self.demo_theme_cycle.elapsed() >= Duration::from_secs(5) {
                self.color_theme = self.color_theme.next();
                self.demo_theme_cycle = Instant::now();
            }
            if self.demo_bg_cycle.elapsed() >= Duration::from_secs(15) {
                self.background_style = self.background_style.next();
                self.update_background_monitors();
                self.demo_bg_cycle = Instant::now();
            }
            if self.demo_font_cycle.elapsed() >= Duration::from_secs(30) {
                let fonts = self.font_registry.list_fonts();
                let current_idx = fonts.iter().position(|f| f == &self.current_font).unwrap_or(0);
                let next_idx = (current_idx + 1) % fonts.len();
                self.current_font = fonts[next_idx].to_string();
                self.demo_font_cycle = Instant::now();
            }
        }

        let now = Local::now();

        // Calculate animation elapsed time
        let elapsed_ms = self.animation_start.elapsed().as_millis() as u64;

        // Get metrics for reactive backgrounds
        let metrics = self.system_monitor.as_ref().map(|m| m.get_metrics());

        // Resolve weather background to actual style
        let effective_background = if self.background_style == BackgroundStyle::Weather {
            self.weather_monitor
                .as_ref()
                .map(|m| m.get_background())
                .unwrap_or(BackgroundStyle::Starfield)
        } else {
            self.background_style
        };

        // Render background first (behind everything else)
        self.background_state.render(
            frame,
            effective_background,
            elapsed_ms,
            self.animation_speed,
            metrics.as_ref(),
        );

        // Update flash intensity for reactive animation (clock mode only)
        if self.display_mode == DisplayMode::Clock {
            self.update_flash(&now);
        }

        // Update timers
        self.update_pomodoro();
        self.update_timer();

        // Branch rendering based on display mode
        if self.display_mode == DisplayMode::WorldClock {
            self.render_world_clock(frame, elapsed_ms);
            let color = self.color_theme.color();
            let area = frame.area();
            self.settings_dialog.render(frame, area, color);
            self.render_help_overlay(frame);
            return;
        }

        if self.display_mode == DisplayMode::Stopwatch {
            self.render_stopwatch(frame, elapsed_ms);
            let color = self.color_theme.color();
            let area = frame.area();
            self.settings_dialog.render(frame, area, color);
            self.render_help_overlay(frame);
            return;
        }

        if self.display_mode == DisplayMode::Timer {
            self.render_timer(frame, elapsed_ms);
            let color = self.color_theme.color();
            let area = frame.area();
            self.settings_dialog.render(frame, area, color);
            self.render_help_overlay(frame);
            return;
        }

        if self.display_mode == DisplayMode::Pomodoro {
            self.render_pomodoro(frame, elapsed_ms);
            let color = self.color_theme.color();
            let area = frame.area();
            self.settings_dialog.render(frame, area, color);
            self.render_help_overlay(frame);
            return;
        }

        // Get time components
        let (hours, is_pm) = match self.time_format {
            TimeFormat::TwentyFourHour => {
                (now.format("%H").to_string().parse().unwrap_or(0), false)
            }
            TimeFormat::TwelveHour => {
                let h: u32 = now.format("%I").to_string().parse().unwrap_or(12);
                let pm = now.format("%p").to_string() == "PM";
                (h, pm)
            }
        };
        let minutes: u32 = now.format("%M").to_string().parse().unwrap_or(0);
        let seconds: u32 = now.format("%S").to_string().parse().unwrap_or(0);

        // Format date
        let date_str = now.format("%A, %B %d, %Y").to_string();

        let color = self.color_theme.color();
        let area = frame.area();

        // Build time string
        let time_str = match (self.time_format, self.show_seconds) {
            (TimeFormat::TwentyFourHour, true) => {
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            }
            (TimeFormat::TwentyFourHour, false) => {
                format!("{hours:02}:{minutes:02}")
            }
            (TimeFormat::TwelveHour, true) => {
                let ampm = if is_pm { "PM" } else { "AM" };
                format!("{hours:2}:{minutes:02}:{seconds:02} {ampm}")
            }
            (TimeFormat::TwelveHour, false) => {
                let ampm = if is_pm { "PM" } else { "AM" };
                format!("{hours:2}:{minutes:02} {ampm}")
            }
        };

        // Get current font and render
        let font = self.font_registry.get_or_default(&self.current_font);
        let time_lines = font.render_text(&time_str);
        let font_height = font.height as u16;

        // Create vertical layout for centering
        let chunks = Layout::vertical([
            Constraint::Fill(1),             // Top padding
            Constraint::Length(font_height), // Big digits (dynamic height)
            Constraint::Length(2),           // Spacing
            Constraint::Length(1),           // Date
            Constraint::Fill(1),             // Bottom padding
            Constraint::Length(1),           // Help text
        ])
        .split(area);

        // Render big time
        let height = time_lines.len();
        let width = time_lines.first().map(|s| s.chars().count()).unwrap_or(0);

        // Build colon position mask for blink effect
        // Maps x-positions in rendered ASCII art back to colon characters in time_str
        let colon_positions: Vec<bool> = if self.colon_blink {
            let mut mask = vec![false; width];
            let mut x_pos = 0;
            for ch in time_str.chars() {
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

        // Render time directly to buffer, skipping spaces to preserve background
        let chunk = chunks[1];
        let text_width = width as u16;
        let start_x = chunk.x + (chunk.width.saturating_sub(text_width)) / 2;

        let buf = frame.buffer_mut();
        for (line_idx, line) in time_lines.iter().enumerate() {
            let y_pos = chunk.y + line_idx as u16;
            if y_pos >= chunk.y + chunk.height {
                break;
            }

            for (char_idx, ch) in line.chars().enumerate() {
                // Skip spaces to preserve background transparency
                if ch == ' ' {
                    continue;
                }

                let x_pos = start_x + char_idx as u16;
                if x_pos >= chunk.x + chunk.width {
                    continue;
                }

                // Apply colon blink by skipping colon characters during "off" phase
                let is_colon = colon_positions.get(char_idx).copied().unwrap_or(false);
                let should_hide = self.colon_blink && is_colon && !is_colon_visible(elapsed_ms);
                if should_hide {
                    continue;
                }

                // Get base color
                let base_color = if self.color_theme.is_dynamic() {
                    self.color_theme
                        .color_at_position(char_idx, line_idx, width, height)
                } else {
                    color
                };

                // Apply animation
                let animated_color = apply_animation(
                    base_color,
                    self.animation_style,
                    self.animation_speed,
                    elapsed_ms,
                    char_idx,
                    width,
                    self.flash_intensity,
                );

                // Write directly to buffer
                if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                    cell.set_char(ch);
                    cell.set_fg(animated_color);
                }
            }
        }

        // Render date directly to buffer - skip in screensaver mode
        if !self.screensaver_mode {
            let date_chunk = chunks[3];
            let date_width = date_str.len() as u16;
            let date_start_x = date_chunk.x + (date_chunk.width.saturating_sub(date_width)) / 2;
            let date_y = date_chunk.y;

            let buf = frame.buffer_mut();
            for (char_idx, ch) in date_str.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }

                let x_pos = date_start_x + char_idx as u16;
                if x_pos >= date_chunk.x + date_chunk.width {
                    continue;
                }

                let base_color = if self.color_theme.is_dynamic() {
                    self.color_theme
                        .color_at_position(char_idx, 0, date_str.len(), 1)
                } else {
                    color
                };

                let animated_color = apply_animation(
                    base_color,
                    self.animation_style,
                    self.animation_speed,
                    elapsed_ms,
                    char_idx,
                    date_str.len(),
                    self.flash_intensity,
                );

                if let Some(cell) = buf.cell_mut(Position::new(x_pos, date_y)) {
                    cell.set_char(ch);
                    cell.set_fg(animated_color);
                }
            }
        }

        // Render help text (clock mode) - skip in screensaver mode
        if !self.screensaver_mode {
            let help = Line::from(vec![
                "q".bold().fg(color),
                " quit  ".dark_gray(),
                "m".bold().fg(color),
                " mode  ".dark_gray(),
                "t".bold().fg(color),
                " 12/24h  ".dark_gray(),
                "c".bold().fg(color),
                " color  ".dark_gray(),
                "s".bold().fg(color),
                " settings  ".dark_gray(),
                "?".bold().fg(color),
                " help".dark_gray(),
            ])
            .centered();
            frame.render_widget(help, chunks[5]);
        }

        // Render settings dialog if visible
        self.settings_dialog.render(frame, area, color);
        self.render_help_overlay(frame);
    }

    /// Render the pomodoro timer display.
    fn render_pomodoro(&mut self, frame: &mut Frame, elapsed_ms: u64) {
        let color = self.color_theme.color();
        let area = frame.area();

        // Format time as MM:SS
        let mins = self.pomodoro_remaining_secs / 60;
        let secs = self.pomodoro_remaining_secs % 60;
        let time_str = format!("{mins:02}:{secs:02}");

        // Get current font and render
        let font = self.font_registry.get_or_default(&self.current_font);
        let time_lines = font.render_text(&time_str);
        let font_height = font.height as u16;

        // Create vertical layout for centering
        let chunks = Layout::vertical([
            Constraint::Fill(1),             // Top padding
            Constraint::Length(font_height), // Timer digits
            Constraint::Length(2),           // Spacing
            Constraint::Length(1),           // Phase indicator
            Constraint::Length(1),           // Session counter
            Constraint::Length(1),           // Stats line
            Constraint::Fill(1),             // Bottom padding
            Constraint::Length(1),           // Help text
        ])
        .split(area);

        // Render big timer
        let height = time_lines.len();
        let width = time_lines.first().map(|s| s.chars().count()).unwrap_or(0);

        let chunk = chunks[1];
        let text_width = width as u16;
        let start_x = chunk.x + (chunk.width.saturating_sub(text_width)) / 2;

        let buf = frame.buffer_mut();
        for (line_idx, line) in time_lines.iter().enumerate() {
            let y_pos = chunk.y + line_idx as u16;
            if y_pos >= chunk.y + chunk.height {
                break;
            }

            for (char_idx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }

                let x_pos = start_x + char_idx as u16;
                if x_pos >= chunk.x + chunk.width {
                    continue;
                }

                // Get base color
                let base_color = if self.color_theme.is_dynamic() {
                    self.color_theme
                        .color_at_position(char_idx, line_idx, width, height)
                } else {
                    color
                };

                // Apply animation
                let animated_color = apply_animation(
                    base_color,
                    self.animation_style,
                    self.animation_speed,
                    elapsed_ms,
                    char_idx,
                    width,
                    self.flash_intensity,
                );

                if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                    cell.set_char(ch);
                    cell.set_fg(animated_color);
                }
            }
        }

        // Render phase indicator
        let phase_str = if self.pomodoro_running {
            self.pomodoro_phase.display_name().to_string()
        } else {
            format!("{} (PAUSED)", self.pomodoro_phase.display_name())
        };
        let phase_chunk = chunks[3];
        let phase_width = phase_str.len() as u16;
        let phase_start_x = phase_chunk.x + (phase_chunk.width.saturating_sub(phase_width)) / 2;

        let buf = frame.buffer_mut();
        for (char_idx, ch) in phase_str.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x_pos = phase_start_x + char_idx as u16;
            if x_pos >= phase_chunk.x + phase_chunk.width {
                continue;
            }

            // Use different color for break phases
            let phase_color = if self.pomodoro_phase.is_break() {
                Color::Green
            } else {
                color
            };

            if let Some(cell) = buf.cell_mut(Position::new(x_pos, phase_chunk.y)) {
                cell.set_char(ch);
                cell.set_fg(phase_color);
            }
        }

        // Render session counter
        let session_str = format!(
            "Session {}/{}",
            self.pomodoro_sessions_completed + 1,
            self.config.pomodoro_sessions_until_long
        );
        let session_chunk = chunks[4];
        let session_width = session_str.len() as u16;
        let session_start_x =
            session_chunk.x + (session_chunk.width.saturating_sub(session_width)) / 2;

        let buf = frame.buffer_mut();
        for (char_idx, ch) in session_str.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x_pos = session_start_x + char_idx as u16;
            if x_pos >= session_chunk.x + session_chunk.width {
                continue;
            }

            if let Some(cell) = buf.cell_mut(Position::new(x_pos, session_chunk.y)) {
                cell.set_char(ch);
                cell.set_fg(Color::DarkGray);
            }
        }

        // Render stats line
        let total_mins = self.pomodoro_total_focus_secs / 60;
        let hours = total_mins / 60;
        let mins = total_mins % 60;

        let stats_str = if hours > 0 {
            format!(
                "Session {} | Total focus: {}h {:02}m",
                self.pomodoro_sessions_completed, hours, mins
            )
        } else {
            format!(
                "Session {} | Total focus: {}m",
                self.pomodoro_sessions_completed, mins
            )
        };

        let stats_chunk = chunks[5];
        let stats_width = stats_str.len() as u16;
        let stats_start_x =
            stats_chunk.x + (stats_chunk.width.saturating_sub(stats_width)) / 2;

        let buf = frame.buffer_mut();
        for (char_idx, ch) in stats_str.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x_pos = stats_start_x + char_idx as u16;
            if x_pos >= stats_chunk.x + stats_chunk.width {
                continue;
            }
            if let Some(cell) = buf.cell_mut(Position::new(x_pos, stats_chunk.y)) {
                cell.set_char(ch);
                cell.set_fg(Color::DarkGray);
            }
        }

        // Render pomodoro help text - skip in screensaver mode
        if !self.screensaver_mode {
            let help = Line::from(vec![
                "q".bold().fg(color),
                " quit  ".dark_gray(),
                "m".bold().fg(color),
                " mode  ".dark_gray(),
                "Space".bold().fg(color),
                " start/pause  ".dark_gray(),
                "r".bold().fg(color),
                " reset  ".dark_gray(),
                "n".bold().fg(color),
                " skip  ".dark_gray(),
                "?".bold().fg(color),
                " help".dark_gray(),
            ])
            .centered();
            frame.render_widget(help, chunks[7]);
        }
    }

    /// Render the countdown timer display.
    fn render_timer(&mut self, frame: &mut Frame, elapsed_ms: u64) {
        let color = self.color_theme.color();
        let area = frame.area();

        // Format time as MM:SS
        let mins = self.timer_remaining_secs / 60;
        let secs = self.timer_remaining_secs % 60;
        let time_str = format!("{mins:02}:{secs:02}");

        // Get current font and render
        let font = self.font_registry.get_or_default(&self.current_font);
        let time_lines = font.render_text(&time_str);
        let font_height = font.height as u16;

        // Create vertical layout for centering
        let chunks = Layout::vertical([
            Constraint::Fill(1),             // Top padding
            Constraint::Length(font_height), // Timer digits
            Constraint::Length(2),           // Spacing
            Constraint::Length(1),           // Status label
            Constraint::Fill(1),             // Bottom padding
            Constraint::Length(1),           // Help text
        ])
        .split(area);

        // Render big timer
        let height = time_lines.len();
        let width = time_lines.first().map(|s| s.chars().count()).unwrap_or(0);

        let chunk = chunks[1];
        let text_width = width as u16;
        let start_x = chunk.x + (chunk.width.saturating_sub(text_width)) / 2;

        let buf = frame.buffer_mut();
        for (line_idx, line) in time_lines.iter().enumerate() {
            let y_pos = chunk.y + line_idx as u16;
            if y_pos >= chunk.y + chunk.height {
                break;
            }

            for (char_idx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }

                let x_pos = start_x + char_idx as u16;
                if x_pos >= chunk.x + chunk.width {
                    continue;
                }

                let base_color = if self.color_theme.is_dynamic() {
                    self.color_theme
                        .color_at_position(char_idx, line_idx, width, height)
                } else {
                    color
                };

                let animated_color = apply_animation(
                    base_color,
                    self.animation_style,
                    self.animation_speed,
                    elapsed_ms,
                    char_idx,
                    width,
                    self.flash_intensity,
                );

                if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                    cell.set_char(ch);
                    cell.set_fg(animated_color);
                }
            }
        }

        // Render status label
        let label_str = if self.timer_completed {
            "TIME'S UP".to_string()
        } else {
            let dur_mins = self.timer_duration_secs / 60;
            let dur_secs = self.timer_duration_secs % 60;
            if self.timer_running {
                format!("{dur_mins:02}:{dur_secs:02} TIMER")
            } else {
                format!("{dur_mins:02}:{dur_secs:02} TIMER (PAUSED)")
            }
        };
        let label_chunk = chunks[3];
        let label_width = label_str.len() as u16;
        let label_start_x = label_chunk.x + (label_chunk.width.saturating_sub(label_width)) / 2;

        let label_color = if self.timer_completed {
            Color::Red
        } else {
            color
        };

        let buf = frame.buffer_mut();
        for (char_idx, ch) in label_str.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x_pos = label_start_x + char_idx as u16;
            if x_pos >= label_chunk.x + label_chunk.width {
                continue;
            }

            if let Some(cell) = buf.cell_mut(Position::new(x_pos, label_chunk.y)) {
                cell.set_char(ch);
                cell.set_fg(label_color);
            }
        }

        // Render timer help text - skip in screensaver mode
        if !self.screensaver_mode {
            let help = Line::from(vec![
                "q".bold().fg(color),
                " quit  ".dark_gray(),
                "m".bold().fg(color),
                " mode  ".dark_gray(),
                "Space".bold().fg(color),
                " start/pause  ".dark_gray(),
                "r".bold().fg(color),
                " reset  ".dark_gray(),
                "+/-".bold().fg(color),
                " duration  ".dark_gray(),
                "?".bold().fg(color),
                " help".dark_gray(),
            ])
            .centered();
            frame.render_widget(help, chunks[5]);
        }
    }

    /// Update flash intensity for reactive animation.
    fn update_flash(&mut self, now: &chrono::DateTime<chrono::Local>) {
        let second: u32 = now.format("%S").to_string().parse().unwrap_or(0);
        let minute: u32 = now.format("%M").to_string().parse().unwrap_or(0);
        let hour: u32 = now.format("%H").to_string().parse().unwrap_or(0);

        // Check for time changes and trigger flash
        if hour != self.last_hour {
            self.flash_intensity = 1.0; // Full flash for hour change
            self.flash_start = Some(Instant::now());
            self.last_hour = hour;
            self.last_minute = minute;
            self.last_second = second;
        } else if minute != self.last_minute {
            self.flash_intensity = 0.7; // Strong flash for minute change
            self.flash_start = Some(Instant::now());
            self.last_minute = minute;
            self.last_second = second;
        } else if second != self.last_second {
            self.flash_intensity = 0.3; // Subtle flash for second change
            self.flash_start = Some(Instant::now());
            self.last_second = second;
        }

        // Decay flash over time
        if let Some(flash_start) = self.flash_start {
            let decay_ms = self.animation_speed.flash_decay_ms();
            let flash_elapsed = flash_start.elapsed().as_millis() as f32;
            let decay_progress = (flash_elapsed / decay_ms as f32).min(1.0);
            self.flash_intensity *= 1.0 - decay_progress;

            if self.flash_intensity < 0.01 {
                self.flash_intensity = 0.0;
                self.flash_start = None;
            }
        }
    }

    /// Reads the crossterm events and updates the state of [`App`].
    /// Uses polling with timeout for real-time clock updates.
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        // Poll for events with 100ms timeout for smooth clock updates
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        // If settings dialog is visible, handle dialog keys
        if self.settings_dialog.visible {
            self.handle_settings_key(key);
            return;
        }

        // Toggle help overlay
        if key.code == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return;
        }
        // Dismiss help on any key if visible
        if self.show_help {
            self.show_help = false;
            return;
        }

        // Main app keybindings
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('m')) => self.toggle_display_mode(),
            (_, KeyCode::Char('t')) => self.toggle_time_format(),
            (_, KeyCode::Char('c')) => self.cycle_color_theme(),
            (_, KeyCode::Char('a')) => self.cycle_animation(),
            (_, KeyCode::Char('b')) => self.cycle_background(),
            (_, KeyCode::Char('s')) => self.open_settings(),
            // Pomodoro-specific keys (only active in pomodoro mode)
            (_, KeyCode::Char(' ')) if self.display_mode == DisplayMode::Pomodoro => {
                self.toggle_pomodoro()
            }
            (_, KeyCode::Char('r')) if self.display_mode == DisplayMode::Pomodoro => {
                self.reset_pomodoro()
            }
            (_, KeyCode::Char('n')) if self.display_mode == DisplayMode::Pomodoro => {
                self.skip_pomodoro_phase()
            }
            // Timer-specific keys (only active in timer mode)
            (_, KeyCode::Char(' ')) if self.display_mode == DisplayMode::Timer => {
                self.toggle_timer()
            }
            (_, KeyCode::Char('r')) if self.display_mode == DisplayMode::Timer => {
                self.reset_timer()
            }
            (_, KeyCode::Char('+') | KeyCode::Char('='))
                if self.display_mode == DisplayMode::Timer =>
            {
                self.adjust_timer_duration(1)
            }
            (_, KeyCode::Char('-')) if self.display_mode == DisplayMode::Timer => {
                self.adjust_timer_duration(-1)
            }
            // Stopwatch-specific keys
            (_, KeyCode::Char(' ')) if self.display_mode == DisplayMode::Stopwatch => {
                self.toggle_stopwatch()
            }
            (_, KeyCode::Char('r')) if self.display_mode == DisplayMode::Stopwatch => {
                self.reset_stopwatch()
            }
            (_, KeyCode::Char('l')) if self.display_mode == DisplayMode::Stopwatch => {
                self.lap_stopwatch()
            }
            _ => {}
        }
    }

    /// Handle key events when settings dialog is open.
    fn handle_settings_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.cancel_settings();
            }
            KeyCode::Enter => {
                self.save_settings();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings_dialog.prev_field();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings_dialog.next_field();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.settings_dialog.prev_value();
                self.apply_preview();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.settings_dialog.next_value();
                self.apply_preview();
            }
            _ => {}
        }
    }

    /// Apply current dialog values as live preview.
    fn apply_preview(&mut self) {
        self.current_font = self.settings_dialog.selected_font().to_string();
        self.color_theme = self.settings_dialog.color_theme;
        self.time_format = self.settings_dialog.time_format;
        self.animation_style = self.settings_dialog.animation_style;
        self.animation_speed = self.settings_dialog.animation_speed;
        self.colon_blink = self.settings_dialog.colon_blink;
        self.show_seconds = self.settings_dialog.show_seconds;
        self.background_style = self.settings_dialog.background_style;
        // Update pomodoro config (will take effect on next timer reset)
        self.config.pomodoro_work_mins = self.settings_dialog.pomodoro_work_mins;
        self.config.pomodoro_break_mins = self.settings_dialog.pomodoro_break_mins;
        self.config.pomodoro_long_break_mins = self.settings_dialog.pomodoro_long_break_mins;
        self.config.pomodoro_sound = self.settings_dialog.pomodoro_sound;
        self.config.desktop_notifications = self.settings_dialog.desktop_notifications;
        // Update timer duration
        let new_timer_mins = self.settings_dialog.timer_duration_mins;
        self.config.timer_duration_mins = new_timer_mins;
        self.timer_duration_secs = new_timer_mins * 60;
        self.update_background_monitors();
    }

    /// Open settings dialog with current settings.
    fn open_settings(&mut self) {
        self.settings_dialog.open(
            &self.current_font,
            self.color_theme,
            self.time_format,
            self.animation_style,
            self.animation_speed,
            self.colon_blink,
            self.show_seconds,
            self.background_style,
            self.config.pomodoro_work_mins,
            self.config.pomodoro_break_mins,
            self.config.pomodoro_long_break_mins,
            self.config.pomodoro_sound,
            self.config.desktop_notifications,
            self.config.timer_duration_mins,
        );
    }

    /// Save current settings to config file and close dialog.
    fn save_settings(&mut self) {
        // Update and save config (values already applied via preview)
        self.config.font_name = self.current_font.clone();
        self.config.color_theme = self.color_theme;
        self.config.time_format = self.time_format;
        self.config.animation_style = self.animation_style;
        self.config.animation_speed = self.animation_speed;
        self.config.colon_blink = self.colon_blink;
        self.config.show_seconds = self.show_seconds;
        self.config.background_style = self.background_style;
        self.config.desktop_notifications = self.settings_dialog.desktop_notifications;
        self.config.timer_duration_mins = self.settings_dialog.timer_duration_mins;

        if let Err(e) = self.config.save() {
            eprintln!("Warning: Failed to save config: {e}");
        }

        self.settings_dialog.close();
    }

    /// Cancel settings and revert to original values.
    fn cancel_settings(&mut self) {
        // Revert to original values
        self.current_font = self.settings_dialog.original_font().to_string();
        self.color_theme = self.settings_dialog.original_color_theme();
        self.time_format = self.settings_dialog.original_time_format();
        self.animation_style = self.settings_dialog.original_animation_style();
        self.animation_speed = self.settings_dialog.original_animation_speed();
        self.colon_blink = self.settings_dialog.original_colon_blink();
        self.show_seconds = self.settings_dialog.original_show_seconds();
        self.background_style = self.settings_dialog.original_background_style();
        self.config.pomodoro_work_mins = self.settings_dialog.original_pomodoro_work_mins();
        self.config.pomodoro_break_mins = self.settings_dialog.original_pomodoro_break_mins();
        self.config.pomodoro_long_break_mins =
            self.settings_dialog.original_pomodoro_long_break_mins();
        self.config.pomodoro_sound = self.settings_dialog.original_pomodoro_sound();
        self.config.desktop_notifications = self.settings_dialog.original_desktop_notifications();
        let orig_timer_mins = self.settings_dialog.original_timer_duration_mins();
        self.config.timer_duration_mins = orig_timer_mins;
        self.timer_duration_secs = orig_timer_mins * 60;
        self.update_background_monitors();

        self.settings_dialog.close();
    }

    /// Toggle between 12-hour and 24-hour time format.
    fn toggle_time_format(&mut self) {
        self.time_format = self.time_format.toggle();
    }

    /// Cycle through available color themes.
    fn cycle_color_theme(&mut self) {
        self.color_theme = self.color_theme.next();
    }

    /// Cycle through animation styles.
    fn cycle_animation(&mut self) {
        self.animation_style = self.animation_style.next();
    }

    /// Cycle through background styles.
    fn cycle_background(&mut self) {
        self.background_style = self.background_style.next();
        self.update_background_monitors();
    }

    /// Start or stop background monitors based on current background style.
    fn update_background_monitors(&mut self) {
        // System monitor for reactive backgrounds
        if self.background_style.is_reactive() && self.system_monitor.is_none() {
            let monitor = SystemMonitor::new();
            monitor.start();
            self.system_monitor = Some(monitor);
        } else if !self.background_style.is_reactive() && self.system_monitor.is_some() {
            self.system_monitor = None;
        }

        // Weather monitor for weather background
        if self.background_style.requires_weather() && self.weather_monitor.is_none() {
            let monitor = WeatherMonitor::new(self.config.weather_location.clone());
            monitor.start();
            self.weather_monitor = Some(monitor);
        } else if !self.background_style.requires_weather() && self.weather_monitor.is_some() {
            self.weather_monitor = None;
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    /// Cycle between clock, pomodoro, and timer display modes.
    fn toggle_display_mode(&mut self) {
        self.display_mode = self.display_mode.next();
    }

    /// Start or resume the pomodoro timer.
    fn start_pomodoro(&mut self) {
        self.pomodoro_running = true;
        self.pomodoro_last_tick = Instant::now();
        if self.pomodoro_phase == PomodoroPhase::Work && self.pomodoro_work_start.is_none() {
            self.pomodoro_work_start = Some(Instant::now());
        }
    }

    /// Pause the pomodoro timer.
    fn pause_pomodoro(&mut self) {
        self.pomodoro_running = false;
    }

    /// Toggle pomodoro timer start/pause.
    fn toggle_pomodoro(&mut self) {
        if self.pomodoro_running {
            self.pause_pomodoro();
        } else {
            self.start_pomodoro();
        }
    }

    /// Reset the pomodoro timer to the beginning of the current phase.
    fn reset_pomodoro(&mut self) {
        self.pomodoro_running = false;
        self.pomodoro_remaining_secs = match self.pomodoro_phase {
            PomodoroPhase::Work => self.config.pomodoro_work_mins * 60,
            PomodoroPhase::ShortBreak => self.config.pomodoro_break_mins * 60,
            PomodoroPhase::LongBreak => self.config.pomodoro_long_break_mins * 60,
        };
    }

    /// Skip to the next pomodoro phase.
    fn skip_pomodoro_phase(&mut self) {
        self.transition_pomodoro_phase();
    }

    /// Transition to the next pomodoro phase.
    fn transition_pomodoro_phase(&mut self) {
        // Track completed work session
        if self.pomodoro_phase == PomodoroPhase::Work {
            if let Some(start) = self.pomodoro_work_start.take() {
                self.pomodoro_total_focus_secs += start.elapsed().as_secs();
            } else {
                self.pomodoro_total_focus_secs += (self.config.pomodoro_work_mins * 60) as u64;
            }
        }

        match self.pomodoro_phase {
            PomodoroPhase::Work => {
                self.pomodoro_sessions_completed += 1;
                if self.pomodoro_sessions_completed >= self.config.pomodoro_sessions_until_long {
                    self.pomodoro_phase = PomodoroPhase::LongBreak;
                    self.pomodoro_remaining_secs = self.config.pomodoro_long_break_mins * 60;
                } else {
                    self.pomodoro_phase = PomodoroPhase::ShortBreak;
                    self.pomodoro_remaining_secs = self.config.pomodoro_break_mins * 60;
                }
            }
            PomodoroPhase::ShortBreak => {
                self.pomodoro_phase = PomodoroPhase::Work;
                self.pomodoro_remaining_secs = self.config.pomodoro_work_mins * 60;
            }
            PomodoroPhase::LongBreak => {
                self.pomodoro_sessions_completed = 0;
                self.pomodoro_phase = PomodoroPhase::Work;
                self.pomodoro_remaining_secs = self.config.pomodoro_work_mins * 60;
            }
        }
        // Trigger flash notification for phase transition
        self.flash_intensity = 1.0;
        self.flash_start = Some(Instant::now());
        // Ring terminal bell if enabled
        if self.config.pomodoro_sound {
            print!("\x07");
        }
        // Send desktop notification if enabled
        if self.config.desktop_notifications {
            let (title, body) = match self.pomodoro_phase {
                PomodoroPhase::Work => (
                    "Pomodoro - Work Time",
                    format!("Focus for {} minutes", self.config.pomodoro_work_mins),
                ),
                PomodoroPhase::ShortBreak => (
                    "Pomodoro - Short Break",
                    format!("Take a {} minute break", self.config.pomodoro_break_mins),
                ),
                PomodoroPhase::LongBreak => (
                    "Pomodoro - Long Break",
                    format!(
                        "Take a {} minute break! You've earned it.",
                        self.config.pomodoro_long_break_mins
                    ),
                ),
            };
            send_desktop_notification(title, &body);
        }
        self.run_on_complete_command();
        // Pause timer after transition (user must start manually)
        self.pomodoro_running = false;
    }

    /// Update the countdown timer (called each frame).
    fn update_timer(&mut self) {
        if !self.timer_running || self.timer_completed {
            return;
        }

        let elapsed = self.timer_last_tick.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let secs_elapsed = elapsed.as_secs() as u32;
            self.timer_last_tick = Instant::now();

            if self.timer_remaining_secs > secs_elapsed {
                self.timer_remaining_secs -= secs_elapsed;
            } else {
                self.timer_remaining_secs = 0;
                self.timer_running = false;
                self.timer_completed = true;
                // Trigger alarm
                self.flash_intensity = 1.0;
                self.flash_start = Some(Instant::now());
                if self.config.pomodoro_sound {
                    print!("\x07");
                }
                if self.config.desktop_notifications {
                    let dur_mins = self.timer_duration_secs / 60;
                    let dur_secs = self.timer_duration_secs % 60;
                    send_desktop_notification(
                        "Timer Complete",
                        &format!("{dur_mins:02}:{dur_secs:02} timer has finished"),
                    );
                }
                self.run_on_complete_command();
            }
        }
    }

    /// Run the on-complete shell command in a background thread if configured.
    fn run_on_complete_command(&self) {
        if let Some(ref cmd) = self.on_complete_command {
            std::thread::spawn({
                let cmd = cmd.clone();
                move || {
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .spawn();
                }
            });
        }
    }

    /// Toggle timer start/pause, or restart if completed.
    fn toggle_timer(&mut self) {
        if self.timer_completed {
            // Restart with same duration
            self.timer_remaining_secs = self.timer_duration_secs;
            self.timer_completed = false;
            self.timer_running = true;
            self.timer_last_tick = Instant::now();
        } else if self.timer_running {
            self.timer_running = false;
        } else {
            self.timer_running = true;
            self.timer_last_tick = Instant::now();
        }
    }

    /// Reset timer to configured duration.
    fn reset_timer(&mut self) {
        self.timer_running = false;
        self.timer_completed = false;
        self.timer_remaining_secs = self.timer_duration_secs;
    }

    /// Adjust timer duration by delta minutes. Clamps to 1–99 minutes.
    fn adjust_timer_duration(&mut self, delta: i32) {
        if self.timer_running {
            return;
        }
        let current_mins = (self.timer_duration_secs / 60) as i32;
        let new_mins = (current_mins + delta).clamp(1, 99) as u32;
        self.timer_duration_secs = new_mins * 60;
        self.timer_remaining_secs = self.timer_duration_secs;
        self.timer_completed = false;
        // Persist to config
        self.config.timer_duration_mins = new_mins;
        let _ = self.config.save();
    }

    /// Toggle stopwatch start/pause.
    fn toggle_stopwatch(&mut self) {
        if self.stopwatch_running {
            self.stopwatch_elapsed_ms += self.stopwatch_last_tick.elapsed().as_millis() as u64;
            self.stopwatch_running = false;
        } else {
            self.stopwatch_running = true;
            self.stopwatch_last_tick = Instant::now();
        }
    }

    /// Reset the stopwatch to zero.
    fn reset_stopwatch(&mut self) {
        self.stopwatch_running = false;
        self.stopwatch_elapsed_ms = 0;
        self.stopwatch_laps.clear();
    }

    /// Record a lap time.
    fn lap_stopwatch(&mut self) {
        if self.stopwatch_running {
            let current =
                self.stopwatch_elapsed_ms + self.stopwatch_last_tick.elapsed().as_millis() as u64;
            self.stopwatch_laps.push(current);
        }
    }

    /// Get current stopwatch elapsed time in milliseconds.
    fn get_stopwatch_elapsed(&self) -> u64 {
        if self.stopwatch_running {
            self.stopwatch_elapsed_ms + self.stopwatch_last_tick.elapsed().as_millis() as u64
        } else {
            self.stopwatch_elapsed_ms
        }
    }

    /// Render the stopwatch display.
    fn render_stopwatch(&mut self, frame: &mut Frame, elapsed_ms: u64) {
        let color = self.color_theme.color();
        let area = frame.area();

        let total_ms = self.get_stopwatch_elapsed();
        let total_secs = total_ms / 1000;
        let mins = (total_secs / 60) % 100;
        let secs = total_secs % 60;
        let centiseconds = (total_ms % 1000) / 10;

        let time_str = format!("{mins:02}:{secs:02}");
        let cs_str = format!(".{centiseconds:02}");

        let font = self.font_registry.get_or_default(&self.current_font);
        let time_lines = font.render_text(&time_str);
        let font_height = font.height as u16;

        let lap_count = self.stopwatch_laps.len().min(5);
        let lap_height = if lap_count > 0 {
            lap_count as u16 + 1
        } else {
            0
        };

        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(font_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(lap_height),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

        let height = time_lines.len();
        let width = time_lines.first().map(|s| s.chars().count()).unwrap_or(0);

        let chunk = chunks[1];
        let text_width = width as u16;
        let start_x = chunk.x + (chunk.width.saturating_sub(text_width)) / 2;

        let buf = frame.buffer_mut();
        for (line_idx, line) in time_lines.iter().enumerate() {
            let y_pos = chunk.y + line_idx as u16;
            if y_pos >= chunk.y + chunk.height {
                break;
            }
            for (char_idx, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x_pos = start_x + char_idx as u16;
                if x_pos >= chunk.x + chunk.width {
                    continue;
                }
                let base_color = if self.color_theme.is_dynamic() {
                    self.color_theme
                        .color_at_position(char_idx, line_idx, width, height)
                } else {
                    color
                };
                let animated_color = apply_animation(
                    base_color,
                    self.animation_style,
                    self.animation_speed,
                    elapsed_ms,
                    char_idx,
                    width,
                    self.flash_intensity,
                );
                if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                    cell.set_char(ch);
                    cell.set_fg(animated_color);
                }
            }
        }

        // Render centiseconds
        let cs_chunk = chunks[2];
        let cs_width = cs_str.len() as u16;
        let cs_start_x = cs_chunk.x + (cs_chunk.width.saturating_sub(cs_width)) / 2;
        let buf = frame.buffer_mut();
        for (char_idx, ch) in cs_str.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x_pos = cs_start_x + char_idx as u16;
            if x_pos >= cs_chunk.x + cs_chunk.width {
                continue;
            }
            if let Some(cell) = buf.cell_mut(Position::new(x_pos, cs_chunk.y)) {
                cell.set_char(ch);
                cell.set_fg(color);
            }
        }

        // Render status
        let status_str = if self.stopwatch_running {
            "RUNNING"
        } else if self.stopwatch_elapsed_ms == 0 {
            "STOPPED"
        } else {
            "PAUSED"
        };
        let status_chunk = chunks[3];
        let status_width = status_str.len() as u16;
        let status_start_x =
            status_chunk.x + (status_chunk.width.saturating_sub(status_width)) / 2;
        let status_color = if self.stopwatch_running {
            Color::Green
        } else {
            color
        };
        let buf = frame.buffer_mut();
        for (char_idx, ch) in status_str.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x_pos = status_start_x + char_idx as u16;
            if x_pos >= status_chunk.x + status_chunk.width {
                continue;
            }
            if let Some(cell) = buf.cell_mut(Position::new(x_pos, status_chunk.y)) {
                cell.set_char(ch);
                cell.set_fg(status_color);
            }
        }

        // Render lap times (last 5)
        if !self.stopwatch_laps.is_empty() {
            let lap_chunk = chunks[4];
            let total_laps = self.stopwatch_laps.len();
            let start_idx = total_laps.saturating_sub(5);
            let visible_laps: Vec<(usize, u64)> = self.stopwatch_laps[start_idx..]
                .iter()
                .enumerate()
                .map(|(i, &ms)| (start_idx + i, ms))
                .collect();

            for (row, (lap_idx, lap_ms)) in visible_laps.iter().enumerate() {
                let lap_mins = (*lap_ms / 1000 / 60) % 100;
                let lap_secs = (*lap_ms / 1000) % 60;
                let lap_cs = (*lap_ms % 1000) / 10;

                let delta_ms = if *lap_idx > 0 {
                    lap_ms - self.stopwatch_laps[lap_idx - 1]
                } else {
                    *lap_ms
                };
                let d_mins = (delta_ms / 1000 / 60) % 100;
                let d_secs = (delta_ms / 1000) % 60;
                let d_cs = (delta_ms % 1000) / 10;

                let lap_str = format!(
                    "Lap {:>2}: {:02}:{:02}.{:02}  (+{:02}:{:02}.{:02})",
                    lap_idx + 1,
                    lap_mins,
                    lap_secs,
                    lap_cs,
                    d_mins,
                    d_secs,
                    d_cs
                );

                let lap_text_width = lap_str.len() as u16;
                let lap_start_x =
                    lap_chunk.x + (lap_chunk.width.saturating_sub(lap_text_width)) / 2;
                let y_pos = lap_chunk.y + row as u16;
                if y_pos >= lap_chunk.y + lap_chunk.height {
                    break;
                }

                let buf = frame.buffer_mut();
                for (char_idx, ch) in lap_str.chars().enumerate() {
                    if ch == ' ' {
                        continue;
                    }
                    let x_pos = lap_start_x + char_idx as u16;
                    if x_pos >= lap_chunk.x + lap_chunk.width {
                        continue;
                    }
                    if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                        cell.set_char(ch);
                        cell.set_fg(Color::DarkGray);
                    }
                }
            }
        }

        // Render stopwatch help text
        if !self.screensaver_mode {
            let help = Line::from(vec![
                "q".bold().fg(color),
                " quit  ".dark_gray(),
                "m".bold().fg(color),
                " mode  ".dark_gray(),
                "Space".bold().fg(color),
                " start/pause  ".dark_gray(),
                "r".bold().fg(color),
                " reset  ".dark_gray(),
                "l".bold().fg(color),
                " lap  ".dark_gray(),
                "?".bold().fg(color),
                " help".dark_gray(),
            ])
            .centered();
            frame.render_widget(help, chunks[6]);
        }
    }

    /// Render the world clock display showing multiple timezone clocks.
    fn render_world_clock(&mut self, frame: &mut Frame, elapsed_ms: u64) {
        let color = self.color_theme.color();
        let area = frame.area();
        let now = chrono::Utc::now();

        let entries = &self.world_clock_entries;
        if entries.is_empty() {
            return;
        }

        let font = self.font_registry.get_or_default(&self.current_font);
        let font_height = font.height as u16;

        // Each zone needs: 1 line for label + font_height for time + 1 line spacing
        let zone_height = font_height + 2;
        let total_content_height = zone_height * entries.len() as u16;

        let help_height = if self.screensaver_mode { 0u16 } else { 1 };

        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(total_content_height),
            Constraint::Fill(1),
            Constraint::Length(help_height),
        ])
        .split(area);

        let content_area = chunks[1];

        for (idx, (label, tz_name)) in entries.iter().enumerate() {
            let zone_y = content_area.y + (idx as u16 * zone_height);
            if zone_y + zone_height > content_area.y + content_area.height {
                break;
            }

            // Try to parse the timezone and get time
            let time_str = if let Ok(tz) = tz_name.parse::<chrono_tz::Tz>() {
                let local_time = now.with_timezone(&tz);
                match (self.time_format, self.show_seconds) {
                    (TimeFormat::TwentyFourHour, true) => {
                        local_time.format("%H:%M:%S").to_string()
                    }
                    (TimeFormat::TwentyFourHour, false) => {
                        local_time.format("%H:%M").to_string()
                    }
                    (TimeFormat::TwelveHour, true) => {
                        local_time.format("%I:%M:%S %p").to_string()
                    }
                    (TimeFormat::TwelveHour, false) => {
                        local_time.format("%I:%M %p").to_string()
                    }
                }
            } else {
                "??:??".to_string()
            };

            // Render label centered
            let label_y = zone_y;
            let label_width = label.len() as u16;
            let label_start_x =
                content_area.x + (content_area.width.saturating_sub(label_width)) / 2;

            let buf = frame.buffer_mut();
            for (char_idx, ch) in label.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x_pos = label_start_x + char_idx as u16;
                if x_pos >= content_area.x + content_area.width {
                    continue;
                }
                if let Some(cell) = buf.cell_mut(Position::new(x_pos, label_y)) {
                    cell.set_char(ch);
                    cell.set_fg(Color::DarkGray);
                }
            }

            // Render time in FIGlet font
            let time_lines = font.render_text(&time_str);
            let width = time_lines.first().map(|s| s.chars().count()).unwrap_or(0);
            let height = time_lines.len();
            let text_width = width as u16;
            let start_x = content_area.x + (content_area.width.saturating_sub(text_width)) / 2;
            let time_y = zone_y + 1;

            let buf = frame.buffer_mut();
            for (line_idx, line) in time_lines.iter().enumerate() {
                let y_pos = time_y + line_idx as u16;
                if y_pos >= content_area.y + content_area.height {
                    break;
                }
                for (char_idx, ch) in line.chars().enumerate() {
                    if ch == ' ' {
                        continue;
                    }
                    let x_pos = start_x + char_idx as u16;
                    if x_pos >= content_area.x + content_area.width {
                        continue;
                    }
                    let base_color = if self.color_theme.is_dynamic() {
                        self.color_theme
                            .color_at_position(char_idx, line_idx, width, height)
                    } else {
                        color
                    };
                    let animated_color = apply_animation(
                        base_color,
                        self.animation_style,
                        self.animation_speed,
                        elapsed_ms,
                        char_idx,
                        width,
                        self.flash_intensity,
                    );
                    if let Some(cell) = buf.cell_mut(Position::new(x_pos, y_pos)) {
                        cell.set_char(ch);
                        cell.set_fg(animated_color);
                    }
                }
            }
        }

        // Render help text
        if !self.screensaver_mode {
            let help = Line::from(vec![
                "q".bold().fg(color),
                " quit  ".dark_gray(),
                "m".bold().fg(color),
                " mode  ".dark_gray(),
                "t".bold().fg(color),
                " 12/24h  ".dark_gray(),
                "c".bold().fg(color),
                " color  ".dark_gray(),
                "s".bold().fg(color),
                " settings  ".dark_gray(),
                "?".bold().fg(color),
                " help".dark_gray(),
            ])
            .centered();
            frame.render_widget(help, chunks[3]);
        }
    }

    /// Render the help overlay showing all keybindings.
    fn render_help_overlay(&self, frame: &mut Frame) {
        if !self.show_help {
            return;
        }

        let area = frame.area();
        let width = 56u16.min(area.width.saturating_sub(4));
        let height = 28u16.min(area.height.saturating_sub(2));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay_area = ratatui::layout::Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay_area);

        let help_lines = vec![
            Line::from("Keyboard Shortcuts".bold()).centered(),
            Line::from(""),
            Line::from(vec!["  Global".bold().fg(Color::Yellow)]),
            Line::from(vec!["    q / Esc     ".bold(), "Quit".into()]),
            Line::from(vec![
                "    m           ".bold(),
                "Cycle mode (Clock/Pomodoro/Timer/Stopwatch/World)".into(),
            ]),
            Line::from(vec!["    t           ".bold(), "Toggle 12/24 hour".into()]),
            Line::from(vec!["    c           ".bold(), "Cycle color theme".into()]),
            Line::from(vec![
                "    a           ".bold(),
                "Cycle animation style".into(),
            ]),
            Line::from(vec!["    b           ".bold(), "Cycle background".into()]),
            Line::from(vec!["    s           ".bold(), "Open settings".into()]),
            Line::from(vec!["    ?           ".bold(), "Toggle this help".into()]),
            Line::from(""),
            Line::from(vec!["  Pomodoro".bold().fg(Color::Yellow)]),
            Line::from(vec!["    Space       ".bold(), "Start / Pause".into()]),
            Line::from(vec![
                "    r           ".bold(),
                "Reset current phase".into(),
            ]),
            Line::from(vec![
                "    n           ".bold(),
                "Skip to next phase".into(),
            ]),
            Line::from(""),
            Line::from(vec!["  Timer".bold().fg(Color::Yellow)]),
            Line::from(vec!["    Space       ".bold(), "Start / Pause".into()]),
            Line::from(vec!["    r           ".bold(), "Reset".into()]),
            Line::from(vec!["    + / -       ".bold(), "Adjust duration".into()]),
            Line::from(""),
            Line::from(vec!["  Stopwatch".bold().fg(Color::Yellow)]),
            Line::from(vec!["    Space       ".bold(), "Start / Pause".into()]),
            Line::from(vec!["    r           ".bold(), "Reset".into()]),
            Line::from(vec!["    l           ".bold(), "Lap".into()]),
            Line::from(""),
            Line::from("Press any key to close".dark_gray()).centered(),
        ];

        let help_widget = Paragraph::new(help_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(ratatui::style::Style::default().fg(Color::DarkGray))
                .title(" Help ")
                .title_alignment(ratatui::layout::Alignment::Center)
                .style(ratatui::style::Style::default().bg(Color::Black)),
        );

        frame.render_widget(help_widget, overlay_area);
    }

    /// Update the pomodoro timer (called each frame).
    fn update_pomodoro(&mut self) {
        if !self.pomodoro_running {
            return;
        }

        let elapsed = self.pomodoro_last_tick.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let secs_elapsed = elapsed.as_secs() as u32;
            self.pomodoro_last_tick = Instant::now();

            if self.pomodoro_remaining_secs > secs_elapsed {
                self.pomodoro_remaining_secs -= secs_elapsed;
            } else {
                self.pomodoro_remaining_secs = 0;
                self.transition_pomodoro_phase();
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a string into a ColorTheme (case-insensitive).
fn parse_color_theme(s: &str) -> Option<ColorTheme> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "cyan" => Some(ColorTheme::Cyan),
        "green" => Some(ColorTheme::Green),
        "white" => Some(ColorTheme::White),
        "magenta" => Some(ColorTheme::Magenta),
        "yellow" => Some(ColorTheme::Yellow),
        "red" => Some(ColorTheme::Red),
        "blue" => Some(ColorTheme::Blue),
        "rainbow" => Some(ColorTheme::Rainbow),
        "rainbowvertical" | "rainbow-vertical" => Some(ColorTheme::RainbowVertical),
        "gradientwarm" | "gradient-warm" | "warm" => Some(ColorTheme::GradientWarm),
        "gradientcool" | "gradient-cool" | "cool" => Some(ColorTheme::GradientCool),
        "gradientocean" | "gradient-ocean" | "ocean" => Some(ColorTheme::GradientOcean),
        "gradientneon" | "gradient-neon" | "neon" => Some(ColorTheme::GradientNeon),
        "gradientfire" | "gradient-fire" | "fire" => Some(ColorTheme::GradientFire),
        "gradientfrost" | "gradient-frost" | "frost" => Some(ColorTheme::GradientFrost),
        "gradientaurora" | "gradient-aurora" | "aurora" => Some(ColorTheme::GradientAurora),
        "gradientwinter" | "gradient-winter" | "winter" => Some(ColorTheme::GradientWinter),
        "gradientsakura" | "gradient-sakura" | "sakura" => Some(ColorTheme::GradientSakura),
        _ => None,
    }
}

/// Parse a string into a BackgroundStyle (case-insensitive).
fn parse_background_style(s: &str) -> Option<BackgroundStyle> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "none" => Some(BackgroundStyle::None),
        "starfield" | "stars" => Some(BackgroundStyle::Starfield),
        "matrix" | "matrixrain" | "matrix-rain" => Some(BackgroundStyle::MatrixRain),
        "gradient" | "gradientwave" | "gradient-wave" => Some(BackgroundStyle::GradientWave),
        "snowfall" | "snow" => Some(BackgroundStyle::Snowfall),
        "frost" => Some(BackgroundStyle::Frost),
        "aurora" => Some(BackgroundStyle::Aurora),
        "sunny" | "sun" => Some(BackgroundStyle::Sunny),
        "rainy" | "rain" => Some(BackgroundStyle::Rainy),
        "stormy" | "storm" => Some(BackgroundStyle::Stormy),
        "windy" | "wind" => Some(BackgroundStyle::Windy),
        "cloudy" | "clouds" => Some(BackgroundStyle::Cloudy),
        "foggy" | "fog" => Some(BackgroundStyle::Foggy),
        "weather" => Some(BackgroundStyle::Weather),
        "dawn" | "twilightdawn" | "twilight-dawn" => Some(BackgroundStyle::TwilightDawn),
        "dusk" | "twilightdusk" | "twilight-dusk" => Some(BackgroundStyle::TwilightDusk),
        "cherryblossom" | "cherry-blossom" | "sakura" => Some(BackgroundStyle::CherryBlossom),
        "systempulse" | "system-pulse" | "pulse" => Some(BackgroundStyle::SystemPulse),
        "resourcewave" | "resource-wave" | "resource" => Some(BackgroundStyle::ResourceWave),
        "dataflow" | "data-flow" => Some(BackgroundStyle::DataFlow),
        "heatmap" | "heat-map" | "heat" => Some(BackgroundStyle::HeatMap),
        _ => None,
    }
}

/// Parse a string into a DisplayMode (case-insensitive).
fn parse_display_mode(s: &str) -> Option<DisplayMode> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "clock" => Some(DisplayMode::Clock),
        "pomodoro" => Some(DisplayMode::Pomodoro),
        "timer" => Some(DisplayMode::Timer),
        "stopwatch" => Some(DisplayMode::Stopwatch),
        "worldclock" | "world-clock" | "world" => Some(DisplayMode::WorldClock),
        _ => None,
    }
}
