#![allow(missing_docs)]
//! Defines fluent builder for styles.

use bevy::{asset::AssetPath, prelude::*, ui};

/// A builder object that provides a fluent interface for defining styles for bevy_ui nodes.
/// Most components such as `BackgroundColor` are mutated directly, however some component types
/// such as `Node` are cached in the builder and not applied until `finish` is called.
/// This is similar to the `StyleBuilder` but provides a commands-based interface.
pub struct StyleCommands<'a, 'w> {
    pub target: &'a mut EntityCommands<'w>,
    pub(crate) asset_server: &'a AssetServer,
    pub(crate) style: ui::Node,
    pub(crate) style_changed: bool,
}

impl<'a, 'w> StyleCommands<'a, 'w> {
    /// Construct a new StyleBuilder instance.
    pub fn new(
        target: &'a mut EntityCommands<'w>,
        asset_server: &'a AssetServer,
        style: ui::Node,
    ) -> Self {
        Self {
            target,
            asset_server,
            style,
            style_changed: false,
        }
    }

    /// Helper method for loading assets.
    pub fn load_asset<A: Asset>(&mut self, path: AssetPath<'_>) -> Handle<A> {
        self.asset_server.load(path)
    }

    /// Consumes the [`StyleBuilder`] and applies the style to the target entity.
    pub fn finish(self) {
        if self.style_changed {
            self.target.insert(self.style);
        }
    }
}

// LineBreak(BreakLineOn),
