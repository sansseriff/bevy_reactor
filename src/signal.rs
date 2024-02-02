use crate::{derived::ReadDerived, mutable::ReadMutable, Derived, Mutable};

/// What type of reactive node underlies this signal. "Signals" in this framework represent
/// any kind of reactive data source, including mutable variables, derived signals, and memoized
/// computations.
#[derive(Copy, Clone)]
pub enum Signal<T> {
    /// A mutable variable that can be read and written to.
    Mutable(Mutable<T>),

    /// A readonly value that is computed from other signals.
    Derived(Derived<T>),

    /// A memoized value that is computed from other signals.
    #[allow(dead_code)] // Not implemented yet.
    Memo,
}

impl<T> Signal<T>
where
    T: Copy + Send + Sync + 'static,
{
    /// Read the value of the signal using Copy semantics.
    pub fn get<R: ReadMutable + ReadDerived>(&self, rc: &R) -> T {
        match self {
            Signal::Mutable(mutable) => rc.read_mutable(mutable.id),
            Signal::Derived(derived) => rc.read_derived(derived.id),
            Signal::Memo => unimplemented!(),
        }
    }
}

/// Object that allows reading a signal using Clone semantics.
#[derive(Copy, Clone)]
pub enum SignalClone<T> {
    /// A mutable variable that can be read and written to.
    Mutable(Mutable<T>),

    /// A readonly value that is computed from other signals.
    Derived(Derived<T>),

    /// A memoized value that is computed from other signals.
    #[allow(dead_code)] // Not implemented yet.
    Memo,
}

impl<T> SignalClone<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Read the value of the signal using Clone semantics.
    pub fn get<R: ReadMutable + ReadDerived>(&self, rc: &R) -> T {
        match self {
            SignalClone::Mutable(mutable) => rc.read_mutable_clone(mutable.id),
            SignalClone::Derived(derived) => rc.read_derived_clone(derived.id),
            SignalClone::Memo => unimplemented!(),
        }
    }
}
