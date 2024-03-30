use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_color::{LinearRgba, Luminance};
use bevy_mod_picking::{backends::raycast::RaycastPickable, prelude::*};
use bevy_reactor::*;
use bevy_reactor_overlays::{OverlayShape, PolygonOptions, StrokeMarker};
use obsidian_ui::colors;

#[derive(Default)]
pub struct TransformOverlay {
    /// Target entity to drag
    pub target: Signal<Option<Entity>>,
    /// Callback called when dragged. The argument is the delta of the drag.
    pub on_change: Option<Callback<Vec2>>,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    drag_origin: Vec3,
}

impl ViewFactory for TransformOverlay {
    fn create(&self, cx: &mut Cx) -> impl View + Send + Sync + 'static {
        let target_entity = self.target;
        let target_position = cx.create_derived(move |rcx| {
            if let Some(target) = target_entity.get(rcx) {
                if let Some(transform) = rcx.use_component::<GlobalTransform>(target) {
                    let mut trans = Transform::from_translation(transform.translation());
                    trans.rotate_local_x(-PI * 0.5);
                    return trans;
                }
            }
            Transform::default()
        });

        // Need to spawn the entity ids early so that we can test for hovering.
        let x_arrow = cx.create_entity();
        let y_arrow = cx.create_entity();

        let x_arrow_hovering = cx.create_hover_signal(x_arrow);
        let y_arrow_hovering = cx.create_hover_signal(y_arrow);

        let x_arrow_color: Signal<LinearRgba> = cx.create_derived(move |cx| {
            if x_arrow_hovering.get(cx) {
                colors::X_RED.lighter(0.1).into()
            } else {
                colors::X_RED.into()
            }
        });

        let y_arrow_color: Signal<LinearRgba> = cx.create_derived(move |cx| {
            if y_arrow_hovering.get(cx) {
                colors::Y_GREEN.lighter(0.1).into()
            } else {
                colors::Y_GREEN.into()
            }
        });

        let on_change = self.on_change;

        let drag_state = cx.create_mutable::<DragState>(DragState::default());

        Cond::new(
            move |cx| target_entity.get(cx).is_some(),
            move || {
                OverlayShape::new(|_cx, sb| {
                    sb.with_stroke_width(0.2)
                        .stroke_rect(Rect::from_center_size(Vec2::new(0., 0.), Vec2::new(2., 2.)));
                })
                .with_transform_signal(target_position)
                .with_pickable(true)
                .with_children((
                    OverlayShape::for_entity(x_arrow, |_cx, sb| {
                        sb.with_stroke_width(0.3)
                            .stroke_polygon(
                                &[Vec2::new(1.2, 0.), Vec2::new(2., 0.)],
                                PolygonOptions {
                                    end_marker: StrokeMarker::Arrowhead,
                                    ..default()
                                },
                            )
                            .stroke_polygon(
                                &[Vec2::new(-1.2, 0.), Vec2::new(-2., 0.)],
                                PolygonOptions {
                                    end_marker: StrokeMarker::Arrowhead,
                                    ..default()
                                },
                            );
                    })
                    .with_color_signal(x_arrow_color)
                    .with_pickable(true)
                    .insert((
                        On::<Pointer<DragStart>>::run(move |world: &mut World| {
                            let event = world
                                .get_resource::<ListenerInput<Pointer<DragStart>>>()
                                .unwrap();
                            // println!("drag start: {:?}", event);
                            // get_intersect_point(world, &event.location);
                            // if let Some(on_change) = on_change {
                            //     world.run_callback(on_change, Vec2::new(event.delta.x, 0.));
                            // }
                        }),
                        On::<Pointer<Drag>>::run(move |world: &mut World| {
                            let event = world
                                .get_resource::<ListenerInput<Pointer<Drag>>>()
                                .unwrap();
                            if let Some(on_change) = on_change {
                                world.run_callback(on_change, Vec2::new(event.delta.x * 0.1, 0.));
                            }
                        }),
                    )),
                    OverlayShape::for_entity(y_arrow, |_cx, sb| {
                        sb.with_stroke_width(0.3)
                            .stroke_polygon(
                                &[Vec2::new(0., 1.2), Vec2::new(0., 2.)],
                                PolygonOptions {
                                    end_marker: StrokeMarker::Arrowhead,
                                    ..default()
                                },
                            )
                            .stroke_polygon(
                                &[Vec2::new(0., -1.2), Vec2::new(0., -2.)],
                                PolygonOptions {
                                    end_marker: StrokeMarker::Arrowhead,
                                    ..default()
                                },
                            );
                    })
                    .with_color_signal(y_arrow_color)
                    .with_pickable(true)
                    .insert(On::<Pointer<Drag>>::run(
                        move |world: &mut World| {
                            let event = world
                                .get_resource::<ListenerInput<Pointer<Drag>>>()
                                .unwrap();
                            if let Some(on_change) = on_change {
                                world.run_callback(on_change, Vec2::new(0., event.delta.y * 0.1));
                            }
                        },
                    )),
                ))
            },
            || (),
        )
    }
}

fn get_intersect_point(world: &mut World, pointer_loc: &PointerLocation) {
    let (camera, camera_transform) = world
        .query_filtered::<(&Camera, &GlobalTransform), With<RaycastPickable>>()
        .single(world);
    let ray = make_ray(camera, camera_transform, pointer_loc);
    println!("ray: {:?}", ray);
    // if let Some(ray) = ray {
    //     let intersect = ray.intersect_plane(Vec3::new(0., 0., 0.), Vec3::new(0., 0., 1.));
    //     println!("intersect: {:?}", intersect);
    // }
}

fn make_ray(
    // primary_window_entity: &Query<Entity, With<PrimaryWindow>>,
    camera: &Camera,
    camera_tfm: &GlobalTransform,
    pointer_loc: &PointerLocation,
) -> Option<Ray3d> {
    let pointer_loc = pointer_loc.location()?;
    // if !pointer_loc.is_in_viewport(camera, primary_window_entity) {
    //     return None;
    // }
    let mut viewport_pos = pointer_loc.position;
    if let Some(viewport) = &camera.viewport {
        let viewport_logical = camera.to_logical(viewport.physical_position)?;
        viewport_pos -= viewport_logical;
    }
    camera.viewport_to_world(camera_tfm, viewport_pos)
}
