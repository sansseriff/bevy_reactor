#![allow(missing_docs)]
//! Defines fluent builder for styles.

use bevy::{asset::AssetPath, prelude::*, ui};

/// An object that provides a fluent interface for defining styles for bevy_ui nodes.
/// Most components such as `BackgroundColor` are mutated immediately, however some component types
/// such as `Style` are cached in the builder and not applied until `finish` is called.
pub struct StyleBuilder<'a, 'w> {
    pub target: &'a mut EntityWorldMut<'w>,
    pub(crate) style: ui::Node,
    pub(crate) style_changed: bool,
}

impl<'a, 'w> StyleBuilder<'a, 'w> {
    /// Construct a new StyleBuilder instance.
    pub fn new(target: &'a mut EntityWorldMut<'w>, style: ui::Node) -> Self {
        Self {
            target,
            style,
            style_changed: false,
        }
    }

    /// Helper method for loading assets.
    pub fn load_asset<A: Asset>(&mut self, path: AssetPath<'_>) -> Handle<A> {
        self.target.world_scope(|world| {
            let server = world.get_resource::<AssetServer>().unwrap();
            server.load(path)
        })
    }

    /// Consumes the [`StyleBuilder`] and applies the style to the target entity.
    pub fn finish(self) {
        if self.style_changed {
            self.target.insert(self.style);
        }
    }
}

// LineBreak(BreakLineOn),
