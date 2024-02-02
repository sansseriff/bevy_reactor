use crate::{
    signal::{Signal, SignalClone},
    RunContextWrite,
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
// * The hard part is handling MutableValueNext, because that is processed via a query.
// * We would need to register a system for each specialization.

/// Contains a mutable reactive value.
#[derive(Component)]
pub(crate) struct MutableCell(pub(crate) Box<dyn Any + Send + Sync + 'static>);

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
    T: PartialEq + Send + Sync + 'static,
{
    /// Update a mutable value in place using a callback. The callback is passed a
    /// `Mut<T>` which can be used to modify the value.
    pub fn update<R: RunContextWrite, F: FnOnce(Mut<T>)>(&mut self, cx: &mut R, updater: F) {
        let value = cx.world_mut().get_mut::<MutableCell>(self.id).unwrap();
        let inner = value.map_unchanged(|v| v.0.downcast_mut::<T>().unwrap());
        (updater)(inner);
    }
}

impl<T> Mutable<T>
where
    T: PartialEq + Copy + Send + Sync + 'static,
{
    /// Returns a getter for this [`Mutable`] with Copy semantics.
    pub fn signal(&self) -> Signal<T> {
        Signal::Mutable(*self)
    }

    /// Get the value of this [`Mutable`] with Copy semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn get<R: ReadMutable>(&self, cx: &mut R) -> T {
        cx.read_mutable(self.id)
    }

    /// Set the value of this [`Mutable`] with Copy semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    /// * `value`: The new value.
    pub fn set<R: WriteMutable>(&self, cx: &mut R, value: T) {
        cx.write_mutable(self.id, value);
    }
}

impl<T> Mutable<T>
where
    T: PartialEq + Clone + Send + Sync + 'static,
{
    /// Returns a getter for this [`Mutable`] with Clone semantics.
    pub fn signal_clone(&self) -> SignalClone<T> {
        SignalClone::Mutable(self.clone())
    }

    /// Get the value of this [`Mutable`] with Clone semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn get_clone<R: ReadMutable>(&self, cx: &mut R) -> T {
        cx.read_mutable_clone(self.id)
    }

    /// Set the value of this [`Mutable`] with Clone semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    /// * `value`: The new value.
    pub fn set_clone<R: WriteMutable>(&self, cx: &mut R, value: T) {
        cx.write_mutable_clone(self.id, value);
    }
}

/// Trait for low-level read-access to mutables given an entity id.
pub trait ReadMutable {
    /// Read the value of a mutable variable using Copy semantics. Calling this function adds the
    /// mutable to the current tracking scope.
    fn read_mutable<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Copy + 'static;

    /// Read the value of a mutable variable using Clone semantics. Calling this function adds the
    /// mutable to the current tracking scope.
    fn read_mutable_clone<T>(&self, mutable: Entity) -> T
    where
        T: Send + Sync + Clone + 'static;
}

/// Trait for low-level write-access to mutables given an entity id.
pub trait WriteMutable {
    /// Write the value of a mutable variable using Copy semantics. Does nothing if
    /// the value being set matches the existing value.
    fn write_mutable<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Copy + PartialEq + 'static;

    /// Write the value of a mutable variable using Clone semantics. Does nothing if the
    /// value being set matches the existing value.
    fn write_mutable_clone<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Clone + PartialEq + 'static;
}

pub(crate) fn commit_mutables(world: &mut World) {
    for (mut sig_val, mut sig_next) in world
        .query::<(&mut MutableCell, &mut MutableValueNext)>()
        .iter_mut(world)
    {
        // Transfer mutable data from next to current.
        std::mem::swap(&mut sig_val.0, &mut sig_next.0);
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

// struct MutablePlugin<T: Send + Sync + 'static> {
//     _marker: std::marker::PhantomData<T>,
// }

// impl<T: Send + Sync + 'static> MutablePlugin<T> {
//     pub(crate) fn commit_mutables(world: &mut World) {
//         for (mut sig_val, mut sig_next) in world
//             .query::<(&mut MutableCell, &mut MutableValueNext)>()
//             .iter_mut(world)
//         {
//             // Transfer mutable data from next to current.
//             std::mem::swap(&mut sig_val.0, &mut sig_next.0);
//         }

//         // Remove all the MutableNext components.
//         let mutables: Vec<Entity> = world
//             .query_filtered::<Entity, With<MutableValueNext>>()
//             .iter(world)
//             .collect();
//         mutables.iter().for_each(|mutable| {
//             world.entity_mut(*mutable).remove::<MutableValueNext>();
//         });
//     }
// }

// impl<T: Send + Sync + 'static> Plugin for MutablePlugin<T> {
//     fn build(&self, app: &mut App) {
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{cx::Cx, RunContextSetup, TrackingScope};

    use super::*;

    #[test]
    fn test_mutable_copy() {
        let mut world = World::default();
        let mut scope = TrackingScope::new(world.read_change_tick());
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
        let mut scope = TrackingScope::new(world.read_change_tick());
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
