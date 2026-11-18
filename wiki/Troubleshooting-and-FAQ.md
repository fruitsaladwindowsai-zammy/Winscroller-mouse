# Troubleshooting and FAQ

### How do I restore my original Windows cursor?
Right-click the Winscroller Mouse tray icon and select **Restore System Cursor**, or run:
```powershell
winscroller-mouse.exe cursor default
```
Winscroller Mouse also automatically restores your original Windows cursor on exit or crash.

### Why are effects hidden inside my fullscreen game or video?
By default, `fullscreen` is set to `"hide"` so games and fullscreen videos stay completely clean. You can change this under **Behavior -> Over fullscreen windows**.

### Where are crash logs and settings stored?
- Settings & saved slots: `%APPDATA%\WinscrollerMouse\settings.json`
- Crash log (if a panic ever occurs): `%APPDATA%\WinscrollerMouse\crash.log`
- Generated `.cur` / `.ani` cache: `%LOCALAPPDATA%\WinscrollerMouse\cursors\`
