use crate::{
    signal::{CloneGetter, Signal, SignalKind},
    ReactiveContext, ReactiveContextMut,
};
use bevy::prelude::*;
use std::any::Any;

// TODO: We could make this component generic over the type of the value. This would mean:
//
// * We would have to use a different component for each type of mutable.
// * No need to box the value.
// * No need to use Any.
// * We'd need to store the component id in the Mutable so that the tracking scope can know
//   which component to access.
// * TrackingScope could treat it just like any other component.
// * We would not be able to migrate to using atomic bools for change flags. (This is contemplated
//   as a means of doing converging reactions in a single update).

/// Contains a mutable reactive value.
#[derive(Component)]
pub(crate) struct MutableValue {
    pub(crate) value: Box<dyn Any + Send + Sync + 'static>,
}

/// Contains the value which will be written to the signal on the next update.
/// This is used to avoid writing to the signal multiple times in a single frame, and also
/// ensures that the signal values remain stable during a reaction.
#[derive(Component)]
pub(crate) struct MutableValueNext(pub(crate) Box<dyn Any + Send + Sync + 'static>);

/// Contains a reference to a reactive mutable variable.
#[derive(Copy, Clone)]
pub struct Mutable<T> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T> Mutable<T>
where
    T: PartialEq + Copy + Send + Sync + 'static,
{
    /// Returns a getter and setter for this [`Mutable`] with Copy semantics.
    pub fn signal(&self) -> Signal<T> {
        Signal {
            id: self.id,
            kind: SignalKind::Mutable,
            marker: std::marker::PhantomData,
        }
    }

    /// Get the value of this [`Mutable`] with Copy semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn get<'p, R: ReactiveContext<'p>>(&self, cx: &mut R) -> T {
        cx.read_mutable(self.id)
    }

    /// Set the value of this [`Mutable`] with Copy semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    /// * `value`: The new value.
    pub fn set<'p, R: ReactiveContextMut<'p>>(&self, cx: &mut R, value: T) {
        cx.write_mutable(self.id, value);
    }
}

impl<T> Mutable<T>
where
    T: PartialEq + Clone + Send + Sync + 'static,
{
    /// Returns a getter and setter for this [`Mutable`] with Clone semantics.
    pub fn signal_clone(&self) -> CloneGetter<T> {
        CloneGetter {
            id: self.id,
            kind: SignalKind::Mutable,
            marker: std::marker::PhantomData,
        }
    }

    /// Get the value of this [`Mutable`] with Clone semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn get_clone<'p, R: ReactiveContext<'p>>(&self, cx: &mut R) -> T {
        cx.read_mutable_clone(self.id)
    }

    /// Set the value of this [`Mutable`] with Clone semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    /// * `value`: The new value.
    pub fn set_clone<'p, R: ReactiveContextMut<'p>>(&self, cx: &mut R, value: T) {
        cx.write_mutable_clone(self.id, value);
    }
}

/// Trait that allows access to a mutable reference to the signal.
// trait WriteSignalRef<T> {
//     fn write_ref<F: FnMut(&mut T)>(&mut self, f: F);
// }

pub(crate) fn commit_mutables(world: &mut World) {
    for (mut sig_val, mut sig_next) in world
        .query::<(&mut MutableValue, &mut MutableValueNext)>()
        .iter_mut(world)
    {
        // Transfer mutable data from next to current.
        std::mem::swap(&mut sig_val.value, &mut sig_next.0);
        // sig_val
        //     .changed
        //     .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    // Remove all the MutableNext components.
    let mutables: Vec<Entity> = world
        .query_filtered::<Entity, With<MutableValueNext>>()
        .iter(world)
        .collect();
    mutables.iter().for_each(|mutable| {
        world.entity_mut(*mutable).remove::<MutableValueNext>();
    });
}

#[cfg(test)]
mod tests {
    use crate::{cx::Cx, SetupContext, TrackingScope};

    use super::*;

    #[test]
    fn test_mutable_copy() {
        let mut world = World::default();
        let mut scope = TrackingScope::new(world.change_tick());
        let mut cx = Cx::new(&(), &mut world, &mut scope);

        let mutable = cx.create_mutable::<i32>(0);
        let reader = mutable.signal();
        let reader2 = cx.create_mutable::<i32>(0).signal();

        // Check initial values
        assert_eq!(reader.get(&cx), 0);
        assert_eq!(reader2.get(&cx), 0);

        // Update signals
        mutable.set(&mut cx, 1);

        // Values should not have changed yet
        assert_eq!(reader.get(&cx), 0);
        assert_eq!(reader2.get(&cx), 0);

        // Now commit the changes
        commit_mutables(&mut world);

        // Signals should have changed
        let cx = Cx::new(&(), &mut world, &mut scope);
        assert_eq!(reader.get(&cx), 1);
        assert_eq!(reader2.get(&cx), 0);
    }

    #[test]
    fn test_mutable_clone() {
        let mut world = World::default();
        let mut scope = TrackingScope::new(world.change_tick());
        let mut cx = Cx::new(&(), &mut world, &mut scope);

        let mutable = cx.create_mutable("Hello".to_string());
        let reader = mutable.signal_clone();
        let reader2 = cx.create_mutable::<i32>(0).signal_clone();

        // Check initial values
        assert_eq!(reader.get(&cx), "Hello".to_string());
        assert_eq!(reader2.get(&cx), 0);

        // Update signals
        mutable.set_clone(&mut cx, "Goodbye".to_string());

        // Values should not have changed yet
        assert_eq!(reader.get(&cx), "Hello".to_string());
        assert_eq!(reader2.get(&cx), 0);

        // Now commit the changes
        commit_mutables(&mut world);

        // Signals should have changed
        let cx = Cx::new(&(), &mut world, &mut scope);
        assert_eq!(reader.get(&cx), "Goodbye".to_string());
        assert_eq!(reader2.get(&cx), 0);
    }
}
