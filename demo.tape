Output asset/demo.gif

Set Shell bash
Set FontSize 14
Set Width 1200
Set Height 640
Set Padding 0
Set FontFamily "Fira Code"
Set TypingSpeed 50ms
Set WindowBar Colorful
Set CursorBlink false

# --- Scene 1: open the JSON sample ---
Type "twig samples/cloud_infrastructure.json"
Enter
Sleep 3s

# Drill down into the structure
Down
Sleep 250ms
Down
Sleep 250ms
Right
Sleep 2s

Right
Sleep 2s

Down
Sleep 250ms
Right
Sleep 2s

# Search for "subnet"
Type "/"
Sleep 400ms
Type "subnet"
Sleep 800ms
Enter
Sleep 3s

# Exit JSON
Type "q"
Sleep 800ms

# --- Scene 2: open the HAR file (URL detection in inspector) ---
Type "clear"
Enter
Sleep 300ms
Type "twig samples/browser_navigation.har"
Enter
Sleep 3s

# Drill into entries -> request
Down
Sleep 250ms
Down
Sleep 250ms
Down
Sleep 250ms
Right
Sleep 1.2s
Down
Sleep 250ms
Right
Sleep 1.5s

# Cycle theme for visual variety
Type "t"
Sleep 1.2s

# Final frame
Type "q"
Sleep 500ms
