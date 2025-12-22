Output asset/demo.gif

Set FontSize 16
Set Width 1600
Set Height 1000
Set Padding 20
Set FontFamily "Menlo"
Set TypingSpeed 100ms
Set WindowBar Colorful

# Intro
Sleep 500ms

# --- Scene 1: JSON Exploration ---
Type "twg samples/cloud_infrastructure.json"
Sleep 500ms
Enter
Sleep 2s

# Navigate JSON
Down 2
Sleep 500ms
Right
Sleep 500ms
Down
Sleep 200ms
Down
Sleep 200ms
Right
Sleep 1s

# Search JSON
Type "/"
Sleep 500ms
Type "subnets"
Sleep 1s
Enter
Sleep 2s

# Expand context
Right
Sleep 1s
Down
Sleep 1s

# Exit JSON
Type "q"
Sleep 1s

# --- Scene 2: YAML Exploration ---
Type "twg samples/k8s_manifest.yaml"
Sleep 500ms
Enter
Sleep 2s

# Navigate YAML
Down
Sleep 500ms
Right
Sleep 500ms
Down 3
Sleep 500ms
Right
Sleep 1s

# Search YAML (Fuzzy)
Type "/"
Sleep 500ms
Type "PullPolicy"
Sleep 1s
Enter
Sleep 2s

# Jump to path
Type ":"
Sleep 1s
Type ".[1].spec.ports[0]"
Sleep 1s
Enter
Sleep 2s

# Theme Toggle
Type "t"
Sleep 1.5s
Type "t"
Sleep 1s

# Exit
Type "q"
Sleep 100ms
