mod mesh_builder;
mod overlay;
mod overlay_material;
mod shape_builder;

use bevy::{app::Plugin, pbr::MaterialPlugin};
pub use overlay::Overlay;
pub use shape_builder::{PolygonOptions, ShapeBuilder, StrokeMarker};

use self::overlay_material::UnderlayMaterial;

/// Plugin for the overlays module.
pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(MaterialPlugin::<UnderlayMaterial>::default());
    }
}

/// Overlay that builds flat shapes.
pub type OverlayShape = Overlay<ShapeBuilder>;
