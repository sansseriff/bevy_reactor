//! Example of a simple UI layout
mod color_edit;
mod transform_overlay;

use bevy_color::{LinearRgba, Srgba};
use bevy_mod_picking::{
    backends::raycast::{RaycastBackendSettings, RaycastPickable},
    debug::DebugPickingMode,
    picking_core::Pickable,
    prelude::*,
    DefaultPickingPlugins, PickableBundle,
};
use bevy_picking_backdrop::{BackdropBackend, BackdropPickable};
use bevy_reactor_overlays as overlays;
use color_edit::{color_edit, ColorEditState, ColorMode};
use obsidian_ui::{
    colors,
    controls::{
        Button, ButtonProps, ButtonVariant, Checkbox, CheckboxProps, Dialog, DialogFooter,
        DialogFooterProps, DialogHeader, DialogHeaderProps, DialogProps, ScrollView,
        ScrollViewProps, Slider, SliderProps, Splitter, SplitterDirection, SplitterProps, Swatch,
        SwatchProps, TextInput, TextInputProps,
    },
    focus::TabGroup,
    size::Size,
    typography, viewport, ObsidianUiPlugin,
};
use transform_overlay::TransformOverlay;

use std::f32::consts::PI;

use bevy::{
    asset::io::{file::FileAssetReader, AssetSource},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
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
        .border_color(colors::U2)
        .display(ui::Display::Flex)
        .pointer_events(false);
}

fn style_aside(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .background_color(colors::U2)
        .padding(8)
        .gap(8)
        .flex_direction(ui::FlexDirection::Column)
        .width(200)
        .border(1)
        .pointer_events(true);
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
        .border_color(Color::BLACK)
        .pointer_events(false);
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

fn style_scroll_area(ss: &mut StyleBuilder) {
    ss.flex_grow(1.0);
}

// fn style_log_entry(ss: &mut StyleBuilder) {
//     ss.display(ui::Display::Flex)
//         .justify_content(ui::JustifyContent::SpaceBetween)
//         .align_self(ui::AlignSelf::Stretch);
// }

#[derive(Resource)]
pub struct PanelWidth(f32);

#[derive(Resource, Default)]
pub struct SelectedShape(Option<Entity>);

fn main() {
    App::new()
        .register_asset_source(
            "obsidian_ui",
            AssetSource::build()
                .with_reader(|| Box::new(FileAssetReader::new("crates/obsidian_ui/assets"))),
        )
        .init_resource::<SelectedShape>()
        .insert_resource(PanelWidth(200.))
        .insert_resource(ColorEditState {
            mode: ColorMode::Rgb,
            rgb: Srgba::new(1., 0., 0., 1.),
            recent: vec![],
            ..default()
        })
        .init_resource::<viewport::ViewportInset>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(DebugPickingMode::Normal)
        .insert_resource(RaycastBackendSettings {
            require_markers: true,
            ..default()
        })
        // .add_plugins((
        //     CorePlugin,
        //     InputPlugin,
        //     InteractionPlugin,
        //     BevyUiBackend,
        //     RaycastBackend,
        // ))
        .add_plugins((
            ReactorPlugin,
            ObsidianUiPlugin,
            overlays::OverlaysPlugin,
            BackdropBackend,
        ))
        .add_systems(
            Startup,
            (
                setup.pipe(setup_view_overlays),
                setup_ui.pipe(setup_view_root),
            ),
        )
        .add_systems(
            Update,
            (
                bevy::window::close_on_esc,
                rotate,
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

fn setup_view_root(camera: In<Entity>, mut commands: Commands) {
    commands.spawn(ViewRoot::new(ui_main.bind(*camera)));
}

fn ui_main(cx: &mut Cx<Entity>) -> impl View {
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
    let checked_2 = cx.create_mutable(true);
    let red = cx.create_mutable::<f32>(128.);
    let name = cx.create_mutable("filename.txt".to_string());

    let rgb_color = cx.create_derived(move |cx| {
        let red = red.get(cx);
        Srgba::new(red / 255., 100.0, 0.0, 1.0)
    });

    let panel_width = cx.create_derived(|cx| {
        let res = cx.use_resource::<PanelWidth>();
        res.0
    });

    Element::<NodeBundle>::new()
        .with_styles((typography::text_default, style_main))
        .insert((TabGroup::default(), TargetCamera(cx.props)))
        .with_children((
            Dialog::new(DialogProps {
                open: checked_1.signal(),
                on_close: Some(cx.create_callback(move |cx| {
                    checked_1.set(cx, false);
                })),
                children: (
                    DialogHeader::new(DialogHeaderProps {
                        children: "Dialog Header".into(),
                    }),
                    "Dialog Body",
                    DialogFooter::new(DialogFooterProps {
                        children: (
                            Button::new(ButtonProps {
                                children: "Cancel".into(),
                                on_click: Some(cx.create_callback(move |cx| {
                                    checked_1.set(cx, false);
                                })),
                                ..default()
                            }),
                            Button::new(ButtonProps {
                                children: "Close".into(),
                                variant: Signal::Constant(ButtonVariant::Primary),
                                autofocus: true,
                                on_click: Some(cx.create_callback(move |cx| {
                                    checked_1.set(cx, false);
                                })),
                                ..default()
                            }),
                        )
                            .fragment(),
                    }),
                )
                    .fragment(),
                width: ui::Val::Px(400.),
                ..default()
            }),
            Element::<NodeBundle>::new()
                .with_styles(style_aside)
                .create_effect(move |cx, ent| {
                    let width = panel_width.get(cx);
                    let mut style = cx.world_mut().get_mut::<ui::Style>(ent).unwrap();
                    style.width = ui::Val::Px(width);
                })
                .with_children((
                    Element::<NodeBundle>::new()
                        .with_styles(style_button_row)
                        .with_children((
                            Button::new(ButtonProps {
                                children: "Open…".into(),
                                on_click: Some(clicked_increment),
                                style: StyleHandle::new(style_button_flex),
                                ..default()
                            }),
                            Button::new(ButtonProps {
                                children: "Save".into(),
                                on_click: Some(clicked_decrement),
                                style: StyleHandle::new(style_button_flex),
                                ..default()
                            }),
                        )),
                    Element::<NodeBundle>::new()
                        .with_styles(style_color_edit)
                        .with_children((
                            Checkbox::new(CheckboxProps {
                                label: "Include Author Name".into(),
                                checked: checked_1.signal(),
                                on_change: Some(cx.create_callback(move |cx: &mut Cx<bool>| {
                                    let checked = cx.props;
                                    println!("Include Author Name: {}", checked);
                                    checked_1.set(cx, checked);
                                })),
                                ..default()
                            }),
                            Checkbox::new(CheckboxProps {
                                label: "Include Metadata".into(),
                                checked: checked_2.signal(),
                                on_change: Some(cx.create_callback(move |cx: &mut Cx<bool>| {
                                    let checked = cx.props;
                                    println!("Include Metadata: {}", checked);
                                    checked_2.set(cx, checked);
                                })),
                                ..default()
                            }),
                        )),
                    Element::<NodeBundle>::new()
                        .with_styles(style_color_edit)
                        .with_children((
                            Slider::new(SliderProps {
                                min: Signal::Constant(0.),
                                max: Signal::Constant(255.),
                                value: red.signal(),
                                style: StyleHandle::new(style_slider),
                                precision: 1,
                                on_change: Some(cx.create_callback(move |cx| {
                                    red.set(cx, cx.props);
                                })),
                                ..default()
                            }),
                            Swatch::new(SwatchProps {
                                color: rgb_color,
                                size: Size::Md,
                                // style: StyleHandle::new(style_slider),
                                ..default()
                            }),
                            text_computed(move |cx| {
                                let color = rgb_color.get(cx);
                                color.to_hex()
                            }),
                        )),
                    color_edit.bind(()),
                    TextInput::new(TextInputProps {
                        value: name.signal(),
                        on_change: Some(cx.create_callback(move |cx: &mut Cx<String>| {
                            name.set_clone(cx, cx.props.clone());
                        })),
                        ..default()
                    }),
                    Element::<NodeBundle>::new().with_styles(style_color_edit),
                    ScrollView::new(ScrollViewProps {
                        children: "Hello".into(),
                        style: StyleHandle::new(style_scroll_area),
                        scroll_enable_x: true,
                        scroll_enable_y: true,
                        ..default()
                    }),
                )),
            Splitter::new(SplitterProps {
                direction: SplitterDirection::Vertical,
                value: panel_width,
                on_change: cx.create_callback(|cx: &mut Cx<f32>| {
                    let value = cx.props;
                    let mut panel_width = cx.world_mut().get_resource_mut::<PanelWidth>().unwrap();
                    panel_width.0 = value.max(200.);
                }),
            }),
            Element::<NodeBundle>::new()
                .with_styles(style_viewport)
                .insert((viewport::ViewportInsetElement, Pickable::IGNORE))
                .with_children(
                    Element::<NodeBundle>::new()
                        .with_styles(style_log)
                        .with_children(Element::<NodeBundle>::new().with_styles(style_log_inner)),
                ),
        ))
}

fn setup_view_overlays(camera: In<Entity>, mut commands: Commands) {
    commands.spawn(ViewRoot::new(overlay_views.bind(*camera)));
    commands.spawn(ViewRoot::new(transform_overlay.bind(*camera)));
}

fn overlay_views(cx: &mut Cx<Entity>) -> impl View {
    let id = cx.create_entity();
    let hovering = cx.create_hover_signal(id);
    // let color = cx.create_derived(|cx| LinearRgba::from(cx.use_resource::<ColorEditState>().rgb));
    let color: Signal<LinearRgba> = cx.create_derived(move |cx| {
        if hovering.get(cx) {
            colors::ACCENT.into()
        } else {
            colors::U1.into()
        }
    });

    overlays::OverlayShape::for_entity(id, |_cx, sb| {
        sb.with_stroke_width(0.3)
            .stroke_circle(Vec2::new(0., 0.), 5., 64)
            .stroke_polygon(
                &[Vec2::new(-4., -4.), Vec2::new(0., -4.), Vec2::new(-4., 0.)],
                overlays::PolygonOptions {
                    start_marker: overlays::StrokeMarker::Arrowhead,
                    end_marker: overlays::StrokeMarker::Arrowhead,
                    // dash_length: 0.1,
                    // gap_length: 0.1,
                    closed: true,
                    ..default()
                },
            );
    })
    .with_color_signal(color)
    .with_pickable(true)
    // .with_transform(Transform::from_rotation(Quat::from_rotation_y(PI * 0.5)))
    .insert(TargetCamera(cx.props))
}

fn transform_overlay(cx: &mut Cx<Entity>) -> impl View {
    let selected = cx.create_derived(|cx| cx.use_resource::<SelectedShape>().0);

    Portal::new(TransformOverlay { target: selected })
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
        meshes.add(Cuboid::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
    ];

    let num_shapes = shapes.len();

    let shapes_parent = commands
        .spawn((
            SpatialBundle { ..default() },
            BackdropPickable,
            On::<Pointer<Down>>::run(
                |mut event: ListenerMut<Pointer<Down>>,
                 shapes: Query<&Shape>,
                 mut selection: ResMut<SelectedShape>| {
                    if shapes.get(event.target).is_ok() {
                        selection.0 = Some(event.target);
                        // println!("Pointer down on shape {:?}", event.target);
                    } else {
                        selection.0 = None;
                        // println!("Pointer down on backdrop {:?}", event.target);
                    }
                    event.stop_propagation();
                },
            ),
        ))
        .id();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands
            .spawn((
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
                PickableBundle::default(),
                RaycastPickable,
            ))
            .set_parent(shapes_parent);
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            // intensity: 9000.0,
            intensity: 10000000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0)),
        material: materials.add(Color::SILVER),
        ..default()
    });

    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 6., 12.0)
                    .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
                ..default()
            },
            viewport::ViewportCamera,
            RaycastPickable,
            BackdropPickable,
        ))
        .id()
}

fn setup_ui(mut commands: Commands) -> Entity {
    commands
        .spawn((Camera2dBundle {
            camera: Camera {
                // HUD goes on top of 3D
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera_2d: Camera2d {},
            ..default()
        },))
        .id()
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
        RenderAssetUsages::default(),
    )
}
