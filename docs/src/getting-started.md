# Getting Started

## 1. Install or Build

Download the latest standalone, statically linked `winscroller-mouse.exe` directly from the repository root or from [GitHub Releases](https://github.com/fruitsaladwindowsai-zammy/Winscroller-mouse/releases/latest) — no VC++ redistributable or external runtimes are required.

Double-clicking `winscroller-mouse.exe` directly launches the interactive 8-tab Settings Panel and system tray icon.

```powershell
Invoke-WebRequest -Uri "https://github.com/fruitsaladwindowsai-zammy/Winscroller-mouse/releases/download/latest/winscroller-mouse.exe" -OutFile "$HOME\Downloads\winscroller-mouse.exe"
& "$HOME\Downloads\winscroller-mouse.exe"
```

## 2. System Tray Interactions

Once running, Winscroller Mouse resides quietly in the Windows notification area:
- **Left-click**: Toggle pointer effects on/off.
- **Right-click**: Open the context menu (Settings Panel, Quick Looks, Ready-Made Cursors, Next Look, Find Pointer, Undo, Restore System Cursor, Quit).
- **Middle-click**: Trigger Shake-to-Find rings immediately.
- **Double-click**: Open or focus the 8-tab Settings Panel.

## 3. Clean Exit & System Cursor Safety

Whenever you disable custom cursors, click **Restore System Cursor**, quit via tray/CLI, or even if an unexpected panic occurs, Winscroller Mouse calls `SystemParametersInfoW(SPI_SETCURSORS)` to restore your exact original Windows cursor scheme.
