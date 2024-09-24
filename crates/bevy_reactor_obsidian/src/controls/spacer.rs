use bevy::prelude::*;
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{EntityStyleBuilder, UiBuilder, UiTemplate};
use bevy_reactor_signals::Cx;
use bevy_reactor_views::{Element, IntoView, ViewTemplate};

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

impl UiTemplate for Spacer {
    fn build(&self, builder: &mut UiBuilder) {
        builder.spawn(NodeBundle::default()).style(style_spacer);
    }
}
