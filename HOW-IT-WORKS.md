# How Do A Barrel Roll gets into Horizon

## 1. What a `.hnmod` is
A zip, not a program. Open `do_a_barrel_roll-0.1.0.hnmod` with any unzip tool:

```
horizon-native.json                      <- the bundle's label (see below)
mixins/do_a_barrel_roll/horizon-mixin.toml  <- the manifest: what to patch, where
mixins/do_a_barrel_roll/runtime.rs       <- the mod's logic and state
mixins/do_a_barrel_roll/*.rs             <- one small helper per hook
```

No compiled code is inside. That is why you never edit or compile the file yourself.

## 2. How it compiles (what the launcher does)
1. You install the `.hnmod` in the Horizon launcher (0.2.23+).
2. You press **Build** and agree to trust native code (it runs with your account's permissions).
3. The launcher downloads Horizon's source at the exact commit named in `horizon-native.json`
   (`c1f4a7c...`) and the horizon-mixins tool at its pinned commit (`ddd9700...`).
4. **horizon-mixins patches the source text**, following `horizon-mixin.toml`:
   - `[[module]]` copies `runtime.rs` into the game as `crate::game::barrel_roll`.
   - each `[[injection]]` pastes a helper function into one game function:
     `HEAD` = at the start, `RETURN` = changes the returned value, `cancellable` = may skip the original.
5. `cargo` builds the patched game (first build 20-30 min) into a "Native mods" instance.
6. You play that instance. Turn the mod on/off in Options > Mods.

If a pinned function changed in a newer Horizon, step 4 fails with a clear error. That is the
risk of the two hooks that target game functions (`turn_player`, `spinning_matrix`).

## 3. Where each identifier and text lives

| Thing | Where | Value | Rules |
|---|---|---|---|
| **id** | `horizon-mixin.toml` `[mod] id`, also `ID` in `runtime.rs`, also `horizon-native.json` | `do_a_barrel_roll` | lowercase letters, digits, `_`. Never change after publishing. Also the name of the settings folder. |
| **version** | `[mod] version` | `0.1.0` | each published version needs a new one |
| **priority** | `[mod] priority` | `100` | order when mods hook the same function |
| **in-game title** | `list.rs` -> `NativeMod.title` | "Do A Barrel Roll 0.1.0" | shown in Options > Mods |
| **in-game description** | `runtime.rs` `DESCRIPTION`, used by `list.rs` | "Fly the elytra like a plane..." | shown in Options > Mods |
| **game / hooks / pins** | `compatibility.json` | game 26.3, hooks 1, two commits | what it was tested on |
| **licence** | `LICENSE`, `Cargo.toml` | MIT | the registry needs a permissive licence |

## 4. Summary, description, icon, screenshots: not in the file
The bundle holds **no** store-page text. On the Horizon registry those come from the submission form at
https://playhorizon.dev/mods/submit (sign in with GitHub, pick the `.hnmod`, fill in the page), which opens a pull
request on the `horizon-mods` repository. A reviewer reads it. Suggested text:

- **Name:** Do A Barrel Roll
- **Summary (one line):** Fly the elytra like a plane: roll with the mouse, bank to turn.
- **Description:** Mouse left/right rolls the camera while gliding; mouse up/down pitches. A banked wing turns you,
  A/D yaw flat, and the roll levels out when you land. Settings: roll speed, banked turns, A/D yaw, auto-level,
  camera roll. Inspired by "Do a Barrel Roll" by enjarai (GPL-3.0); a from-scratch implementation with no shared
  code, not affiliated with its authors.
- **Source link:** https://github.com/alamien060512-alt/Do-A-Barrel-Roll
- **Licence:** MIT

Registry rules that apply: the source link must match the published file, and a port or inspired-by mod must credit
and link the original (`MOD-POLICY.md` in `horizon-mods`). Native mods also need the launcher/`cargo-horizon-mod`
native support to have shipped before they can be published.

## 5. Checking it yourself
From a checkout of https://github.com/horizon-rust/horizon-mods with this folder placed at `native/do-a-barrel-roll`:

```sh
python3 native/tools/native.py check do-a-barrel-roll     # patch + cargo check
python3 native/tools/native.py package do-a-barrel-roll   # makes the .hnmod (what I built by hand)
```
