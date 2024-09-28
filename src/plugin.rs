use bevy::prelude::*;

/// Plugin that adds the reactive UI system to the app.
pub struct ReactorPlugin;

/// A system set that runs all the reactions.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReactionSet;

impl Plugin for ReactorPlugin {
    fn build(&self, _app: &mut App) {}
}
