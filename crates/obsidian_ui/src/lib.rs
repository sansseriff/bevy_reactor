//! An opinionated library of ui editor controls for Bevy.

#![warn(missing_docs)]

use bevy::{app::*, ui::UiMaterialPlugin};
use materials::{GradientRectMaterial, RoundedRectMaterial};

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

/// Module containing standard sizes.
pub mod size;

/// Module of utilities for embedding a 3D viewport in the 2D UI.
pub mod viewport;

/// Standard styles for fonts.
pub mod typography;

/// Plugin for the Obsidian UI library.
pub struct ObsidianUiPlugin;

impl Plugin for ObsidianUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UiMaterialPlugin::<RoundedRectMaterial>::default(),
            UiMaterialPlugin::<GradientRectMaterial>::default(),
            hooks::BistableTransitionPlugin,
            animation::AnimatedTransitionPlugin,
        ));
    }
}
