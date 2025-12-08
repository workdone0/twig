Output asset/demo.gif

Set FontSize 16
Set Width 1200
Set Height 800
Set Padding 20
Set FontFamily "Menlo"

Type "twg samples/tweets.json"
Sleep 1s
Enter

Sleep 2s

# 0. Setup
Set TypingSpeed 100ms

# 1. Basic Navigation
Down
Sleep 1s
Right
Sleep 1s
Right
Sleep 1s
Down
Sleep 1s
Down
Sleep 2s

# 2. Search
Type "/"
Sleep 1.5s
Type "created_at"
Sleep 1.5s
Enter
Sleep 3s

# 3. Jump to Path
Type ":"
Sleep 1.5s
Type ".includes.tweets"
Sleep 1.5s
Enter
Sleep 3s

# 4. Inline Help
Type "?"
Sleep 4s
Escape
Sleep 1.5s

# 5. Theme Toggle
Type "t"
Sleep 3s
Type "t"
Sleep 3s

# 6. Copy Path
Type "c"
Sleep 2s

# Quit
Type "q"
Sleep 1s
