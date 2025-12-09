Output asset/demo.gif

Set FontSize 16
Set Width 1200
Set Height 800
Set Padding 20
Set FontFamily "Menlo"

Type "twg samples/cloud_infrastructure.json"
Sleep 1s
Enter

Sleep 2s

# 0. Setup
Set TypingSpeed 120ms

# 1. Basic Navigation (Explore Regions -> VPCs)
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

# 2. Search (Find 'api-gateway')
Type "/"
Sleep 2s
Type "api-gateway"
Sleep 2s
Enter
Sleep 5s

# 3. Smart Search (Jump to Instance ID)
Type "/"
Sleep 2s
# Jump to a specific instance in private subnet
Type '.regions["us-east-1"].vpcs[0].subnets[1].instances[0].name'
Sleep 3s
Enter
Sleep 4s

# 4. Search Navigation (Finding Databases)
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

# 5. Inline Help
Type "?"
Sleep 3s
Escape
Sleep 1s

# 6. Theme Toggle
Type "t"
Sleep 2s
Type "t"
Sleep 2s

# 7. Copy Path
Type "c"
Sleep 2s

# Quit
Type "q"
Sleep 1s
