// Do A Barrel Roll: flight-simulator controls for elytra flight. An original mod; it
// shares only the idea with other "roll while gliding" mods.
//
// While gliding:
//   * mouse left/right rolls the camera (the horizon tilts),
//   * mouse up/down pitches as usual,
//   * a banked wing turns you: heading changes with the sine of the roll,
//   * A / D yaw flat, without rolling.
// On the ground, or out of the elytra, the roll eases back to level.
//
// This file is both the mod's Cargo library (its tests) and the module the
// bundle adds to the game as `game::barrel_roll`. It holds state and arithmetic
// only; the hooks that connect it to the game are the small files beside it.
use std::cell::RefCell;

/// The mod's id: its settings folder.
pub const ID: &str = "do_a_barrel_roll";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Fly the elytra like a plane: the mouse rolls and pitches, a banked wing turns you, A and D yaw.";

/// Degrees of turn per mouse count: the game's own (`Entity.turn`).
const DEGREES_PER_COUNT: f32 = 0.15;
/// The roll speeds the settings button cycles through.
pub const SPEEDS: [f32; 4] = [0.5, 1.0, 1.5, 2.0];
/// Heading change, degrees per second, at a 90 degree bank.
const BANK_TURN: f32 = 70.0;
/// Heading change, degrees per second, while A or D is held.
const STRAFE_TURN: f32 = 75.0;
/// How fast a roll returns to level outside flight (per second, exponential).
const LEVEL_OUT: f32 = 9.0;
/// Auto-level: degrees per second back toward level once the mouse is still.
const AUTO_LEVEL: f32 = 55.0;
/// The mouse counts as still after this long without a roll input (seconds).
const STILL_AFTER: f32 = 0.25;
/// Longest frame step the arithmetic trusts (seconds): a hitch is not a spin.
const MAX_STEP: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Index into `SPEEDS`.
    pub speed: usize,
    /// A banked wing turns the player.
    pub bank: bool,
    /// A and D yaw while gliding.
    pub strafe: bool,
    /// Roll returns to level when the mouse is still.
    pub auto_level: bool,
    /// Tilt the camera. Off keeps only the controls.
    pub camera: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            speed: 1,
            bank: true,
            strafe: true,
            auto_level: false,
            camera: true,
        }
    }
}

impl Settings {
    /// From the stored values (`get` reads one key); defaults for missing ones.
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let d = Self::default();
        let flag = |key: &str, default: bool| {
            get(key).map_or(default, |v| match v.trim() {
                "true" => true,
                "false" => false,
                _ => default,
            })
        };
        Self {
            speed: get("speed")
                .and_then(|v| v.trim().parse::<usize>().ok())
                .filter(|&s| s < SPEEDS.len())
                .unwrap_or(d.speed),
            bank: flag("bank", d.bank),
            strafe: flag("strafe", d.strafe),
            auto_level: flag("auto_level", d.auto_level),
            camera: flag("camera", d.camera),
        }
    }
    /// The settings page's buttons.
    pub fn labels(&self) -> Vec<String> {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        vec![
            format!("Roll speed: {:.1}x", SPEEDS[self.speed]),
            format!("Banked turns: {}", on(self.bank)),
            format!("A / D yaw: {}", on(self.strafe)),
            format!("Auto-level: {}", on(self.auto_level)),
            format!("Camera roll: {}", on(self.camera)),
        ]
    }
    /// A click on button `i` of `labels`.
    pub fn click(&mut self, i: usize) {
        match i {
            0 => self.speed = (self.speed + 1) % SPEEDS.len(),
            1 => self.bank = !self.bank,
            2 => self.strafe = !self.strafe,
            3 => self.auto_level = !self.auto_level,
            4 => self.camera = !self.camera,
            _ => {}
        }
    }
    /// What to store, key by key.
    pub fn stored(&self) -> [(&'static str, String); 5] {
        let flag = |b: bool| if b { "true" } else { "false" }.to_owned();
        [
            ("speed", self.speed.to_string()),
            ("bank", flag(self.bank)),
            ("strafe", flag(self.strafe)),
            ("auto_level", flag(self.auto_level)),
            ("camera", flag(self.camera)),
        ]
    }
}

#[derive(Default)]
struct State {
    /// Read on first use, then kept.
    settings: Option<Settings>,
    /// Bank angle in degrees, -180..180; positive banks right.
    roll: f32,
    /// Seconds since the last roll input.
    quiet: f32,
    /// Set while the mod itself turns the player, so its hook lets that through.
    turning: bool,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// The settings, read once through `get`.
pub fn settings(get: impl Fn(&str) -> Option<String>) -> Settings {
    STATE.with_borrow_mut(|s| *s.settings.get_or_insert_with(|| Settings::read(get)))
}

/// Replaces the settings (the settings page changed them).
pub fn set_settings(settings: Settings) {
    STATE.with_borrow_mut(|s| s.settings = Some(settings));
}

fn wrap(degrees: f32) -> f32 {
    let w = (degrees + 180.0).rem_euclid(360.0) - 180.0;
    if w.is_finite() { w } else { 0.0 }
}

/// Mouse movement while gliding: `xo` counts of sideways movement roll the
/// wing. The caller turns pitch itself with the vertical movement.
pub fn steer(xo: f64) {
    STATE.with_borrow_mut(|s| {
        let speed = SPEEDS[s.settings.unwrap_or_default().speed];
        s.roll = wrap(s.roll + xo as f32 * DEGREES_PER_COUNT * speed);
        if xo != 0.0 {
            s.quiet = 0.0;
        }
    });
}

/// One frame. `gliding`: the player is flying with the elytra and playing.
/// `strafe`: -1 for A, +1 for D, 0 for neither. Returns the heading change in
/// degrees (positive turns right) for the caller to apply.
pub fn step(gliding: bool, seconds: f32, strafe: f32) -> f32 {
    let dt = seconds.clamp(0.0, MAX_STEP);
    STATE.with_borrow_mut(|s| {
        let cfg = s.settings.unwrap_or_default();
        if !gliding {
            // Ease out of the bank; snap the last sliver so it ends exactly level.
            s.roll *= (-LEVEL_OUT * dt).exp();
            if s.roll.abs() < 0.05 {
                s.roll = 0.0;
            }
            s.quiet = 0.0;
            return 0.0;
        }
        s.quiet += dt;
        if cfg.auto_level && s.quiet > STILL_AFTER {
            let back = AUTO_LEVEL * dt;
            s.roll = if s.roll.abs() <= back {
                0.0
            } else {
                s.roll - back * s.roll.signum()
            };
        }
        let mut turn = 0.0;
        if cfg.bank {
            turn += BANK_TURN * s.roll.to_radians().sin() * dt;
        }
        if cfg.strafe {
            turn += STRAFE_TURN * strafe.clamp(-1.0, 1.0) * dt;
        }
        turn
    })
}

/// The camera's roll in degrees: zero when the camera option is off.
pub fn camera_roll() -> f32 {
    STATE.with_borrow(|s| {
        if s.settings.unwrap_or_default().camera {
            s.roll
        } else {
            0.0
        }
    })
}

/// Forgets the bank (the mod was switched off).
pub fn reset() {
    STATE.with_borrow_mut(|s| {
        s.roll = 0.0;
        s.quiet = 0.0;
    });
}

/// Whether the mod itself is turning the player right now.
pub fn turning() -> bool {
    STATE.with_borrow(|s| s.turning)
}

/// Runs `turn` with the mod's own turning flagged, so its hook on the game's
/// turn does not meet it again.
pub fn own_turn(turn: impl FnOnce()) {
    STATE.with_borrow_mut(|s| s.turning = true);
    turn();
    STATE.with_borrow_mut(|s| s.turning = false);
}

/// The mouse counts that turn the heading by `degrees`.
pub fn counts_for(degrees: f32) -> f64 {
    f64::from(degrees / DEGREES_PER_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh(settings: Settings) {
        STATE.with_borrow_mut(|s| *s = State::default());
        set_settings(settings);
    }

    fn stored<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        |key| pairs.iter().find(|p| p.0 == key).map(|p| p.1.to_owned())
    }

    #[test]
    fn settings_default_and_round_trip() {
        assert_eq!(Settings::read(stored(&[])), Settings::default());
        let s = Settings {
            speed: 3,
            bank: false,
            strafe: false,
            auto_level: true,
            camera: false,
        };
        let pairs: Vec<(&str, String)> = s.stored().into();
        let back: Vec<(&str, &str)> = pairs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        assert_eq!(Settings::read(stored(&back)), s);
        // Junk falls back to the default rather than failing.
        let bad = Settings::read(stored(&[("speed", "99"), ("bank", "maybe")]));
        assert_eq!(bad, Settings::default());
    }

    #[test]
    fn buttons_cycle() {
        let mut s = Settings::default();
        for _ in 0..SPEEDS.len() {
            s.click(0);
        }
        assert_eq!(s.speed, Settings::default().speed);
        s.click(1);
        s.click(3);
        assert!(!s.bank && s.auto_level);
        assert_eq!(s.labels().len(), s.stored().len());
    }

    #[test]
    fn mouse_rolls_and_wraps() {
        fresh(Settings::default());
        steer(100.0); // 15 degrees at 1.0x
        assert!((camera_roll() - 15.0).abs() < 1e-4);
        steer(1200.0); // +180: wraps past the top to -165
        assert!((camera_roll() + 165.0).abs() < 1e-3);
        steer(-100.0);
        assert!((camera_roll() + 180.0).abs() < 1e-3 || (camera_roll() - 180.0).abs() < 1e-3);
    }

    #[test]
    fn bank_turns_toward_the_low_wing() {
        fresh(Settings::default());
        steer(600.0); // 90 degrees right
        let right = step(true, 0.05, 0.0);
        assert!(right > 0.0);
        fresh(Settings::default());
        steer(-600.0);
        assert!(step(true, 0.05, 0.0) < 0.0);
        // Level wings fly straight.
        fresh(Settings::default());
        assert_eq!(step(true, 0.05, 0.0), 0.0);
        // Inverted (180) does not turn.
        steer(1200.0);
        assert!(step(true, 0.05, 0.0).abs() < 1e-3);
    }

    #[test]
    fn strafe_yaws_flat_and_options_gate_it() {
        fresh(Settings::default());
        assert!(step(true, 0.1, 1.0) > 0.0);
        assert!(step(true, 0.1, -1.0) < 0.0);
        assert_eq!(camera_roll(), 0.0);
        fresh(Settings {
            strafe: false,
            bank: false,
            ..Settings::default()
        });
        steer(300.0);
        assert_eq!(step(true, 0.1, 1.0), 0.0);
    }

    #[test]
    fn levels_out_when_not_gliding() {
        fresh(Settings::default());
        steer(400.0);
        assert!(camera_roll() > 10.0);
        for _ in 0..200 {
            assert_eq!(step(false, 0.05, 1.0), 0.0);
        }
        assert_eq!(camera_roll(), 0.0);
    }

    #[test]
    fn auto_level_waits_for_a_still_mouse() {
        fresh(Settings {
            auto_level: true,
            ..Settings::default()
        });
        steer(400.0);
        let before = camera_roll();
        step(true, 0.05, 0.0); // within the quiet window: holds
        assert_eq!(camera_roll(), before);
        for _ in 0..100 {
            step(true, 0.05, 0.0);
        }
        assert_eq!(camera_roll(), 0.0);
    }

    #[test]
    fn camera_option_hides_the_tilt_not_the_flight() {
        fresh(Settings {
            camera: false,
            ..Settings::default()
        });
        steer(600.0);
        assert_eq!(camera_roll(), 0.0);
        assert!(step(true, 0.05, 0.0) > 0.0);
    }

    #[test]
    fn a_long_hitch_is_clamped() {
        fresh(Settings::default());
        steer(600.0);
        let a = step(true, 10.0, 0.0);
        fresh(Settings::default());
        steer(600.0);
        let b = step(true, MAX_STEP, 0.0);
        assert_eq!(a, b);
    }

    #[test]
    fn own_turn_flag_is_scoped() {
        assert!(!turning());
        own_turn(|| assert!(turning()));
        assert!(!turning());
        assert!((counts_for(0.15) - 1.0).abs() < 1e-6);
    }
}
