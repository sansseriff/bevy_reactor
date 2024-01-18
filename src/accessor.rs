use bevy::prelude::*;

use crate::{Cx, ReactiveContext};

/// What type of reactive node underlies this signal. "Signals" in this framework represent
/// any kind of reactive data source, including mutable variables, derived signals, and memoized
/// computations.
pub enum SignalKind {
    /// A mutable variable that can be read and written to.
    Mutable,

    /// A readonly value that is computed from other signals.
    Derived,

    /// A memoized value that is computed from other signals.
    Memo,
}

/// Object that allows reading a signal using Copy semantics.
pub struct Getter<T> {
    pub(crate) id: Entity,
    pub(crate) kind: SignalKind,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T> Getter<T>
where
    T: Copy + Send + Sync + 'static,
{
    pub fn get<'p, R: ReactiveContext<'p>>(&self, rc: &R) -> T {
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
    pub fn get<'p, R: ReactiveContext<'p>>(&self, rc: &R) -> T {
        match self.kind {
            SignalKind::Mutable => rc.read_mutable_clone(self.id),
            SignalKind::Derived => unimplemented!(),
            SignalKind::Memo => unimplemented!(),
        }
    }
}

/// Trait that allows reading the value from a signal via an immutable reference.
// trait ReadSignalRef<T: Copy> {
//     fn as_ref(&self) -> &T;

//     // fn try_get(&mut self) -> Option<Self::Value>;
// }

/// Object that allows writing the value to a mutable, using Copy semantics.
pub struct Setter<T> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + Copy + PartialEq + 'static> Setter<T> {
    pub fn set<P>(&mut self, cx: &mut Cx<P>, value: T) {
        cx.write_mutable(self.id, value)
    }
}

/// Object that allows writing the value to a mutable, using Clone semantics.
pub struct CloneSetter<T> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + Clone + PartialEq + 'static> CloneSetter<T> {
    pub fn set<P>(&mut self, cx: &mut Cx<P>, value: T) {
        cx.write_mutable_clone(self.id, value)
    }
}

// Trait that allows access to a mutable reference to the signal.
// trait WriteSignalRef<T> {
//     fn write_ref<F: FnMut(&mut T)>(&mut self, f: F);
// }
