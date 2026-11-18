# Winscroller Mouse — Usage Guide

This guide covers everyday usage of **Winscroller Mouse**: system tray interactions, the 8-tab Settings Panel, designing custom cursors, saving slots, and controlling the daemon via CLI / Named-Pipe IPC.

---

## 1. Launching & Single-Instance Behavior

Double-click `winscroller-mouse.exe` (or run it from PowerShell):
- On first launch, Winscroller Mouse starts silently in the **Windows system tray** with click-through DirectComposition overlays attached to every monitor.
- If Winscroller Mouse is **already running**, launching `winscroller-mouse.exe` again automatically focuses or opens the **Settings Panel** instead of creating a duplicate process.
- To launch directly into the Settings Panel:
  ```powershell
  winscroller-mouse.exe settings
  ```

---

## 2. System Tray Controls

| Action | Behavior |
|--------|----------|
| **Left-click** | Toggle pointer effects on / off |
| **Right-click** | Open full context menu (Settings Panel, Effects Enabled, Quick Looks, Ready-Made Cursors, Next Look, Find Pointer, Undo, Restore System Cursor, Quit) |
| **Middle-click** | Trigger the **Shake-to-Find** radar burst immediately |
| **Double-click** | Open or focus the **Settings Panel** |

---

## 3. Using the 8-Tab Settings Panel

All changes apply live with zero restart required and persist automatically to `%APPDATA%\WinscrollerMouse\settings.json`. When you close the Settings Panel, its window and Direct2D/DirectWrite resources are completely freed (`0 MB` idle overhead).

### Header: Live Preview & Search
- **Live Preview Banner**: Continuously animates your active trail and halo along a smooth Lissajous curve. Move your pointer inside the banner to test live motion response, or click **Find** to preview the Shake-to-Find burst.
- **Search Bar**: Click and type any keyword (e.g., `neon`, `shadow`, `fps`, `marble`) to filter settings across every tab at once. Press `Esc` to clear.

### Tab Overview
1. **Looks**: Apply any of the **12 Ready-Made Looks** or **16 Ready-Made Cursors**, click **Undo** to revert the last preset, **Copy/Paste Look JSON**, or manage your **12 Saved Places**:
   - **Left-click empty slot**: Save all current settings into that slot.
   - **Left-click filled slot (`*`)**: Load all settings from that slot.
   - **Right-click slot**: Overwrite that slot with current settings.
   - **Middle-click slot**: Clear that slot.
2. **Trail**: Pick from `None`, `Comet`, `Beads`, `Ribbon`, `Sparks`, `Echoes`, `Bubbles` and tune Length, Thickness, and Strength. Modified sliders display a `↺` reset button.
3. **Halo**: Pick from `None`, `Glow`, `Ring`, `Pulse`, `Spotlight`, `Crosshair`, `Target`, adjust Size and Strength, and choose visibility (`Always`, `While moving`, or `When still`).
4. **Colors**: Pick any of the 8 color modes (`Theme`, `Theme palette`, `Window border`, `Custom`, `Gradient`, `Rainbow`, `Drift`, `Speed`), pick palette swatches or click a hex button to paste a `#RRGGBB` color from your clipboard, and theme the Settings Panel itself (`Follow theme`, `Theme palette`, `Window borders`, `Custom`, `Monochrome`, outline, tint, opacity).
5. **Cursor**: Preview your Arrow, Hand, Beam, Busy, Help, and Move cursors live on a Light + Dark strip. Switch **Cursor Mode** to **Made in Winscroller Mouse** to customize Size (`16–96 px`), 14 Arrows, Tilt, Roundness, 30 Surfaces, Fill/Gradient/Glass, Texture, Depth, Opacity, Accent tint, Primary & Second colors, 22 Lines, 18 Ornaments, Accent cap, Detail, 5 Shadows, Link/Text/Busy cursor variants, and idle/typing hiding.
6. **Find**: Enable/disable **Shake to Find**, adjust Sensitivity (`1–10`) and Ring Size (`30–160`), and test immediately.
7. **Behavior**: Toggle Effects, Fullscreen auto-hide (`Hide` over games/videos vs `Show`), Smoothness (`30`, `60`, `120`, or `Auto (Monitor Hz)`), Start with Windows, and Startup Slot (`As left` or `Slot 1..5`).
8. **About**: Version summary, CLI quick reference, and **Reset All Settings**.

---

## 4. Command Line & Named-Pipe IPC Scripting

Any command sent to `winscroller-mouse.exe` while the daemon is running is forwarded over the local named pipe `\\.\pipe\WinscrollerMouse`:

```powershell
winscroller-mouse.exe --version           # Print version
winscroller-mouse.exe --help              # Print CLI help
winscroller-mouse.exe toggle              # Toggle effects on/off (also: on, off)
winscroller-mouse.exe find                # Trigger Shake-to-Find radar rings
winscroller-mouse.exe settings            # Open or focus the Settings Panel
winscroller-mouse.exe look neon           # Apply a ready-made look
winscroller-mouse.exe nextLook            # Cycle to the next ready-made look
winscroller-mouse.exe undo                # Undo the last look or cursor change
winscroller-mouse.exe cursor candy        # Apply a ready-made cursor
winscroller-mouse.exe cursor default      # Restore original Windows system cursor
winscroller-mouse.exe set trail sparks    # Set any configuration key
winscroller-mouse.exe set haloSize 96     # Adjust numeric settings live
winscroller-mouse.exe save 3              # Save all settings into slot 3 (also: saveNew)
winscroller-mouse.exe load 3              # Restore all settings from slot 3
winscroller-mouse.exe exportLook          # Print current look JSON to stdout
winscroller-mouse.exe importLook "{...}"  # Apply a look JSON string
winscroller-mouse.exe status              # Print full active settings JSON
winscroller-mouse.exe quit                # Cleanly exit and restore Windows system cursors
```

### Example: Bind `find` to a Global Hotkey (AutoHotkey / PowerToys / WinScroller 2D)
You can trigger `winscroller-mouse.exe find` or `winscroller-mouse.exe nextLook` from any shortcut runner, Stream Deck, or AutoHotkey script — execution takes `< 5 ms`.
