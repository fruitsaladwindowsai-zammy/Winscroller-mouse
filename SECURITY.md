# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0.0 | :x:                |

---

## Reporting a Vulnerability

Security issues and vulnerabilities should be reported responsibly. Please do **not** open public GitHub issues for security vulnerabilities.

### Reporting Process
1. Email your report or open a private security advisory on GitHub via **Security** > **Advisories** > **Report a vulnerability**.
2. Include:
   * Description of the vulnerability and impact.
   * Minimal reproduction steps or proof-of-concept (PoC).
   * Your Windows build version (e.g. Windows 11 23H2 / 24H2).
3. We will acknowledge receipt of your report within 48 hours and work on a coordinated fix and advisory.

---

## Privileges & Integrity Model

Winscroller Mouse runs at **standard user privilege** (no Administrator elevation required):
- It performs **zero network communication**; the IPC named pipe (`\\.\pipe\WinscrollerMouse`) is strictly local.
- Clipboard reads are bounded to `4 KB` and occur only when the user explicitly clicks **Paste Look JSON** or **Paste Hex**.
- System cursor overrides are applied in-session via `SetSystemCursor` and cleanly restored via `SystemParametersInfoW(SPI_SETCURSORS)` on exit, reset, or panic.
- No external shell interpreters (`cmd.exe`, `powershell.exe`, `python`) are spawned at runtime.
