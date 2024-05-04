use std::sync::Arc;

use bevy::{prelude::*, reflect::ParsedPath};
use bevy_reactor::*;

/// Trait that represents an item that can be inspected
#[allow(unused_variables)]
pub trait Inspectable: Send + Sync {
    /// The name of the item being inspected
    fn name(&self, cx: &Cx) -> String;

    /// The reflect data for the item being inspected
    fn reflect<'a>(&self, cx: &'a Cx) -> &'a dyn Reflect;

    /// The reflect data for a field within the item.
    fn reflect_field<'a>(&self, cx: &'a Cx, path: &ParsedPath) -> &'a dyn Reflect;

    /// Update a field within the item
    fn get_field<'a>(&self, cx: &'a Rcx, path: &ParsedPath) -> &'a dyn Reflect;

    /// Update a field within the item
    fn set_field(&self, cx: &mut Cx, path: &ParsedPath, value: &dyn Reflect);
}

/// A resource that can be inspected
pub struct InspectableResource<T: Resource + Reflect> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Resource + Reflect> Default for InspectableResource<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: Resource + Reflect> Inspectable for InspectableResource<T> {
    fn name(&self, cx: &Cx) -> String {
        let res = cx.use_resource::<T>();
        res.reflect_short_type_path().to_string()
    }

    fn reflect<'a>(&self, cx: &'a Cx) -> &'a dyn Reflect {
        cx.use_resource::<T>().as_reflect()
    }

    fn reflect_field<'a>(&self, cx: &'a Cx, path: &ParsedPath) -> &'a dyn Reflect {
        let res = cx.use_resource::<T>();
        res.reflect_path(path).expect("Invalid path")
    }

    fn get_field<'a>(&self, cx: &'a Rcx, path: &ParsedPath) -> &'a dyn Reflect {
        let res = cx.use_resource::<T>();
        res.reflect_path(path).expect("Invalid path")
    }

    fn set_field(&self, cx: &mut Cx, path: &ParsedPath, value: &dyn Reflect) {
        let mut res = cx.world_mut().get_resource_mut::<T>().unwrap();
        res.reflect_path_mut(path)
            .expect("Invalid path")
            .apply(value);
    }
}

/// A reference to a field within an `Inspectable`. This contains information needed to
/// get and set the field as well as query it's type.
#[derive(Clone)]
pub struct InspectableField {
    pub(crate) root: Arc<dyn Inspectable>,
    pub(crate) name: String,
    pub(crate) path: ParsedPath,
    pub(crate) can_remove: bool,
}

impl InspectableField {
    /// Return the name of this field.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Update the value of the field
    pub fn reflect<'a>(&self, cx: &'a Cx) -> &'a dyn Reflect {
        self.root.reflect_field(cx, &self.path)
    }

    /// Update the value of the field
    pub fn get_value<'a>(&self, cx: &'a Rcx) -> &'a dyn Reflect {
        self.root.get_field(cx, &self.path)
    }

    /// Update the value of the field
    pub fn set_value(&self, cx: &mut Cx, value: &dyn Reflect) {
        self.root.set_field(cx, &self.path, value);
    }

    /// Whether the item can be removed (in other words, is it optional or an array element)
    pub fn can_remove(&self) -> bool {
        self.can_remove
    }

    /// Remove the value from the parent
    pub fn remove(&self, _cx: &mut Cx) {}
}
