# Winscroller Mouse — Configuration Reference

Winscroller Mouse stores its configuration and 12 saved slots in JSON format at:

```text
%APPDATA%\WinscrollerMouse\settings.json
```

A complete annotated default template is included in the repository root as [`winscroller-mouse.example.json`](winscroller-mouse.example.json). Generated Windows `.cur` and `.ani` cursor binaries are cached under `%LOCALAPPDATA%\WinscrollerMouse\cursors\`.

---

## Top-Level Structure

```json
{
  "settings": { ... },
  "slots": [ null, null, ... ]
}
```

- `settings`: Active configuration object (all keys use `camelCase`, matching `omarchy-plume` 1-to-1).
- `slots`: Array of 12 optional full `settings` snapshots (`null` when empty).

---

## Trail Settings

| Key | Type | Range / Values | Default | Description |
|-----|------|----------------|---------|-------------|
| `trail` | enum | `none`, `comet`, `beads`, `ribbon`, `sparks`, `echoes`, `bubbles` | `"comet"` | Trail style drawn behind the pointer |
| `trailLife` | integer | `15..150` (step 5) | `45` | Trail lifetime in hundredths of a second (`45` = `0.45 s`) |
| `trailSize` | integer | `2..30` | `8` | Base stroke thickness / particle scale in pixels |
| `trailStrength` | integer | `10..100` (step 5) | `85` | Opacity/intensity percentage |

---

## Halo Settings

| Key | Type | Range / Values | Default | Description |
|-----|------|----------------|---------|-------------|
| `halo` | enum | `none`, `glow`, `ring`, `pulse`, `spotlight`, `crosshair`, `target` | `"glow"` | Visual halo around the pointer |
| `haloSize` | integer | `20..240` (step 4) | `64` | Radius/scale of the halo in pixels |
| `haloStrength` | integer | `10..100` (step 5) | `70` | Opacity/intensity percentage |
| `haloShow` | enum | `always`, `moving`, `still` | `"always"` | Visibility trigger (`still` fades in after 0.6s of rest) |

---

## Color & Panel Theme Settings

| Key | Type | Range / Values | Default | Description |
|-----|------|----------------|---------|-------------|
| `colorMode` | enum | `theme`, `palette`, `border`, `custom`, `gradient`, `rainbow`, `fade`, `speed` | `"theme"` | Color source for trails, halos, and find rings |
| `customColor` | hex string | `#RRGGBB` | `"#7aa2f7"` | Primary color for `custom` and `gradient` modes |
| `customColor2` | hex string | `#RRGGBB` | `"#f7768e"` | End color for `gradient` mode |
| `panelColor` | enum | `theme`, `palette`, `borders`, `custom`, `monochrome` | `"theme"` | Settings panel color scheme |
| `panelBg` | hex string | `#RRGGBB` | `"#141414"` | Panel background (`custom` scheme) |
| `panelFg` | hex string | `#RRGGBB` | `"#e0e0e0"` | Panel text color (`custom` scheme / `panelText = "custom"`) |
| `panelCustomColor` | hex string | `#RRGGBB` | `"#ff6a1f"` | Panel accent (`custom` scheme) |
| `panelText` | enum | `auto`, `theme`, `custom` | `"auto"` | Automatic contrast protection vs theme/custom text |
| `panelFrame` | boolean | `true` / `false` | `false` | Draw a colored border around the Settings Panel |
| `panelFrameGradient` | boolean | `true` / `false` | `true` | Use window-border gradient on panel outline |
| `panelFrameColor` | enum | `accent`, `first`, `middle`, `last` | `"accent"` | Outline color when gradient is off |
| `panelTint` | integer | `0..30` | `0` | Accent tint wash percentage over panel background |
| `panelOpacity` | integer | `50..100` | `97` | Panel window opacity percentage |

---

## Custom Cursor Forge Settings

| Key | Type | Range / Values | Default | Description |
|-----|------|----------------|---------|-------------|
| `pointerTheme` | string | `""` (Windows Default) or `"*winscroller"` | `""` | Active cursor mode (`*plume` also accepted as alias) |
| `pointerSize` | integer | `16..96` (step 2) | `32` | Rendered cursor pixel size |
| `cursorShape` | enum | `classic`, `slim`, `chunky`, `wedge`, `dart`, `kite`, `needle`, `teardrop`, `leaf`, `bullet`, `crest`, `chevron`, `feather`, `dot` | `"classic"` | Primary arrow silhouette (14 options) |
| `cursorTilt` | integer | `-30..30` (step 2) | `0` | Rotation around the hotspot tip in degrees |
| `cursorRound` | integer | `0..100` (step 5) | `30` | Corner rounding from sharp (`0`) to soft (`100`) |
| `cursorSurface` | enum | 30 surfaces: `filled`, `faint`, `twotone`, `split`, `frost`, `bevel`, `keycap`, `metal`, `carbon`, `crt`, `candy`, `halftone`, `holo`, `comet`, `damask`, `velvet`, `marble`, `ledge`, `band`, `underglow`, `spiderweb`, `wood`, `terrazzo`, `crackle`, `topo`, `nebula`, `leather`, `stained`, `shadow`, `clear` | `"filled"` | Procedural surface material |
| `cursorFill` | enum | `solid`, `gradient`, `glass` | `"solid"` | Fill mode |
| `cursorGradientTo` | enum | `second`, `lighter`, `darker`, `accent`, `palette`, `border` | `"second"` | Target color when `cursorFill == "gradient"` |
| `cursorGradientDir` | enum | `along`, `across`, `diagonal`, `theme` | `"along"` | Gradient direction |
| `cursorPattern` | enum | `none`, `stripes`, `dots`, `grid` | `"none"` | Micro-texture overlay |
| `cursorDepth` | enum | `flat`, `raised`, `sunken` | `"flat"` | 3D rim lighting |
| `cursorOpacity` | integer | `30..100` (step 5) | `100` | Cursor opacity percentage |
| `cursorTint` | integer | `0..40` (step 2) | `0` | Theme accent tint percentage |
| `cursorColor` / `cursorColor2` | enum | `accent`, `foreground`, `background`, `border`, `border2`, `custom`, `white`, `black` | `"foreground"` / `"accent"` | Primary & secondary color sources |
| `cursorColorCustom` / `cursorColor2Custom` | hex string | `#RRGGBB` | `"#7aa2f7"` / `"#f7768e"` | Hex colors when set to `custom` |
| `cursorLine` | enum | 22 lines: `none`, `solid`, `double`, `triple`, `glow`, `neon`, `dashed`, `dotted`, `dashdot`, `stitched`, `inset`, `mat`, `rail`, `wavy`, `sawtooth`, `rope`, `chain`, `beads`, `ladder`, `sketch`, `gilded`, `glitch` | `"solid"` | Outline style |
| `cursorLineColor` | enum | `auto`, `subtle`, + color sources | `"auto"` | Outline color source |
| `cursorLineCustom` | hex string | `#RRGGBB` | `"#101010"` | Custom line hex color |
| `cursorLineWidth` | integer | `1..4` | `2` | Outline thickness tier |
| `cursorLineGradient` | boolean | `true` / `false` | `false` | Fade outline color across cursor |
| `cursorLineFadeTo` | enum | `lighter`, `second`, `accent`, `palette`, `border` | `"lighter"` | Outline gradient end color |
| `cursorLineFadeDir` | enum | `along`, `diagonal`, `across`, `theme` | `"along"` | Outline gradient direction |
| `cursorOrnament` | enum | 18 ornaments: `none`, `studs`, `rivets`, `pearls`, `crosses`, `stars`, `brackets`, `rosettes`, `diamonds`, `fleurons`, `filigree`, `flourish`, `scrolls`, `chevrons`, `fans`, `deco`, `crown`, `laurels` | `"none"` | Corner/side decorative elements |
| `cursorOrnamentSize` | integer | `3..16` | `7` | Ornament scale |
| `cursorOrnamentsOn` | enum | `arrow`, `all` | `"arrow"` | Apply ornaments to arrow-based cursors or all cursors |
| `cursorCap` | enum | `none`, `tip`, `tail`, `both` | `"none"` | Accent dot on arrow ends |
| `cursorDetail` | enum | `none`, `tip`, `spine`, `spark` | `"none"` | Fine accent detail on arrow |
| `cursorShadow` | enum | `none`, `soft`, `strong`, `hard`, `long` | `"soft"` | Drop shadow style |
| `cursorShadowColor` | enum | `dark`, `accent` | `"dark"` | Dark drop shadow vs accent glow |
| `cursorShadowDir` | enum | `below`, `around` | `"below"` | Shadow offset direction |
| `cursorShadowStrength` | integer | `10..100` (step 5) | `60` | Shadow opacity percentage |
| `cursorHand` | enum | `hand`, `arrow`, `ring` | `"hand"` | Link pointer style |
| `cursorBeam` | enum | `beam`, `slim`, `caret` | `"beam"` | Text I-beam style |
| `cursorBusy` | enum | `ring`, `dots`, `orbit` | `"ring"` | Animated busy/progress spinner style |
| `cursorBusySpeed` | enum | `slow`, `normal`, `fast` | `"normal"` | Animated `.ani` frame rate (`110ms`, `70ms`, `45ms`) |

---

## Hiding, Shake-to-Find & Behavior Settings

| Key | Type | Range / Values | Default | Description |
|-----|------|----------------|---------|-------------|
| `hideAfter` | enum | `keep`, `0`, `2`, `5`, `10`, `30` | `"keep"` | Auto-hide cursor after N idle seconds |
| `hideTyping` | enum | `keep`, `on`, `off` | `"keep"` | Hide cursor while typing on the keyboard |
| `shake` | boolean | `true` / `false` | `true` | Enable Shake-to-Find radar rings |
| `shakeSensitivity` | integer | `1..10` | `5` | `1` requires a big shake; `10` triggers on a small wiggle |
| `findSize` | integer | `30..160` (step 5) | `60` | Base radius of Find radar rings |
| `enabled` | boolean | `true` / `false` | `true` | Master toggle for pointer effects |
| `fullscreen` | enum | `hide`, `show` | `"hide"` | Hide effects automatically over fullscreen games/videos |
| `fps` | enum | `"30"`, `"60"`, `"120"`, `"auto"` | `"60"` | Frame rate cap (`auto` syncs to monitor refresh rate) |
| `wheelLooks` | boolean | `true` / `false` | `true` | Enable cycling looks |
| `startLook` | enum | `keep`, `"1"`..`"12"` | `"keep"` | Saved slot to load automatically on launch |
| `startWithWindows` | boolean | `true` / `false` | `false` | Register/remove `HKCU\...\Run\WinscrollerMouse` |
