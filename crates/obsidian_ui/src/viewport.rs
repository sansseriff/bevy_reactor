use bevy::{prelude::*, render::camera::Viewport};

/// Used to create margins around the viewport so that side panels don't overwrite the 3d scene.
#[derive(Default, Resource, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct ViewportInset {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

/// Marker which identifies which camera is displayed in the viewport.
#[derive(Component)]
pub struct ViewportCamera;

/// A marker component for that identifies which element contains the 3d view. The
/// `update_viewport_inset` system measures the on-screen position of the UiNode that this
/// component is attached to, and updates the screen position of the 3D view to match it.
#[derive(Component, Clone)]
pub struct ViewportInsetElement;

/// Update the viewport inset based on the global position of the ui element representing the
/// viewport.
pub fn update_viewport_inset(
    windows: Query<&Window>,
    query: Query<(&Node, &GlobalTransform), With<ViewportInsetElement>>,
    mut viewport_inset: ResMut<ViewportInset>,
) {
    let mut inset = ViewportInset::default();
    match query.get_single() {
        Ok((node, transform)) => {
            let position = transform.translation();
            let ui_position = position.truncate();
            let extents = node.size() / 2.0;
            let min = ui_position - extents;
            let max = ui_position + extents;

            let window = windows.single();
            let ww = window.resolution.physical_width() as f32;
            let wh = window.resolution.physical_height() as f32;
            let sf = window.resolution.scale_factor() as f32;

            inset.left = min.x;
            inset.top = min.y;
            inset.right = ww / sf - max.x;
            inset.bottom = wh / sf - max.y;
        }
        Err(_) => {
            if query.iter().count() > 1 {
                error!("Multiple ViewportInset elements!");
            }
        }
    }

    if inset != *viewport_inset {
        *viewport_inset.as_mut() = inset;
    }
}

/// Update the camera viewport and fov properties based on the window size and the viewport
/// margins.
pub fn update_camera_viewport(
    viewport_inset: Res<ViewportInset>,
    windows: Query<&Window>,
    mut camera_query: Query<(&mut Camera, &mut Projection), With<ViewportCamera>>,
) {
    let window = windows.single();
    let ww = window.resolution.physical_width() as f32;
    let wh = window.resolution.physical_height() as f32;
    let sf = window.resolution.scale_factor() as f32;
    let left = viewport_inset.left * sf;
    let right = viewport_inset.right * sf;
    let top = viewport_inset.top * sf;
    let bottom = viewport_inset.bottom * sf;
    let vw = (ww - left - right).max(1.);
    let vh = (wh - top - bottom).max(1.);

    let (mut camera, mut _projection) = camera_query.single_mut();
    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(left as u32, top as u32),
        physical_size: UVec2::new(vw as u32, vh as u32),
        ..default()
    });
}
