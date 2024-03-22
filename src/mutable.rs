use crate::{signal::Signal, RunContextWrite};
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
// * What's missing: a way to issue commands from World. With that, we wouldn't need a system
//   for each specialization, we wouldn't need a system at all.

/// Contains a mutable reactive value.
#[derive(Component)]
pub(crate) struct MutableCell(pub(crate) Box<dyn Any + Send + Sync + 'static>);

/// Contains the value which will be written to the signal on the next update.
/// This is used to avoid writing to the signal multiple times in a single frame, and also
/// ensures that the signal values remain stable during a reaction.
#[derive(Component)]
pub(crate) struct MutableNextCell(pub(crate) Option<Box<dyn Any + Send + Sync + 'static>>);

/// Contains a reference to a reactive mutable variable.
#[derive(PartialEq)]
pub struct Mutable<T> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T> Mutable<T> {
    /// The entity that holds the mutable value.
    pub fn id(&self) -> Entity {
        self.id
    }
}

impl<T> Copy for Mutable<T> {}
impl<T> Clone for Mutable<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Mutable<T>
where
    T: PartialEq + Send + Sync + 'static,
{
    /// Update a mutable value in place using a callback. The callback is passed a
    /// `Mut<T>` which can be used to modify the value.
    pub fn update<R: RunContextWrite, F: FnOnce(Mut<T>)>(&self, cx: &mut R, updater: F) {
        let value = cx.world_mut().get_mut::<MutableCell>(self.id).unwrap();
        let inner = value.map_unchanged(|v| v.0.downcast_mut::<T>().unwrap());
        (updater)(inner);
    }
}

impl<T> Mutable<T>
where
    T: PartialEq + Send + Sync + 'static,
{
    /// Returns a signal for this [`Mutable`] with Copy semantics.
    pub fn signal(&self) -> Signal<T> {
        Signal::Mutable(*self)
    }

    /// Get a reference to the value of this [`Mutable`].
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn as_ref<'a, 'b: 'a, R: ReadMutable>(&'a self, cx: &'b mut R) -> &'a T {
        cx.read_mutable_as_ref(self)
    }
}

impl<T> Mutable<T>
where
    T: PartialEq + Copy + Send + Sync + 'static,
{
    /// Get the value of this [`Mutable`] with Copy semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn get<R: ReadMutable>(&self, cx: &R) -> T {
        cx.read_mutable(self)
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
    /// Get the value of this [`Mutable`] with Clone semantics.
    ///
    /// Arguments:
    /// * `cx`: The reactive context.
    pub fn get_clone<R: ReadMutable>(&self, cx: &mut R) -> T {
        cx.read_mutable_clone(self)
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
    fn read_mutable<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Copy + 'static;

    /// Read the value of a mutable variable using Clone semantics. Calling this function adds the
    /// mutable to the current tracking scope.
    fn read_mutable_clone<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Clone + 'static;

    /// Return an immutable reference to the mutable variable.
    fn read_mutable_as_ref<T>(&self, mutable: &Mutable<T>) -> &T
    where
        T: Send + Sync + 'static;

    /// Read the value of a mutable variable using a mapping function.
    fn read_mutable_map<T, U, F: Fn(&T) -> U>(&self, mutable: &Mutable<T>, f: F) -> U
    where
        T: Send + Sync + 'static;
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
        .query::<(&mut MutableCell, &mut MutableNextCell)>()
        .iter_mut(world)
    {
        // Transfer mutable data from next to current.
        sig_val.0 = sig_next.0.take().unwrap();
    }

    // Remove all the MutableNext components.
    let mutables: Vec<Entity> = world
        .query_filtered::<Entity, With<MutableNextCell>>()
        .iter(world)
        .collect();
    mutables.iter().for_each(|mutable| {
        world.entity_mut(*mutable).remove::<MutableNextCell>();
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
        let reader = mutable.signal();
        let reader2 = cx.create_mutable::<i32>(0).signal();

        // Check initial values
        assert_eq!(reader.get_clone(&cx), "Hello".to_string());
        assert_eq!(reader2.get(&cx), 0);

        // Update signals
        mutable.set_clone(&mut cx, "Goodbye".to_string());

        // Values should not have changed yet
        assert_eq!(reader.get_clone(&cx), "Hello".to_string());
        assert_eq!(reader2.get(&cx), 0);

        // Now commit the changes
        commit_mutables(&mut world);

        // Signals should have changed
        let cx = Cx::new(&(), &mut world, &mut scope);
        assert_eq!(reader.get_clone(&cx), "Goodbye".to_string());
        assert_eq!(reader2.get(&cx), 0);
    }
}
