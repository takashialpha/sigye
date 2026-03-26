# VHS tape file for sigye - Quick punchy demo (~20s)
# Best for social media sharing
#
# Prerequisites: brew install vhs && cargo build --release
# Usage: cd assets && vhs demo-quick.tape

Output demo-quick.gif

Set Shell "zsh"
Set FontSize 18
Set Width 1400
Set Height 750
Set Theme "Dracula"
Set TypingSpeed 0ms
Set Padding 10

# Hero shot: Aurora + Neon + Doom font
Hide
Type "../target/release/sigye --bg aurora --theme neon --font Doom"
Enter
Sleep 2s
Show
Sleep 4s
Type "q"
Sleep 300ms

# Matrix + Fire
Hide
Type "../target/release/sigye --bg matrix --theme fire --font Doom"
Enter
Sleep 2s
Show
Sleep 3s
Type "q"
Sleep 300ms

# World clock
Hide
Type "../target/release/sigye --mode worldclock --bg starfield --theme ocean"
Enter
Sleep 2s
Show
Sleep 4s
Type "q"
Sleep 300ms

# Screensaver mode
Hide
Type "../target/release/sigye --screensaver --bg aurora --theme neon --font Doom"
Enter
Sleep 2s
Show
Sleep 3s

Type "q"
Sleep 200ms
