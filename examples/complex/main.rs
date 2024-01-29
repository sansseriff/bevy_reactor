//! Example of a simple UI layout
use bevy_mod_picking::{
    backends::bevy_ui::BevyUiBackend,
    input::InputPlugin,
    picking_core::{CorePlugin, InteractionPlugin},
};
use obsidian_ui::{
    colors,
    controls::{button, ButtonProps},
    viewport::{self},
};

use std::f32::consts::PI;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        // render_asset::RenderAssetPersistencePolicy,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    ui,
};
use bevy_reactor::*;

fn style_main(ss: &mut StyleBuilderContext) {
    ss.position(ui::PositionType::Absolute)
        .left(10.)
        .top(10.)
        .bottom(10)
        .right(10.)
        .border(1)
        .border_color(colors::SURFACE_250)
        .display(ui::Display::Flex);
}

fn style_aside(ss: &mut StyleBuilderContext) {
    ss.display(ui::Display::Flex)
        .background_color(colors::BACKGROUND)
        .padding(8)
        .gap(8)
        .flex_direction(ui::FlexDirection::Column)
        .width(200)
        .border(1);
}

fn style_button_row(ss: &mut StyleBuilderContext) {
    ss.gap(8);
}

fn style_button_flex(ss: &mut StyleBuilderContext) {
    ss.flex_grow(1.);
}

fn style_slider(ss: &mut StyleBuilderContext) {
    ss.align_self(ui::AlignSelf::Stretch);
}

fn style_color_edit(ss: &mut StyleBuilderContext) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .gap(8);
}

fn style_viewport(ss: &mut StyleBuilderContext) {
    ss.flex_grow(1.)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .justify_content(ui::JustifyContent::FlexEnd);
}

fn style_log(ss: &mut StyleBuilderContext) {
    ss.background_color("#0008")
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_self(ui::AlignSelf::Stretch)
        .height(ui::Val::Percent(30.))
        .margin(8);
}

fn style_log_inner(ss: &mut StyleBuilderContext) {
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

fn style_log_entry(ss: &mut StyleBuilderContext) {
    ss.display(ui::Display::Flex)
        .justify_content(ui::JustifyContent::SpaceBetween)
        .align_self(ui::AlignSelf::Stretch);
}

fn main() {
    App::new()
        .init_resource::<Counter>()
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
    commands.spawn(ViewRoot::new(
        Element::<NodeBundle>::new()
            .with_styles(style_main)
            // .insert(TargetCamera(c2d.0))
            .children((
                Element::<NodeBundle>::new()
                    .with_styles(style_aside)
                    .children((
                        Element::<NodeBundle>::new()
                            .with_styles(style_button_row)
                            .children((
                                button.bind(ButtonProps {
                                    children: "Increment",
                                    ..default()
                                }),
                                button.bind(ButtonProps {
                                    children: "Decrement",
                                    ..default()
                                }),
                            )),
                        Element::<NodeBundle>::new().with_styles(style_color_edit),
                    )),
                Element::<NodeBundle>::new()
                    .with_styles(style_viewport)
                    .insert(viewport::ViewportInsetElement)
                    .children(
                        Element::<NodeBundle>::new()
                            .with_styles(style_log)
                            .children(Element::<NodeBundle>::new().with_styles(style_log_inner)),
                    ),
            )),
    ));
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
