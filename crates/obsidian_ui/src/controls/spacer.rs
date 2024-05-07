use bevy::prelude::*;
use bevy_reactor::*;

fn style_spacer(ss: &mut StyleBuilder) {
    ss.flex_grow(1.);
}

/// A spacer widget that fills the available space.
#[derive(Clone, Default)]
pub struct Spacer;

impl ViewTemplate for Spacer {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        Element::<NodeBundle>::new().style(style_spacer)
    }
}
