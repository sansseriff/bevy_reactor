use bevy::prelude::*;

use crate::{build_added_view_roots, mutable::commit_mutables, scope::run_reactions};

/// Plugin that adds the reactive UI system to the app.
pub struct ReactorPlugin;

impl Plugin for ReactorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (commit_mutables, build_added_view_roots, run_reactions).chain(),
        );
    }
}
