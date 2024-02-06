use bevy::{
    ecs::{entity::Entity, world::World},
    hierarchy::Parent,
};
use bevy_mod_picking::{focus::HoverMap, pointer::PointerId};

use crate::{
    signal::Signal, Cx, Reaction, ReactionHandle, RunContextSetup, TrackingScope, WriteMutable,
};

pub(crate) struct HoverReaction {
    target: Entity,
}

impl Reaction for HoverReaction {
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        tracking.track_resource::<HoverMap>(world);
        let hover_map = world.get_resource::<HoverMap>().unwrap();

        // "Hovering" is defined as "the mouse is over this element or one of its descendants."
        let is_hovering = match hover_map.get(&PointerId::Mouse) {
            Some(map) => map
                .iter()
                .any(|(ha, _)| is_descendant(world, ha, &self.target)),
            None => false,
        };

        // TODO: Direct access to mutable by entity id is kind of a cheat.
        world.write_mutable::<bool>(owner, is_hovering);
    }
}

/// True if the given entity is a descendant of the given ancestor.
fn is_descendant(world: &World, e: &Entity, ancestor: &Entity) -> bool {
    let mut ha = e;
    loop {
        if ha == ancestor {
            return true;
        }
        match world.get_entity(*ha).map(|e| e.get::<Parent>()) {
            Some(Some(parent)) => ha = parent,
            _ => return false,
        }
    }
}

/// Method to create a signal that tracks whether the mouse is hovering over the given entity.
pub trait CreateHoverSignal {
    /// Signal that returns true when the mouse is hovering over the given entity or a descendant.
    fn create_hover_signal(&mut self, target: Entity) -> Signal<bool>;
}

impl<'p, 'w, Props> CreateHoverSignal for Cx<'p, 'w, Props> {
    fn create_hover_signal(&mut self, target: Entity) -> Signal<bool> {
        let mutable = self.create_mutable::<bool>(false);
        let mut reaction = HoverReaction { target };
        let mut tracking = TrackingScope::new(self.world.read_change_tick());
        reaction.react(mutable.id, self.world, &mut tracking);
        self.world
            .entity_mut(mutable.id)
            .insert((ReactionHandle::new(reaction), tracking));
        mutable.signal()
    }
}
