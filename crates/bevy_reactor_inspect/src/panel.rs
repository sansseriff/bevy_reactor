use bevy::{
    core::Name,
    prelude::{NodeBundle, World},
    ui,
};
use bevy_mod_stylebuilder::{
    StyleBuilder, StyleBuilderBackground, StyleBuilderBorderColor, StyleBuilderLayout,
};
use bevy_reactor_builder::EntityStyleBuilder;
use bevy_reactor_obsidian::{colors, typography};

fn style_panel(sb: &mut StyleBuilder) {
    sb.position(ui::PositionType::Absolute)
        .left(10)
        .right(10)
        .width(200)
        .height(400)
        .background_color(colors::BACKGROUND)
        .border(2)
        .border_color(colors::U1);
}

pub(crate) fn create_inspector_panel(world: &mut World) {
    world
        .spawn((NodeBundle::default(), Name::new("InspectorPanel")))
        .styles((typography::text_default, style_panel));
}
