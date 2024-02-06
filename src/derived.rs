use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    signal::{Signal, SignalClone},
    Rcx, TrackingScope,
};

pub(crate) trait DerivedFnRef<R> {
    fn call(&self, cx: &mut Rcx) -> R;
}

impl<R, F: Fn(&mut Rcx) -> R> DerivedFnRef<R> for F {
    fn call(&self, cx: &mut Rcx) -> R {
        self(cx)
    }
}

/// Contains a boxed, type-erased function which returns a reactive result.
#[derive(Component)]
pub(crate) struct DerivedCell<R>(pub(crate) Arc<dyn DerivedFnRef<R> + Send + Sync>);

/// A [`Derived`] is a readonly value that is computed from other signals.
#[derive(Copy, PartialEq)]
pub struct Derived<R> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<R>,
}

impl<T> Clone for Derived<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            marker: self.marker,
        }
    }
}

impl<R> Derived<R>
where
    R: PartialEq + Copy + Send + Sync + 'static,
{
    /// Returns a getter for this [`Derived`] with Copy semantics.
    pub fn signal(&self) -> Signal<R> {
        Signal::Derived(*self)
    }

    /// Get the value of this [`Derived`] with Copy semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn get<Rc: ReadDerived>(&self, cx: &mut Rc) -> R {
        cx.read_derived(self.id)
    }
}

impl<R> Derived<R>
where
    R: PartialEq + Clone + Send + Sync + 'static,
{
    /// Returns a getter for this [`Derived`] with Clone semantics.
    pub fn signal_clone(self) -> SignalClone<R> {
        SignalClone::Derived(self)
    }

    /// Get the value of this [`Derived`] with Clone semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn get_clone<Rc: ReadDerived>(&self, cx: &mut Rc) -> R {
        cx.read_derived_clone(self.id)
    }
}

/// An immutable reactive context, used for reactive closures such as derived signals.
pub trait ReadDerived {
    /// Read the value of a derived signal using Copy semantics. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived<R>(&self, derived: Entity) -> R
    where
        R: Send + Sync + Copy + 'static;

    /// Read the value of a mutable variable using Copy semantics. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_clone<R>(&self, derived: Entity) -> R
    where
        R: Send + Sync + Clone + 'static;

    /// Read the value of a mutable variable using a mapping function. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_map<R, U, F: Fn(&R) -> U>(&self, derived: Entity, f: F) -> U
    where
        R: Send + Sync + 'static;
}

/// Trait used to implement reading of derives while passing an explicit tracking scope.
pub(crate) trait ReadDerivedInternal {
    /// Read the value of a derived signal using Copy semantics. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_with_scope<R>(&self, derived: Entity, scope: &mut TrackingScope) -> R
    where
        R: Send + Sync + Copy + 'static;

    /// Read the value of a mutable variable using Copy semantics. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_clone_with_scope<R>(&self, derived: Entity, scope: &mut TrackingScope) -> R
    where
        R: Send + Sync + Clone + 'static;

    /// Read the value of a mutable variable using a mapping function. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_map_with_scope<R, U, F: Fn(&R) -> U>(
        &self,
        derived: Entity,
        scope: &mut TrackingScope,
        f: F,
    ) -> U
    where
        R: Send + Sync + 'static;
}
