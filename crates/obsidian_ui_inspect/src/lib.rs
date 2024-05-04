mod default_factory;
mod edit_bool;
mod edit_color;
mod edit_fallback;
mod field_label;
mod inspectable;
mod inspector;
mod inspector_factory;

use bevy::app::{App, Plugin};
use default_factory::DefaultInspectorFactory;

pub use inspectable::*;
pub use inspector::*;
pub use inspector_factory::*;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspector::<DefaultInspectorFactory>();
    }
}
