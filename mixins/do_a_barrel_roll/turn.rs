fn barrel_roll_turn(
    game: &mut crate::game::Game<'_>,
    xo: f64,
    yo: f64,
) -> core::ops::ControlFlow<()> {
    use crate::game::barrel_roll as barrel_roll;
    if barrel_roll::turning()
        || !game.native_on(barrel_roll::ID)
        || !game.native_playing()
        || !game.player.as_ref().is_some_and(|p| p.fall_flying)
    {
        return core::ops::ControlFlow::Continue(());
    }
    {
        let g = &*game;
        barrel_roll::settings(|key| g.native_setting(barrel_roll::ID, key));
    }
    barrel_roll::steer(xo);
    barrel_roll::own_turn(|| game.turn_player(0.0, yo));
    core::ops::ControlFlow::Break(())
}
