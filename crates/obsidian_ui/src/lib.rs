//! An opinionated library of ui editor controls for Bevy.

#![warn(missing_docs)]

use bevy::{app::*, ui::UiMaterialPlugin};
use gradient_rect::GradientRectMaterial;
use rounded_rect::RoundedRectMaterial;

/// Module containing standard color definitions.
#[allow(missing_docs)]
pub mod colors;

/// Module containing interactive and layout control widgets.
pub mod controls;

/// Module containing standard sizes.
pub mod size;

/// Module of utilities for embedding a 3D viewport in the 2D UI.
pub mod viewport;

/// Standard styles for fonts.
pub mod typography;

mod gradient_rect;
mod rounded_rect;

/// Plugin for the Obsidian UI library.
pub struct ObsidianUiPlugin;

impl Plugin for ObsidianUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<RoundedRectMaterial>::default());
        app.add_plugins(UiMaterialPlugin::<GradientRectMaterial>::default());
    }
}
