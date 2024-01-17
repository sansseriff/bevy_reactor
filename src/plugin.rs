use bevy::prelude::*;

use crate::{build_added_views, mutable::commit_mutables, scope::run_reactions};

pub struct SignalsPlugin;

impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (commit_mutables, build_added_views, run_reactions).chain(),
        );
    }
}
