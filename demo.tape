Output asset/demo.gif

Set Shell bash
Set FontSize 14
Set Width 1600
Set Height 1000
Set Padding 24
Set FontFamily "JetBrains Mono"
Set TypingSpeed 60ms
Set WindowBar Colorful
Set CursorBlink false

# --- Scene 1: open the JSON sample, navigate Miller columns ---
Type "twig samples/cloud_infrastructure.json"
Enter
Sleep 2.5s

# Drill down: cloud_provider -> regions
Down
Sleep 200ms
Down
Sleep 200ms
Right
Sleep 1.5s

# Continue drilling: us-east-1
Right
Sleep 1s

# Further drill into VPCs
Down
Sleep 200ms
Right
Sleep 1.5s

# Search across keys and values
Type "/"
Sleep 400ms
Type "subnet"
Sleep 600ms
Enter
Sleep 2s

# Exit JSON
Type "q"
Sleep 800ms

# --- Scene 2: open a HAR file, show URL detection in inspector ---
Type "clear"
Enter
Sleep 200ms
Type "twig samples/browser_navigation.har"
Enter
Sleep 2.5s

# Drill into entries -> request
Down
Sleep 200ms
Down
Sleep 200ms
Right
Sleep 800ms
Down
Sleep 200ms
Right
Sleep 800ms
Down
Sleep 200ms
Right
Sleep 1.5s

# Cycle through themes for visual variety
Type "t"
Sleep 1s

# Final frame
Type "q"
Sleep 400ms
