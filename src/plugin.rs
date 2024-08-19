use bevy::prelude::*;
use bevy_mod_stylebuilder::StyleBuilderPlugin;
use bevy_reactor_signals::SignalsPlugin;

use crate::{
    attach_child_views, build_added_view_roots, compositor::update_compositor_size,
    hover::update_hover_states,
};

/// Plugin that adds the reactive UI system to the app.
pub struct ReactorPlugin;

/// A system set that runs all the reactions.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReactionSet;

impl Plugin for ReactorPlugin {
    fn build(&self, app: &mut App) {
        app
            //.register_asset_loader(TextureAtlasLoader)
            .add_plugins((SignalsPlugin, StyleBuilderPlugin))
            .add_systems(
                Update,
                (
                    // run_view_reactions.in_set(ReactionSet),
                    build_added_view_roots.before(ReactionSet),
                    attach_child_views.after(ReactionSet),
                    // update_text_styles.after(attach_child_views),
                    update_hover_states,
                    update_compositor_size,
                ),
            );
    }
}

// / Run reactions whose dependencies have changed.
// pub fn run_view_reactions(world: &mut World) {
//     let mut scopes = world.query::<(Entity, &mut TrackingScope, &ViewHandle)>();
//     let mut changed = HashSet::<Entity>::default();
//     let tick = world.change_tick();
//     for (entity, scope, _) in scopes.iter(world) {
//         if scope.dependencies_changed(world, tick) {
//             changed.insert(entity);
//         }
//     }

//     // Record the changed entities for debugging purposes.
//     if let Some(mut tracing) = world.get_resource_mut::<TrackingScopeTracing>() {
//         if !changed.is_empty() {
//             tracing.0.extend(changed.iter().copied());
//         }
//     }

//     for scope_entity in changed.iter() {
//         // It's possible that an earlier reaction or cleanup deleted the entity.
//         if world.get_entity(*scope_entity).is_none() {
//             continue;
//         }

//         // Call registered cleanup functions
//         let (_, mut scope, _) = scopes.get_mut(world, *scope_entity).unwrap();
//         let mut cleanups = std::mem::take(&mut scope.cleanups);
//         for cleanup_fn in cleanups.drain(..) {
//             cleanup_fn(world);
//         }

//         // Run the reaction
//         let (_, _, view_handle) = scopes.get_mut(world, *scope_entity).unwrap();
//         let mut next_scope = TrackingScope::new(tick);
//         let inner = view_handle.0.clone();
//         inner
//             .lock()
//             .unwrap()
//             .react(*scope_entity, world, &mut next_scope);

//         // Replace deps and cleanups in the current scope with the next scope.
//         let (_, mut scope, _) = scopes.get_mut(world, *scope_entity).unwrap();
//         scope.take_deps(&mut next_scope);
//         scope.tick = tick;
//     }
// }
