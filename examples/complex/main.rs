//! Example of a simple UI layout
use bevy_mod_picking::{
    backends::bevy_ui::BevyUiBackend,
    input::InputPlugin,
    picking_core::{CorePlugin, InteractionPlugin},
};
use obsidian_ui::{
    colors,
    controls::{
        button, checkbox, splitter, ButtonProps, CheckboxProps, SplitterDirection, SplitterProps,
    },
    typography,
    viewport::{self},
};

use std::f32::consts::PI;

use bevy::{
    asset::io::{file::FileAssetReader, AssetSource},
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        // render_asset::RenderAssetPersistencePolicy,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    ui,
};
use bevy_reactor::*;

fn style_main(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .left(0)
        .top(0)
        .bottom(0)
        .right(0)
        .border(1)
        .border_color(colors::GRAY_300)
        .display(ui::Display::Flex);
}

fn style_aside(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .background_color(colors::BACKGROUND)
        .padding(8)
        .gap(8)
        .flex_direction(ui::FlexDirection::Column)
        .width(200)
        .border(1);
}

fn style_button_row(ss: &mut StyleBuilder) {
    ss.gap(8);
}

fn style_button_flex(ss: &mut StyleBuilder) {
    ss.flex_grow(1.);
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.align_self(ui::AlignSelf::Stretch);
}

fn style_color_edit(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::FlexStart)
        .gap(8);
}

fn style_viewport(ss: &mut StyleBuilder) {
    ss.flex_grow(1.)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .justify_content(ui::JustifyContent::FlexEnd)
        .border_left(1)
        .border_color(Color::BLACK);
}

fn style_log(ss: &mut StyleBuilder) {
    ss.background_color("#0008")
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_self(ui::AlignSelf::Stretch)
        .height(ui::Val::Percent(30.))
        .margin(8);
}

fn style_log_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .justify_content(ui::JustifyContent::FlexEnd)
        .align_self(ui::AlignSelf::Stretch)
        .flex_grow(1.)
        .flex_basis(0)
        .overflow(ui::OverflowAxis::Clip)
        .gap(3)
        .margin(8);
}

fn style_log_entry(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .justify_content(ui::JustifyContent::SpaceBetween)
        .align_self(ui::AlignSelf::Stretch);
}

#[derive(Resource)]
pub struct PanelWidth(f32);

fn main() {
    App::new()
        .register_asset_source(
            "obsidian_ui",
            AssetSource::build()
                .with_reader(|| Box::new(FileAssetReader::new("crates/obsidian_ui/assets"))),
        )
        .init_resource::<Counter>()
        .insert_resource(PanelWidth(200.))
        .init_resource::<viewport::ViewportInset>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((CorePlugin, InputPlugin, InteractionPlugin, BevyUiBackend))
        .add_plugins(ReactorPlugin)
        .add_systems(Startup, setup.pipe(setup_view_root))
        .add_systems(
            Update,
            (
                bevy::window::close_on_esc,
                rotate,
                update_counter,
                viewport::update_viewport_inset,
                viewport::update_camera_viewport,
            ),
        )
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const X_EXTENT: f32 = 14.5;

fn setup_view_root(_camera: In<Entity>, mut commands: Commands) {
    commands.spawn(ViewRoot::new(ui_main.bind(())));
}

fn ui_main(cx: &mut Cx) -> impl View {
    let mut inc_count = 0;
    let mut dec_count = 0;
    let clicked_increment = cx.create_callback_mut(move |_cx| {
        inc_count += 1;
        println!("Increment clicked: {} times", inc_count);
    });
    let clicked_decrement = cx.create_callback_mut(move |_cx| {
        dec_count += 1;
        println!("Decrement clicked: {} times", dec_count);
    });

    let checked_1 = cx.create_mutable(false);
    let checked_2 = cx.create_mutable(false);

    let panel_width = cx
        .create_derived(|cx| {
            let res = cx.use_resource::<PanelWidth>();
            res.0
        })
        .signal();

    Element::<NodeBundle>::new()
        .with_styles((typography::text_default, style_main))
        // .insert(TargetCamera(c2d.0))
        .children((
            Element::<NodeBundle>::new()
                .with_styles(style_aside)
                .create_effect(move |cx, ent| {
                    let width = panel_width.get(cx);
                    let mut style = cx.world_mut().get_mut::<ui::Style>(ent).unwrap();
                    style.width = ui::Val::Px(width);
                })
                .children((
                    Element::<NodeBundle>::new()
                        .with_styles(style_button_row)
                        .children((
                            button.bind(ButtonProps {
                                children: "Open…",
                                on_click: Some(clicked_increment),
                                styles: StyleHandle::new(style_button_flex),
                                ..default()
                            }),
                            button.bind(ButtonProps {
                                children: "Save",
                                on_click: Some(clicked_decrement),
                                styles: StyleHandle::new(style_button_flex),
                                ..default()
                            }),
                        )),
                    Element::<NodeBundle>::new()
                        .with_styles(style_color_edit)
                        .children((
                            checkbox.bind(CheckboxProps {
                                label: "Include Author Name",
                                checked: Some(checked_1.signal()),
                                on_change: Some(cx.create_callback(move |cx: &mut Cx<bool>| {
                                    let checked = *cx.props;
                                    println!("Include Author Name: {}", checked);
                                    checked_1.set(cx, checked);
                                })),
                                ..default()
                            }),
                            checkbox.bind(CheckboxProps {
                                label: "Include Metadata",
                                checked: Some(checked_2.signal()),
                                on_change: Some(cx.create_callback(move |cx: &mut Cx<bool>| {
                                    let checked = *cx.props;
                                    println!("Include Metadata: {}", checked);
                                    checked_2.set(cx, checked);
                                })),
                                ..default()
                            }),
                        )),
                    Element::<NodeBundle>::new().with_styles(style_color_edit),
                )),
            splitter.bind(SplitterProps {
                direction: SplitterDirection::Vertical,
                value: panel_width,
                on_change: cx.create_callback(|cx: &mut Cx<f32>| {
                    let value = *cx.props;
                    let mut panel_width = cx.world_mut().get_resource_mut::<PanelWidth>().unwrap();
                    panel_width.0 = value.max(200.);
                }),
            }),
            Element::<NodeBundle>::new()
                .with_styles(style_viewport)
                .insert(viewport::ViewportInsetElement)
                .children(
                    Element::<NodeBundle>::new()
                        .with_styles(style_log)
                        .children(Element::<NodeBundle>::new().with_styles(style_log_inner)),
                ),
        ))
}

#[derive(Resource, Default)]
pub struct Counter {
    pub count: u32,
    pub foo: usize,
}

fn update_counter(mut counter: ResMut<Counter>, key: Res<Input<KeyCode>>) {
    if key.pressed(KeyCode::Space) {
        counter.count += 1;
    }
}

// Setup 3d shapes
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let shapes = [
        meshes.add(shape::Cube::default().into()),
        meshes.add(shape::Box::default().into()),
        meshes.add(shape::Capsule::default().into()),
        meshes.add(shape::Torus::default().into()),
        meshes.add(shape::Cylinder::default().into()),
        meshes.add(Mesh::try_from(shape::Icosphere::default()).unwrap()),
        meshes.add(shape::UVSphere::default().into()),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: shape,
                material: debug_material.clone(),
                transform: Transform::from_xyz(
                    -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    2.0,
                    0.0,
                )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Shape,
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            // intensity: 1500000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

    let c2d = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    // HUD goes on top of 3D
                    order: 1,
                    // clear_color: ClearColorConfig::None,
                    ..default()
                },
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None,
                },
                ..default()
            },
            UiCameraConfig { show_ui: true },
        ))
        .id();

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        viewport::ViewportCamera,
        UiCameraConfig { show_ui: false },
    ));

    c2d
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        // RenderAssetPersistencePolicy::Unload,
    )
}
