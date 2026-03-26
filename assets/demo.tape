# VHS tape file for sigye demo - Full Feature Showcase
# https://github.com/charmbracelet/vhs
#
# Prerequisites: brew install vhs && cargo build --release
# Usage: cd assets && vhs demo.tape

Output demo.gif

Set Shell "zsh"
Set FontSize 16
Set Width 1400
Set Height 750
Set Theme "Dracula"
Set TypingSpeed 0ms
Set Padding 10
Set PlaybackSpeed 1

# ─────────────────────────────────────────────────
# SCENE 1: Aurora + Neon — the hero shot
# ─────────────────────────────────────────────────
Hide
Type "../target/release/sigye --bg aurora --theme neon"
Enter
Sleep 2s
Show
Sleep 3s
Type "q"
Sleep 300ms

# ─────────────────────────────────────────────────
# SCENE 2: Matrix Rain + Fire theme
# ─────────────────────────────────────────────────
Hide
Type "../target/release/sigye --bg matrix --theme fire --font Doom"
Enter
Sleep 2s
Show
Sleep 3s
Type "q"
Sleep 300ms

# ─────────────────────────────────────────────────
# SCENE 3: Cherry Blossom + Sakura theme
# ─────────────────────────────────────────────────
Hide
Type "../target/release/sigye --bg sakura --theme sakura"
Enter
Sleep 2s
Show
Sleep 3s
Type "q"
Sleep 300ms

# ─────────────────────────────────────────────────
# SCENE 4: Pomodoro with starfield + animation
# ─────────────────────────────────────────────────
Hide
Type "../target/release/sigye --mode pomodoro --bg starfield --theme rainbow"
Enter
Sleep 2s
Show
Type "a"
Sleep 3s
Type "q"
Sleep 300ms

# ─────────────────────────────────────────────────
# SCENE 5: Stopwatch with laps
# ─────────────────────────────────────────────────
Hide
Type "../target/release/sigye --mode stopwatch --bg aurora --theme ocean"
Enter
Sleep 2s
Show
Type " "
Sleep 1.5s
Type "l"
Sleep 1.5s
Type "l"
Sleep 1s
Type " "
Sleep 1s
Type "q"
Sleep 300ms

# ─────────────────────────────────────────────────
# SCENE 6: World Clock
# ─────────────────────────────────────────────────
Hide
Type "../target/release/sigye --mode worldclock --bg starfield --theme cool"
Enter
Sleep 2s
Show
Sleep 3s
Type "q"
Sleep 300ms

# ─────────────────────────────────────────────────
# SCENE 7: Help overlay
# ─────────────────────────────────────────────────
Hide
Type "../target/release/sigye --bg aurora --theme neon"
Enter
Sleep 2s
Show
Sleep 500ms
Type "?"
Sleep 2.5s
Type " "
Sleep 1s

Type "q"
Sleep 300ms
