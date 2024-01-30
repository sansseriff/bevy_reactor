use bevy::prelude::*;

use crate::RunContextRead;

/// What type of reactive node underlies this signal. "Signals" in this framework represent
/// any kind of reactive data source, including mutable variables, derived signals, and memoized
/// computations.
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
pub struct Signal<T> {
    pub(crate) id: Entity,
    pub(crate) kind: SignalKind,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T> Signal<T>
where
    T: Copy + Send + Sync + 'static,
{
    pub fn get<'p, R: RunContextRead<'p>>(&self, rc: &R) -> T {
        match self.kind {
            SignalKind::Mutable => rc.read_mutable(self.id),
            SignalKind::Derived => unimplemented!(),
            SignalKind::Memo => unimplemented!(),
        }
    }
}

/// Object that allows reading a signal using Clone semantics.
pub struct CloneGetter<T> {
    pub(crate) id: Entity,
    pub(crate) kind: SignalKind,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T> CloneGetter<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn get<'p, R: RunContextRead<'p>>(&self, rc: &R) -> T {
        match self.kind {
            SignalKind::Mutable => rc.read_mutable_clone(self.id),
            SignalKind::Derived => unimplemented!(),
            SignalKind::Memo => unimplemented!(),
        }
    }
}
