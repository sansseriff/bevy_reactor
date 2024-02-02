use bevy::prelude::*;

use crate::{
    attach_child_views,
    build_added_view_roots,
    // callback::{run_deferred_callbacks, DeferredCall},
    mutable::commit_mutables,
    style::TextureAtlasLoader,
    tracking_scope::run_reactions,
    update_text_styles,
};

/// Plugin that adds the reactive UI system to the app.
pub struct ReactorPlugin;

impl Plugin for ReactorPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(TextureAtlasLoader)
            // .add_event::<DeferredCall<f32>>()
            .add_systems(
                Update,
                (
                    // run_deferred_callbacks::<f32>,
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
