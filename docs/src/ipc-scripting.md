# CLI & Named-Pipe IPC Scripting

Winscroller Mouse exposes a local named pipe at `\\.\pipe\WinscrollerMouse`. Invoking `winscroller-mouse.exe` with arguments forwards the command to the running daemon in `< 5 ms`:

```powershell
winscroller-mouse.exe toggle              # Toggle effects (also: on, off)
winscroller-mouse.exe find                # Trigger Shake-to-Find burst
winscroller-mouse.exe settings            # Open or focus the Settings Panel
winscroller-mouse.exe look aurora         # Apply a built-in look (also: nextLook, undo)
winscroller-mouse.exe cursor neon         # Apply a built-in cursor (or: cursor default)
winscroller-mouse.exe set trail bubbles   # Set any configuration key
winscroller-mouse.exe save 1              # Save settings to slot 1 (also: saveNew, load 1)
winscroller-mouse.exe exportLook          # Print look JSON
winscroller-mouse.exe importLook "{...}"  # Apply look JSON
winscroller-mouse.exe status              # Print active settings JSON
winscroller-mouse.exe quit                # Cleanly exit & restore system cursors
```
