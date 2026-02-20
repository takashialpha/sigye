# sigye (시계)

[![Crates.io](https://img.shields.io/crates/v/sigye)](https://crates.io/crates/sigye)
[![License](https://img.shields.io/crates/l/sigye)](https://github.com/am2rican5/sigye/blob/main/LICENSE)
[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

A terminal clock with ASCII art fonts, animated backgrounds, and a built-in Pomodoro timer. The name "sigye" (시계) means "clock" in Korean.

![sigye demo](assets/demo.gif)

## Features

- **3 display modes** — Clock, Pomodoro timer, and countdown timer
- **40 bundled FIGlet fonts** — From classic Standard to stylish Star Wars
- **18 color themes** — Static colors, rainbow gradients, and seasonal palettes
- **20 animated backgrounds** — Starfield, matrix rain, weather effects, cherry blossoms, and system-reactive visuals
- **4 animation styles** — Shifting, pulsing, wave, and reactive clock effects
- **Live weather backgrounds** — Auto-selects rain, snow, fog, or sun based on real conditions via wttr.in
- **System-reactive backgrounds** — CPU, memory, network, and disk metrics drive visual effects
- **Blinking colon** — Optional colon separator animation
- **12/24 hour format** — Toggle with a single keypress
- **Live settings preview** — See changes in real time before saving
- **Persistent configuration** — Settings saved automatically to TOML
- **Custom font support** — Drop FIGlet `.flf` files into `~/.config/sigye/fonts/`

## Installation

### From crates.io

```bash
cargo install sigye
```

### From source

```bash
git clone https://github.com/am2rican5/sigye
cd sigye
cargo install --path crates/sigye
```

## Usage

```bash
sigye
```

Press `s` to open the settings dialog, or use the keybindings below to adjust on the fly.

## Keybindings

### Clock

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `m` | Cycle display mode (Clock / Pomodoro / Timer) |
| `t` | Toggle 12/24 hour format |
| `c` | Cycle color theme |
| `a` | Cycle animation style |
| `b` | Cycle background style |
| `s` | Open settings dialog |

### Pomodoro Mode

| Key | Action |
|-----|--------|
| `Space` | Pause / resume |
| `r` | Reset current phase |
| `n` | Skip to next phase |

### Timer Mode

| Key | Action |
|-----|--------|
| `Space` | Pause / resume |
| `r` | Reset timer |
| `+` / `=` | Add time |
| `-` | Subtract time |

### Settings Dialog

| Key | Action |
|-----|--------|
| `↑` / `k` | Previous field |
| `↓` / `j` | Next field |
| `←` / `h` | Previous value |
| `→` / `l` | Next value |
| `Enter` | Save settings |
| `Esc` | Cancel |

## Configuration

Settings are stored at `~/.config/sigye/config.toml`:

```toml
font_name = "Standard"
color_theme = "Cyan"
time_format = "TwentyFourHour"
animation_style = "None"
animation_speed = "Medium"
colon_blink = false
show_seconds = true
background_style = "None"
weather_location = ""          # Empty for auto-detect, or city name (e.g., "Seoul")
pomodoro_work_mins = 25
pomodoro_break_mins = 5
pomodoro_long_break_mins = 15
pomodoro_sessions_until_long = 4
pomodoro_sound = true
```

### Custom Fonts

Place FIGlet font files (`.flf`) in `~/.config/sigye/fonts/` and they will appear in the settings dialog.

## Color Themes

### Static Colors

Cyan, Green, White, Magenta, Yellow, Red, Blue

### Dynamic Gradients

| Theme | Description |
|-------|-------------|
| Rainbow | Horizontal rainbow spectrum |
| Rainbow V | Vertical rainbow |
| Warm | Red to Orange to Yellow |
| Cool | Blue to Cyan to Green |
| Ocean | Dark Blue to Cyan to Teal |
| Neon | Magenta to Cyan (synthwave) |
| Fire | Dark Red to Orange to Yellow |
| Frost | White to Ice Blue to Steel Blue |
| Aurora | Green to Cyan to Blue to Purple |
| Winter | Deep Blue to Royal Blue to Ice Blue |
| Sakura | Sakura Pink to Lavender Blush |

## Background Styles

### Classic

| Style | Description |
|-------|-------------|
| Starfield | Twinkling stars with varying brightness |
| Matrix | Falling green Matrix-style characters |
| Gradient | Flowing diagonal color wave |

### Weather & Atmospheric

| Style | Description |
|-------|-------------|
| Weather | Auto-selects based on real-time conditions via wttr.in |
| Sunny | Radiant sun with animated rays |
| Cloudy | Layered drifting clouds |
| Foggy | Ground-hugging mist effect |
| Rainy | Falling rain droplets |
| Stormy | Rain with lightning flashes |
| Windy | Horizontal wind streaks |
| Snowfall | Drifting snowflakes in shades of blue |
| Frost | Ice crystals growing from screen edges |
| Aurora | Northern lights in green, cyan, blue, and purple |

### Twilight & Seasonal

| Style | Description |
|-------|-------------|
| Dawn | Sunrise gradient with fading stars |
| Dusk | Sunset gradient with emerging stars |
| Sakura | Cherry blossom petals drifting down |

### System-Reactive

Driven by real-time system metrics (CPU, memory, network, disk):

| Style | Description |
|-------|-------------|
| Sys Pulse | CPU usage drives pulsing rings from center |
| Resource | Memory usage controls wave amplitude |
| Data Flow | Network I/O drives particle density |
| Heat Map | Combined metrics as color intensity |

## Bundled Fonts

3D-ASCII, Acrobatic, Alligator, Alphabet, ANSI Regular, ANSI Shadow, Avatar, Banner, Bell, Big, Big Money-ne, Block, BlurVision ASCII, Chunky, Colossal, Doh, Doom, Electronic, Epic, Graffiti, Ivrit, Larry 3D, Lean, Mini, Mono 9, Mono 12, Ogre, Poison, Puffy, Rebel, Rectangles, Script, Shadow, Slant, Small, Speed, Standard, Star Wars, Terrace, Tmplr

## Contributing

```bash
git clone https://github.com/am2rican5/sigye
cd sigye
cargo build
cargo test --workspace
cargo fmt -- --check
cargo clippy
```

## License

Copyright (c) am2rican5

This project is licensed under the MIT license ([LICENSE](./LICENSE) or <http://opensource.org/licenses/MIT>)
