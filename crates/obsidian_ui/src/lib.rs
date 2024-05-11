//! An opinionated library of ui editor controls for Bevy.

#![warn(missing_docs)]

use bevy::{app::*, ui::UiMaterialPlugin};
use bevy_mod_picking::prelude::EventListenerPlugin;
use controls::MenuCloseEvent;
use materials::{
    DotGridMaterial, DrawPathMaterial, GradientRectMaterial, SliderRectMaterial, SwatchRectMaterial,
};

/// Utilities for animation.
pub mod animation;

/// Module containing standard color definitions.
#[allow(missing_docs)]
pub mod colors;

/// Module containing interactive and layout control widgets.
pub mod controls;

/// Utilities for tabbing between widgets.
pub mod focus;

/// Utilities for floating popups.
pub mod floating;

/// Module containing extensions to `Cx`.
pub mod hooks;

/// Module containing custom materials.
pub mod materials;

/// Utilities for managing scrolling views.
pub mod scrolling;

/// Module containing standard sizes.
pub mod size;

/// Module of utilities for embedding a 3D viewport in the 2D UI.
pub mod viewport;

/// Standard styles for fonts.
pub mod typography;

/// Plugin for the Obsidian UI library.
pub struct ObsidianUiPlugin;

use scrolling::ScrollWheel;

mod rounded_corners;
pub use rounded_corners::RoundedCorners;

impl Plugin for ObsidianUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UiMaterialPlugin::<GradientRectMaterial>::default(),
            UiMaterialPlugin::<SliderRectMaterial>::default(),
            UiMaterialPlugin::<SwatchRectMaterial>::default(),
            UiMaterialPlugin::<DotGridMaterial>::default(),
            UiMaterialPlugin::<DrawPathMaterial>::default(),
            hooks::BistableTransitionPlugin,
            animation::AnimatedTransitionPlugin,
            focus::KeyboardInputPlugin,
        ))
        .add_plugins((
            EventListenerPlugin::<ScrollWheel>::default(),
            EventListenerPlugin::<MenuCloseEvent>::default(),
        ))
        .add_event::<ScrollWheel>()
        .add_systems(
            Update,
            (
                scrolling::handle_scroll_events,
                scrolling::update_scroll_positions,
            ),
        )
        .add_systems(PostUpdate, floating::position_floating);
    }
}
