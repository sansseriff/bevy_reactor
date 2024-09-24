use bevy_reactor_signals::{Rcx, Signal};

/// Trait that abstracts over the boolean condition that controls the If. We use this trait
/// to allow boolean signals to be passed directly as conditions.
pub trait TestCondition: Send + Sync {
    fn test(&self, rcx: &Rcx) -> bool;
}

impl<F: Send + Sync + Fn(&Rcx) -> bool> TestCondition for F {
    fn test(&self, rcx: &Rcx) -> bool {
        self(rcx)
    }
}

impl TestCondition for bool {
    fn test(&self, _rcx: &Rcx) -> bool {
        *self
    }
}

impl TestCondition for Signal<bool> {
    fn test(&self, rcx: &Rcx) -> bool {
        self.get(rcx)
    }
}
