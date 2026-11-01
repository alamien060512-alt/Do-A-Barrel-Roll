# Do A Barrel Roll (Horizon native mod)

Fly the elytra like a plane. While gliding:

- **Mouse left/right** rolls the camera; **mouse up/down** pitches as usual.
- **A banked wing turns you**: heading changes with the sine of the bank.
- **A / D** yaw flat, without rolling.
- Out of flight the roll eases back to level.

Settings (Options > Mods > Do A Barrel Roll): roll speed, banked turns, A/D yaw, auto-level, camera roll.

An original, from-scratch implementation of the flight-sim elytra idea, inspired by
[Do a Barrel Roll](https://codeberg.org/enjarai/do-a-barrel-roll) by enjarai (GPL-3.0). It contains no code
from that mod, and is not made by, affiliated with, or endorsed by its authors. Project page:
https://github.com/alamien060512-alt/Do-A-Barrel-Roll. Licence: MIT.

## How it works

A [horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle in `mixins/do_a_barrel_roll/`:

| Hook | Target | What it does |
|---|---|---|
| `frame` | `native_frame` (documented hook point) | eases the roll, applies banked turn and A/D yaw |
| `turn` | `perspective::Game::turn_player` (game function) | while gliding, mouse X rolls; only mouse Y reaches the game's turn |
| `camera` | `fov::Game::spinning_matrix` (game function, `RETURN`) | multiplies a roll about the view axis into the nausea/portal spin |
| `list`, `settings`, `click` | documented hook points | Options > Mods row and settings page |

`turn` and `camera` hook the game's own functions, so they build only on the commit in
`horizon_revision` and must be re-checked on a newer Horizon.

## Status

- `cargo test` (the logic in `runtime.rs`): **passes** (10 tests).
- **Not yet verified**: that the hooks apply and the game compiles with the bundle, and how it feels in
  game. Run:

  ```sh
  python3 native/tools/native.py check /path/to/do-a-barrel-roll
  horizon-launcher native-build /path/to/do-a-barrel-roll/mixins/do_a_barrel_roll --trust-native-code
  ```

  (from a checkout of `horizon-mods`; the first build takes 20 to 30 minutes).
  Likely things to adjust in game: the roll direction (flip the sign in `camera.rs`), the
  turn rates in `runtime.rs`, and that the first-person hand does not roll with the camera.

## Limits

- Pitch stays clamped to +/-90 degrees (the game's rule), so loops over the top are not possible; rolls are.
- Roll is client-side: other players do not see your model bank.
