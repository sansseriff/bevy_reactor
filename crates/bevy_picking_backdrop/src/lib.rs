//! A backend for `bevy_mod_picking` that always returns a hit.
//!
//! # Usage
//!
//! This backend always registers a hit on the designated backdrop entity, but it's order
//! is set to be lower than the camera's order, so it will not interfere with other hits.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_mod_picking::{
    backend::{ray::RayMap, HitData, PointerHits},
    picking_core::PickSet,
};
// use bevy_app::prelude::*;
// use bevy_ecs::prelude::*;
// use bevy_reflect::prelude::*;
// use bevy_render::prelude::*;

// use bevy_picking_core::backend::prelude::*;

/// Marks a camera that should be used in the backdrop picking backend.
/// Also marks the entity which is used as the backdrop.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct BackdropPickable;

/// Adds the raycasting picking backend to your app.
#[derive(Clone)]
pub struct BackdropBackend;
impl Plugin for BackdropBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_hits.in_set(PickSet::Backend))
            .register_type::<BackdropPickable>();
    }
}

/// Returns a hit on the camera backdrop.
pub fn update_hits(
    ray_map: Res<RayMap>,
    picking_cameras: Query<&Camera, With<BackdropPickable>>,
    picking_backdrop: Query<(Entity, &BackdropPickable), Without<Camera>>,
    mut output_events: EventWriter<PointerHits>,
) {
    let backdrop = picking_backdrop.get_single().unwrap();

    for (&ray_id, &_ray) in ray_map.map().iter() {
        let Ok(camera) = picking_cameras.get(ray_id.camera) else {
            continue;
        };

        let hit_data = HitData::new(ray_id.camera, f32::MAX, None, None);
        let picks = Vec::from([(backdrop.0, hit_data)]);
        let order = camera.order as f32 - 1.0;
        output_events.send(PointerHits::new(ray_id.pointer, picks, order));
    }
}
