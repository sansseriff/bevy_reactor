//! An opinionated library of ui editor controls for Bevy.

#![warn(missing_docs)]

use bevy::{app::*, ui::UiMaterialPlugin};
use bevy_mod_picking::prelude::EventListenerPlugin;
use materials::{GradientRectMaterial, RoundedRectMaterial, SliderRectMaterial};

/// Utilities for animation.
pub mod animation;

/// Module containing standard color definitions.
#[allow(missing_docs)]
pub mod colors;

/// Module containing interactive and layout control widgets.
pub mod controls;

/// Module containing extensions to `Cx`.
pub mod hooks;

/// Module containing custom materials.
pub mod materials;
pub use materials::RoundedCorners;

/// Module defining utilities for interactive overlays.
pub mod overlays;

/// Utilities for managing scrolling views.
pub mod scrolling;

/// Module containing standard sizes.
pub mod size;

/// Module of utilities for embedding a 3D viewport in the 2D UI.
pub mod viewport;

/// Utilities for tabbing between widgets.
pub mod focus;

/// Standard styles for fonts.
pub mod typography;

/// Plugin for the Obsidian UI library.
pub struct ObsidianUiPlugin;

use scrolling::ScrollWheel;

impl Plugin for ObsidianUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UiMaterialPlugin::<RoundedRectMaterial>::default(),
            UiMaterialPlugin::<GradientRectMaterial>::default(),
            UiMaterialPlugin::<SliderRectMaterial>::default(),
            hooks::BistableTransitionPlugin,
            animation::AnimatedTransitionPlugin,
            focus::KeyboardInputPlugin,
            overlays::OverlaysPlugin,
        ))
        .add_plugins(EventListenerPlugin::<ScrollWheel>::default())
        .add_event::<ScrollWheel>()
        .add_systems(
            Update,
            (
                scrolling::handle_scroll_events,
                scrolling::update_scroll_positions,
            ),
        );
    }
}
