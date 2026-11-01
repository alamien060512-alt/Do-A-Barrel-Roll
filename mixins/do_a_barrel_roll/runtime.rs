use std::cell::RefCell;

pub const ID: &str = "do_a_barrel_roll";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Fly the elytra like a plane: the mouse rolls and pitches, a banked wing turns you, A and D yaw.";

const DEGREES_PER_COUNT: f32 = 0.15;
pub const SPEEDS: [f32; 4] = [0.5, 1.0, 1.5, 2.0];
const BANK_TURN: f32 = 70.0;
const STRAFE_TURN: f32 = 75.0;
const LEVEL_OUT: f32 = 9.0;
const AUTO_LEVEL: f32 = 55.0;
const STILL_AFTER: f32 = 0.25;
const MAX_STEP: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub speed: usize,
    pub bank: bool,
    pub strafe: bool,
    pub auto_level: bool,
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
    settings: Option<Settings>,
    roll: f32,
    quiet: f32,
    turning: bool,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn settings(get: impl Fn(&str) -> Option<String>) -> Settings {
    STATE.with_borrow_mut(|s| *s.settings.get_or_insert_with(|| Settings::read(get)))
}

pub fn set_settings(settings: Settings) {
    STATE.with_borrow_mut(|s| s.settings = Some(settings));
}

fn wrap(degrees: f32) -> f32 {
    let w = (degrees + 180.0).rem_euclid(360.0) - 180.0;
    if w.is_finite() { w } else { 0.0 }
}

pub fn steer(xo: f64) {
    STATE.with_borrow_mut(|s| {
        let speed = SPEEDS[s.settings.unwrap_or_default().speed];
        s.roll = wrap(s.roll + xo as f32 * DEGREES_PER_COUNT * speed);
        if xo != 0.0 {
            s.quiet = 0.0;
        }
    });
}

pub fn step(gliding: bool, seconds: f32, strafe: f32) -> f32 {
    let dt = seconds.clamp(0.0, MAX_STEP);
    STATE.with_borrow_mut(|s| {
        let cfg = s.settings.unwrap_or_default();
        if !gliding {
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

pub fn camera_roll() -> f32 {
    STATE.with_borrow(|s| {
        if s.settings.unwrap_or_default().camera {
            s.roll
        } else {
            0.0
        }
    })
}

pub fn reset() {
    STATE.with_borrow_mut(|s| {
        s.roll = 0.0;
        s.quiet = 0.0;
    });
}

pub fn turning() -> bool {
    STATE.with_borrow(|s| s.turning)
}

pub fn own_turn(turn: impl FnOnce()) {
    STATE.with_borrow_mut(|s| s.turning = true);
    turn();
    STATE.with_borrow_mut(|s| s.turning = false);
}

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
        steer(100.0);
        assert!((camera_roll() - 15.0).abs() < 1e-4);
        steer(1200.0);
        assert!((camera_roll() + 165.0).abs() < 1e-3);
        steer(-100.0);
        assert!((camera_roll() + 180.0).abs() < 1e-3 || (camera_roll() - 180.0).abs() < 1e-3);
    }

    #[test]
    fn bank_turns_toward_the_low_wing() {
        fresh(Settings::default());
        steer(600.0);
        let right = step(true, 0.05, 0.0);
        assert!(right > 0.0);
        fresh(Settings::default());
        steer(-600.0);
        assert!(step(true, 0.05, 0.0) < 0.0);
        fresh(Settings::default());
        assert_eq!(step(true, 0.05, 0.0), 0.0);
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
        step(true, 0.05, 0.0);
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
