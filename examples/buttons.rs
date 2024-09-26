//! Example of a simple UI layout

use bevy::{
    asset::io::{file::FileAssetReader, AssetSource},
    prelude::*,
    ui,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{CreateChilden, EntityStyleBuilder, InvokeUiTemplate, TextBuilder};
use bevy_reactor_obsidian::{
    controls::Button, input_dispatch::DefaultKeyHandler, prelude::*, tab_navigation::TabGroup,
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
        .background_color(colors::U1);
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
            builder.text("Variants");
            builder
                .spawn(NodeBundle::default())
                .style(style_row)
                .create_children(|builder| {
                    builder
                        .invoke(Button::new().children(|b| {
                            b.text("Default");
                        }))
                        .invoke(
                            Button::new()
                                .variant(ButtonVariant::Primary)
                                .labeled("Primary"),
                        )
                        .invoke(
                            Button::new()
                                .variant(ButtonVariant::Danger)
                                .labeled("Danger"),
                        )
                        .invoke(
                            Button::new()
                                .variant(ButtonVariant::Selected)
                                .labeled("Selected"),
                        )
                        .invoke(Button::new().minimal(true).labeled("Minimal"));
                });
            builder.text("Variants (disabled)");
            builder
                .spawn(NodeBundle::default())
                .style(style_row)
                .create_children(|builder| {
                    builder
                        .invoke(Button::new().labeled("Default").disabled(true))
                        .invoke(
                            Button::new()
                                .variant(ButtonVariant::Primary)
                                .labeled("Primary")
                                .disabled(true),
                        )
                        .invoke(
                            Button::new()
                                .variant(ButtonVariant::Danger)
                                .labeled("Danger")
                                .disabled(true),
                        )
                        .invoke(
                            Button::new()
                                .variant(ButtonVariant::Selected)
                                .labeled("Selected")
                                .disabled(true),
                        )
                        .invoke(
                            Button::new()
                                .minimal(true)
                                .labeled("Minimal")
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
            builder.text("IconButton");
            builder
                .spawn(NodeBundle::default())
                .style(style_row)
                .create_children(|builder| {
                    builder
                        .invoke(IconButton::new("obsidian_ui://icons/chevron_left.png"))
                        .invoke(
                            IconButton::new("obsidian_ui://icons/chevron_left.png").minimal(true),
                        );
                });
            builder.text("IconButton Size");
            builder
                .spawn(NodeBundle::default())
                .style(style_row)
                .create_children(|builder| {
                    builder
                        .invoke(
                            IconButton::new("obsidian_ui://icons/chevron_left.png").size(Size::Xl),
                        )
                        .invoke(
                            IconButton::new("obsidian_ui://icons/chevron_left.png").size(Size::Lg),
                        )
                        .invoke(
                            IconButton::new("obsidian_ui://icons/chevron_left.png").size(Size::Md),
                        )
                        .invoke(
                            IconButton::new("obsidian_ui://icons/chevron_left.png").size(Size::Sm),
                        )
                        .invoke(
                            IconButton::new("obsidian_ui://icons/chevron_left.png").size(Size::Xs),
                        )
                        .invoke(
                            IconButton::new("obsidian_ui://icons/chevron_left.png").size(Size::Xxs),
                        )
                        .invoke(
                            IconButton::new("obsidian_ui://icons/chevron_left.png")
                                .size(Size::Xxxs),
                        );
                });
        });
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
