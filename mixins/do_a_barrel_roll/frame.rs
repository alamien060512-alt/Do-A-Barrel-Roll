fn barrel_roll_frame(game: &mut crate::game::Game<'_>, elapsed: std::time::Duration) {
    use crate::game::barrel_roll as barrel_roll;
    if !game.native_on(barrel_roll::ID) {
        barrel_roll::reset();
        return;
    }
    let gliding = game.native_playing() && game.player.as_ref().is_some_and(|p| p.fall_flying);
    let strafe = if gliding {
        let held = |game: &mut crate::game::Game<'_>, name: &str| {
            game.native_mapping(name).is_some_and(|(down, _)| down)
        };
        let right = f32::from(u8::from(held(game, "key.right")));
        let left = f32::from(u8::from(held(game, "key.left")));
        right - left
    } else {
        0.0
    };
    {
        let g = &*game;
        barrel_roll::settings(|key| g.native_setting(barrel_roll::ID, key));
    }
    let degrees = barrel_roll::step(gliding, elapsed.as_secs_f32(), strafe);
    if degrees != 0.0 {
        barrel_roll::own_turn(|| game.turn_player(barrel_roll::counts_for(degrees), 0.0));
    }
}
