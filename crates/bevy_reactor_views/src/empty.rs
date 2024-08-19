use crate::View;

pub(crate) struct EmptyView;

#[allow(unused)]
impl View for EmptyView {
    fn nodes(&self, out: &mut Vec<bevy::prelude::Entity>) {}

    fn build(&mut self, owner: bevy::prelude::Entity, world: &mut bevy::prelude::World) {}

    fn react(
        &mut self,
        owner: bevy::prelude::Entity,
        world: &mut bevy::prelude::World,
        tracking: &mut bevy_reactor_signals::TrackingScope,
    ) {
    }

    fn raze(&mut self, owner: bevy::prelude::Entity, world: &mut bevy::prelude::World) {}
}
