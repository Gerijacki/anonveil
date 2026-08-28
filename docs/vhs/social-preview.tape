# GitHub's recommended social-preview size is 1280x640 — set before
# Output, and Screenshot's filename must have no hyphens/underscores
# (VHS's tape parser chokes on both — found the hard way).
Set FontFamily "DejaVu Sans Mono"
Set FontSize 30
Set Width 1280
Set Height 640
Set Padding 60
Set Theme {"name":"anonveil","black":"#0a0a0c","red":"#ff5a5a","green":"#00ff9c","yellow":"#ffd154","blue":"#4fa8ff","purple":"#c77dff","cyan":"#33ccff","white":"#d6ffe0","brightBlack":"#78827f","brightRed":"#ff5a5a","brightGreen":"#00ff9c","brightYellow":"#ffd154","brightBlue":"#4fa8ff","brightPurple":"#c77dff","brightCyan":"#33ccff","brightWhite":"#ffffff","background":"#0a0a0c","foreground":"#d6ffe0","cursor":"#00ff9c","selection":"#0c8c5a"}
Set Shell bash
Output socialpreviewtmp.gif

Hide
Type "clear"
Enter
Show

Sleep 500ms
Type "sh banner.sh"
Sleep 300ms
Enter
Sleep 1s
Screenshot socialpreview.png
