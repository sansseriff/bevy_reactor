use bevy::app::{Plugin, Startup, Update};
use bevy_mod_stylebuilder::StyleBuilderPlugin;
use bevy_reactor_obsidian::ObsidianUiPlugin;
use bevy_reactor_signals::SignalsPlugin;
use inspector_panel::{copy_top_level_entities, create_inspector_panel, TopLevelEntities};

mod inspector_panel;

pub struct WorldInspector;

impl Plugin for WorldInspector {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<TopLevelEntities>()
            .add_plugins((SignalsPlugin, StyleBuilderPlugin, ObsidianUiPlugin))
            .add_systems(Startup, create_inspector_panel)
            .add_systems(Update, copy_top_level_entities);
    }
}
