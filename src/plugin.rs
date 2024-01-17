use bevy::prelude::*;

use crate::{build_view_handles, mutable::commit_mutables, scope::run_reactions};

pub struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (commit_mutables, build_view_handles, run_reactions).chain(),
        );
    }
}
