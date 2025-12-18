Output asset/demo.gif

Set FontSize 16
Set Width 1600
Set Height 1000
Set Padding 20
Set FontFamily "Menlo"
Set TypingSpeed 120ms
Set WindowBar Colorful

# Intro
Sleep 1s


# 1. Launch TUI
Type "twg samples/cloud_infrastructure.json"
Sleep 1s
Enter
Sleep 2s

# 0. Setup
Set TypingSpeed 120ms

# 1. Basic Navigation
Down
Sleep 1s
Right
Sleep 1s
Down
Sleep 1s
Down
Sleep 1s
Right
Sleep 1.5s
Down
Sleep 1s
Right
Sleep 1.5s
# Exploring Subnets
Down
Sleep 1s
Down
Sleep 1s
Right
Sleep 1.5s
Down
Sleep 3s

# 2. Search
Type "/"
Sleep 2s
Type "running"
Sleep 2s
Enter
Sleep 2s

# 3. Next/Prev Match
Type "n"
Sleep 2s
Type "n"
Sleep 2s
Type "N"
Sleep 2s

# 4. Smart Jump (Path)
Type "/"
Sleep 1s
Type '.regions["us-east-1"].vpcs[0].subnets[1].instances[0].name'
Sleep 1s
Enter
Sleep 3s

# 5. Themes
Type "t"
Sleep 2s
Type "t"
Sleep 2s

# 6. Copy
Type "c"
Sleep 2s

# Quit
Type "q"
Sleep 1s
