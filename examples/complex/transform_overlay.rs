use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_reactor::*;
use bevy_reactor_overlays::OverlayShape;

#[derive(Default)]
pub struct TransformOverlay {
    pub target: Signal<Option<Entity>>,
}

impl ViewFactory for TransformOverlay {
    fn create(&self, _cx: &mut Cx) -> impl View + Send + Sync + 'static {
        let target_entity = self.target;
        Cond::new(
            move |cx| target_entity.get(cx).is_some(),
            move || {
                OverlayShape::new(|cx, sb| {
                    sb.with_stroke_width(0.2)
                        .stroke_rect(Rect::from_center_size(Vec2::new(0., 0.), Vec2::new(2., 2.)));
                })
                .with_transform(Transform::from_rotation(Quat::from_rotation_x(-PI * 0.5)))
            },
            || (),
        )
    }
}
