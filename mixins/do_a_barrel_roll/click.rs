fn barrel_roll_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::barrel_roll as barrel_roll;
    if mod_id != barrel_roll::ID {
        return;
    }
    let mut settings = barrel_roll::settings(|key| game.native_setting(barrel_roll::ID, key));
    settings.click(button);
    barrel_roll::set_settings(settings);
    for (key, value) in settings.stored() {
        game.native_set_setting(barrel_roll::ID, key, &value);
    }
}
