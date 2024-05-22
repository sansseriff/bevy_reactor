use bevy::prelude::*;
use bevy_reactor_signals::{Cx, IntoSignal, RunContextRead, RunContextSetup, Signal};

/// Trait which adds `use_element_rect` to [`Cx`].
pub trait UseElementRect {
    /// Returns the logical rect of the element with the given `id`.
    fn use_element_rect(&mut self, id: Entity) -> Signal<Rect>;
}

impl<'p, 'w> UseElementRect for Cx<'p, 'w> {
    fn use_element_rect(&mut self, id: Entity) -> Signal<Rect> {
        self.create_derived(move |cx| {
            match (
                cx.use_component::<Node>(id),
                cx.use_component_untracked::<GlobalTransform>(id),
            ) {
                (Some(node), Some(transform)) => node.logical_rect(transform),
                _ => Rect::new(0., 0., 0., 0.),
            }
        })
        .into_signal()
    }
}
