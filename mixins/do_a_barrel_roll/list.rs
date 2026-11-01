fn barrel_roll_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::barrel_roll as barrel_roll;
    mods.push(crate::game::native_hooks::NativeMod {
        id: barrel_roll::ID.into(),
        title: format!("Do A Barrel Roll {}", barrel_roll::VERSION),
        description: barrel_roll::DESCRIPTION.into(),
        keys: vec![],
    });
}
