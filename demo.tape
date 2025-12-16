Output asset/demo.gif

Set FontSize 16
Set Width 1200
Set Height 800
Set Padding 20
Set FontFamily "Menlo"
Set TypingSpeed 120ms
Set WindowBar Colorful

# Intro
Sleep 1s

# 0. Version Check
Type "twig --version"
Sleep 1s
Enter
Sleep 2s

# 0.1 Show Invalid JSON
Type "cat samples/unformatted_invalid_config.json"
Sleep 1s
Enter
Sleep 2s

# 0.2 Fix and Print
Type "twig --fix --print samples/unformatted_invalid_config.json"
Sleep 1s
Enter
Sleep 3s

# 0.3 Navigation Demo
Type "twig samples/cloud_infrastructure.json"
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
Type "api-gateway"
Sleep 2s
Enter
Sleep 5s

# 3. Smart Search
Type "/"
Sleep 2s
# Jump to a specific instance in private subnet
Type '.regions["us-east-1"].vpcs[0].subnets[1].instances[0].name'
Sleep 3s
Enter
Sleep 4s

# 4. next/Prev
Type "/"
Sleep 2s
Type "available"
Sleep 2s
Enter
Sleep 2s
Type "n"
Sleep 2s
Type "n"
Sleep 2s
Type "N"
Sleep 2s

# 5. Help
Type "?"
Sleep 3s
Escape
Sleep 1s

# 6. Themes
Type "t"
Sleep 2s
Type "t"
Sleep 2s

# 7. Copy
Type "c"
Sleep 2s

# Quit
Type "q"
Sleep 1s
