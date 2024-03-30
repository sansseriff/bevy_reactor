use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_reactor::*;
use bevy_reactor_overlays::OverlayShape;

#[derive(Default)]
pub struct TransformOverlay {
    pub target: Signal<Option<Entity>>,
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
        Cond::new(
            move |cx| target_entity.get(cx).is_some(),
            move || {
                OverlayShape::new(|_cx, sb| {
                    sb.with_stroke_width(0.2)
                        .stroke_rect(Rect::from_center_size(Vec2::new(0., 0.), Vec2::new(2., 2.)));
                })
                .with_transform_signal(target_position)
            },
            || (),
        )
    }
}
