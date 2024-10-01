use bevy::app::{Plugin, Startup};
use bevy_mod_stylebuilder::StyleBuilderPlugin;
use bevy_reactor_obsidian::ObsidianUiPlugin;
use bevy_reactor_signals::SignalsPlugin;
use panel::create_inspector_panel;

mod panel;

pub struct WorldInspector;

impl Plugin for WorldInspector {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((SignalsPlugin, StyleBuilderPlugin, ObsidianUiPlugin))
            .add_systems(Startup, create_inspector_panel);
    }
}
