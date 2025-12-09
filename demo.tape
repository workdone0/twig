Output asset/demo.gif

Set FontSize 16
Set Width 1200
Set Height 800
Set Padding 20
Set FontFamily "Menlo"

Type "twg samples/complex.json"
Sleep 1s
Enter

Sleep 2s

# 0. Setup
Set TypingSpeed 100ms

# 1. Basic Navigation (Explore Exhibits)
Down
Sleep 0.5s
Down
Sleep 0.5s
Down
Sleep 0.5s
Right
Sleep 1s
Down
Sleep 0.5s
Right
Sleep 1s
Down
Sleep 0.5s
Down
Sleep 2s

# 2. Search (Find 'Leopard')
Type "/"
Sleep 1.5s
Type "Leopard"
Sleep 1.5s
Enter
Sleep 3s

# 3. Smart Search (Path Jump)
Type "/"
Sleep 1.5s
Type ".exhibits[0].animals[5].name"
Sleep 1.5s
Enter
Sleep 2s

# 4. Search Navigation (Next/Prev)
Type "/"
Sleep 1s
Type "Lion"
Sleep 1s
Enter
Sleep 1.5s
Type "n"
Sleep 1s
Type "n"
Sleep 1s
Type "N"
Sleep 1s

# 5. Inline Help
Type "?"
Sleep 4s
Escape
Sleep 1.5s

# 6. Theme Toggle
Type "t"
Sleep 3s
Type "t"
Sleep 3s

# 7. Copy Path
Type "c"
Sleep 2s

# Quit
Type "q"
Sleep 1s
