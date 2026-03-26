# VHS tape file for sigye - Screensaver showcase
# Perfect for r/unixporn
#
# Prerequisites: brew install vhs && cargo build --release
# Usage: cd assets && vhs demo-screensaver.tape

Output demo-screensaver.gif

Set Shell "zsh"
Set FontSize 20
Set Width 1400
Set Height 800
Set Theme "Dracula"
Set TypingSpeed 0ms
Set Padding 0

# Aurora + Neon
Hide
Type "../target/release/sigye --screensaver --bg aurora --theme neon --font Doom"
Enter
Sleep 2s
Show
Sleep 5s
Type "q"
Sleep 300ms

# Matrix + Fire
Hide
Type "../target/release/sigye --screensaver --bg matrix --theme fire --font Doom"
Enter
Sleep 2s
Show
Sleep 5s
Type "q"
Sleep 300ms

# Cherry Blossom + Sakura
Hide
Type "../target/release/sigye --screensaver --bg sakura --theme sakura"
Enter
Sleep 2s
Show
Sleep 5s
Type "q"
Sleep 300ms

# Snowfall + Frost
Hide
Type "../target/release/sigye --screensaver --bg snow --theme frost --font Doom"
Enter
Sleep 2s
Show
Sleep 5s

Type "q"
Sleep 200ms
