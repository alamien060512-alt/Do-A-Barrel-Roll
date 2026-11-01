fn barrel_roll_camera(game: &crate::game::Game<'_>, value: glam::Mat4) -> glam::Mat4 {
    use crate::game::barrel_roll as barrel_roll;
    if !game.native_on(barrel_roll::ID) {
        return value;
    }
    let roll = barrel_roll::camera_roll();
    if roll == 0.0 {
        return value;
    }
    glam::Mat4::from_rotation_z(roll.to_radians()) * value
}
