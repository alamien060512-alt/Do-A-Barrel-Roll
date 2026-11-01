fn barrel_roll_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::barrel_roll as barrel_roll;
    if mod_id != barrel_roll::ID {
        return;
    }
    let game = &*game;
    let settings = barrel_roll::settings(|key| game.native_setting(barrel_roll::ID, key));
    buttons.extend(settings.labels());
}
