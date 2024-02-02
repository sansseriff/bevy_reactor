use bevy::prelude::*;

use crate::{derived::ReadDerived, mutable::ReadMutable};

/// What type of reactive node underlies this signal. "Signals" in this framework represent
/// any kind of reactive data source, including mutable variables, derived signals, and memoized
/// computations.
#[derive(Copy, Clone)]
pub enum SignalKind {
    /// A mutable variable that can be read and written to.
    Mutable,

    /// A readonly value that is computed from other signals.
    #[allow(dead_code)] // Not implemented yet.
    Derived,

    /// A memoized value that is computed from other signals.
    #[allow(dead_code)] // Not implemented yet.
    Memo,
}

/// Object that allows reading a signal source using Copy semantics.
#[derive(Copy, Clone)]
pub struct Signal<T> {
    pub(crate) id: Entity,
    pub(crate) kind: SignalKind,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T> Signal<T>
where
    T: Copy + Send + Sync + 'static,
{
    /// Read the value of the signal using Copy semantics.
    pub fn get<R: ReadMutable + ReadDerived>(&self, rc: &R) -> T {
        match self.kind {
            SignalKind::Mutable => rc.read_mutable(self.id),
            SignalKind::Derived => rc.read_derived(self.id),
            SignalKind::Memo => unimplemented!(),
        }
    }
}

/// Object that allows reading a signal using Clone semantics.
#[derive(Copy, Clone)]
pub struct SignalClone<T> {
    pub(crate) id: Entity,
    pub(crate) kind: SignalKind,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T> SignalClone<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Read the value of the signal using Clone semantics.
    pub fn get<R: ReadMutable + ReadDerived>(&self, rc: &R) -> T {
        match self.kind {
            SignalKind::Mutable => rc.read_mutable_clone(self.id),
            SignalKind::Derived => rc.read_derived_clone(self.id),
            SignalKind::Memo => unimplemented!(),
        }
    }
}
