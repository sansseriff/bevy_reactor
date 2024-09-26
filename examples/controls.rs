//! Example of a simple UI layout

use bevy::{
    asset::io::{file::FileAssetReader, AssetSource},
    color::palettes,
    ecs::world::DeferredWorld,
    prelude::*,
    ui,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{CreateChilden, EntityStyleBuilder, InvokeUiTemplate, TextBuilder};
use bevy_reactor_obsidian::{controls::Button, input_dispatch::DefaultKeyHandler, prelude::*};
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
        .register_asset_source(
            "obsidian_ui",
            AssetSource::build()
                .with_reader(|| Box::new(FileAssetReader::new("crates/obsidian_ui/assets"))),
        )
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
            builder.text("Swatch");
            builder
                .spawn(NodeBundle::default())
                .style(style_row)
                .create_children(|builder| {
                    builder
                        .invoke(Swatch::new(palettes::css::GOLDENROD))
                        .invoke(Swatch::new(palettes::css::LIME))
                        .invoke(Swatch::new(palettes::css::RED))
                        .invoke(Swatch::new(Srgba::NONE))
                        .invoke(Swatch::new(palettes::css::BLUE).selected(true));
                });
            builder.text("Checkbox");
            builder
                .spawn(NodeBundle::default())
                .style(style_row)
                .create_children(|builder| {
                    let checked_1 = builder.create_mutable(true);
                    let checked_2 = builder.create_mutable(false);
                    let on_change_1 = builder.create_callback(
                        move |value: In<bool>, mut world: DeferredWorld| {
                            checked_1.set(&mut world, *value);
                        },
                    );
                    let on_change_2 = builder.create_callback(
                        move |value: In<bool>, mut world: DeferredWorld| {
                            checked_2.set(&mut world, *value);
                        },
                    );
                    builder
                        .invoke(
                            Checkbox::new()
                                .labeled("Checked")
                                .checked(checked_1)
                                .on_change(on_change_1),
                        )
                        .invoke(
                            Checkbox::new()
                                .labeled("Checked (disabled)")
                                .checked(checked_1)
                                .on_change(on_change_1)
                                .disabled(true),
                        )
                        .invoke(
                            Checkbox::new()
                                .labeled("Unchecked")
                                .checked(checked_2)
                                .on_change(on_change_2),
                        )
                        .invoke(
                            Checkbox::new()
                                .labeled("Unchecked (disabled)")
                                .checked(checked_2)
                                .on_change(on_change_2)
                                .disabled(true),
                        );
                });
            builder.text("Size");
            builder
                .spawn(NodeBundle::default())
                .style(style_row)
                .create_children(|builder| {
                    builder
                        .invoke(Button::new().labeled("Size: Xl").size(Size::Xl))
                        .invoke(Button::new().labeled("Size: Lg").size(Size::Lg))
                        .invoke(Button::new().labeled("Size: Md").size(Size::Md))
                        .invoke(Button::new().labeled("Size: Sm").size(Size::Sm))
                        .invoke(Button::new().labeled("Size: Xs").size(Size::Xs))
                        .invoke(Button::new().labeled("Size: Xxs").size(Size::Xxs))
                        .invoke(Button::new().labeled("Size: Xxxs").size(Size::Xxxs));
                });
            builder.text("Corners");
            builder
                .spawn(NodeBundle::default())
                .style(style_row)
                .create_children(|builder| {
                    builder
                        .invoke(
                            Button::new()
                                .labeled("corners: All")
                                .corners(RoundedCorners::All),
                        )
                        .invoke(
                            Button::new()
                                .labeled("corners: Top")
                                .corners(RoundedCorners::Top),
                        )
                        .invoke(
                            Button::new()
                                .labeled("corners: Bottom")
                                .corners(RoundedCorners::Bottom),
                        )
                        .invoke(
                            Button::new()
                                .labeled("corners: Left")
                                .corners(RoundedCorners::Left),
                        )
                        .invoke(
                            Button::new()
                                .labeled("corners: Right")
                                .corners(RoundedCorners::Right),
                        )
                        .invoke(
                            Button::new()
                                .labeled("corners: None")
                                .corners(RoundedCorners::None),
                        );
                });
        });
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
