use bevy::prelude::*;

use crate::{
    attach_child_views, build_added_view_roots, mutable::commit_mutables,
    style::TextureAtlasLoader, tracking_scope::run_reactions, update_text_styles,
};

/// Plugin that adds the reactive UI system to the app.
pub struct ReactorPlugin;

impl Plugin for ReactorPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(TextureAtlasLoader).add_systems(
            Update,
            (
                commit_mutables,
                build_added_view_roots,
                run_reactions,
                attach_child_views,
                update_text_styles,
            )
                .chain(),
        );
    }
}
