//! Example of a simple UI layout

use bevy::{color::palettes, ecs::world::DeferredWorld, prelude::*, ui};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{CreateChilden, EntityStyleBuilder, InvokeUiTemplate, TextBuilder};
use bevy_reactor_obsidian::{
    animation::{BistableTransitionState, CreateBistableTransition},
    input_dispatch::DefaultKeyHandler,
    prelude::*,
};
use bevy_reactor_signals::SignalsPlugin;

fn style_test(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .position(ui::PositionType::Absolute)
        .padding(3)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0)
        .row_gap(4)
        .background_color(colors::BACKGROUND);
}

fn style_row(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .column_gap(4);
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SignalsPlugin,
            StyleBuilderPlugin,
            ObsidianUiPlugin,
        ))
        .add_systems(Startup, setup_view_root)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup_view_root(world: &mut World) {
    let camera = world
        .spawn((Camera2dBundle {
            camera: Camera::default(),
            camera_2d: Camera2d {},
            ..default()
        },))
        .id();

    world
        .spawn(NodeBundle::default())
        .insert((TargetCamera(camera), TabGroup::default(), DefaultKeyHandler))
        .observe(handle_tab_navigation)
        .style(style_test)
        .create_children(|builder| {
            builder.text("bistable_transition");
            let row = builder.spawn(NodeBundle::default());
            let row_id = row.id();
            let is_hover = builder.create_hover_signal(row_id);
            let transition_state = builder.create_bistable_transition(is_hover, 0.3);
            let color = builder.create_derived(move |rcx| match transition_state.get(rcx) {
                BistableTransitionState::EnterStart | BistableTransitionState::Entering => {
                    palettes::css::GREEN
                }
                BistableTransitionState::Entered => palettes::css::YELLOW,
                BistableTransitionState::ExitStart | BistableTransitionState::Exiting => {
                    palettes::css::ORANGE
                }
                BistableTransitionState::Exited => palettes::css::GRAY,
            });
            builder
                .entity_mut(row_id)
                .style(style_row)
                .create_children(|builder| {
                    builder.invoke(Swatch::new(color).style(|sb: &mut StyleBuilder| {
                        sb.width(64);
                    }));
                });

            builder.text("DisclosureToggle");
            builder
                .spawn(NodeBundle::default())
                .create_children(|builder| {
                    let expanded = builder.create_mutable(false);
                    let on_change = builder.create_callback(
                        move |value: In<bool>, mut world: DeferredWorld| {
                            expanded.set(&mut world, *value);
                        },
                    );
                    builder.invoke(
                        DisclosureToggle::new()
                            .expanded(expanded)
                            .on_change(on_change),
                    );
                });
        });
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
