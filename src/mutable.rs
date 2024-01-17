use crate::accessor::{CloneGetter, CloneSetter, Getter, Setter, SignalKind};
use bevy::prelude::*;
use std::{any::Any, sync::atomic::AtomicBool};

/// Contains a mutable reactive value.
#[derive(Component)]
pub(crate) struct MutableValue {
    // TODO: Use this to allow multiple changes per frame.
    pub(crate) changed: AtomicBool,
    pub(crate) value: Box<dyn Any + Send + Sync + 'static>,
}

/// Contains the value which will be written to the signal on the next update.
/// This is used to avoid writing to the signal multiple times in a single frame, and also
/// ensures that the signal values remain stable during a reaction.
#[derive(Component)]
pub(crate) struct MutableValueNext(pub(crate) Box<dyn Any + Send + Sync + 'static>);

/// Contains a reference to a reactive mutable variable.
pub struct Mutable<T> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<T>,
}

impl<T> Mutable<T>
where
    T: PartialEq + Copy + Send + Sync + 'static,
{
    pub fn signal(&self) -> (Getter<T>, Setter<T>) {
        (
            Getter {
                id: self.id,
                kind: SignalKind::Mutable,
                marker: std::marker::PhantomData,
            },
            Setter {
                id: self.id,
                marker: std::marker::PhantomData,
            },
        )
    }
}

impl<T> Mutable<T>
where
    T: PartialEq + Clone + Send + Sync + 'static,
{
    pub fn signal_clone(&self) -> (CloneGetter<T>, CloneSetter<T>) {
        (
            CloneGetter {
                id: self.id,
                kind: SignalKind::Mutable,
                marker: std::marker::PhantomData,
            },
            CloneSetter {
                id: self.id,
                marker: std::marker::PhantomData,
            },
        )
    }
}

/// Trait that allows writing the value to a signal, using Clone semantics.
// pub struct WriteSignalClone<T: Clone> {
//     state: Entity,
//     marker: std::marker::PhantomData<T>,
// }

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
    use crate::{cx::Cx, scope::TrackingScope};

    use super::*;

    #[test]
    fn test_mutable_copy() {
        let mut world = World::default();
        let mut scope = TrackingScope::new(world.change_tick());
        let mut cx = Cx::new(&(), &mut world, &mut scope);

        let (reader, mut writer) = cx.create_mutable::<i32>(0).signal();
        let (reader2, mut _writer2) = cx.create_mutable::<i32>(0).signal();

        // Check initial values
        assert_eq!(reader.get(&cx), 0);
        assert_eq!(reader2.get(&cx), 0);

        // Update signals
        writer.set(&mut cx, 1);

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

        let (reader, mut writer) = cx.create_mutable("Hello".to_string()).signal_clone();
        let (reader2, mut _writer2) = cx.create_mutable::<i32>(0).signal_clone();

        // Check initial values
        assert_eq!(reader.get(&cx), "Hello".to_string());
        assert_eq!(reader2.get(&cx), 0);

        // Update signals
        writer.set(&mut cx, "Goodbye".to_string());

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
