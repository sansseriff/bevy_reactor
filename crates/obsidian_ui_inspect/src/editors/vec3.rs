use bevy::{
    math::Vec3,
    reflect::Reflect,
    ui::{self, node_bundles::NodeBundle},
};
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextSetup};
use obsidian_ui::controls::SpinBox;

use crate::{templates::field_label::FieldLabel, InspectableField, Precision, Step};

pub struct FieldEditVec3(pub(crate) InspectableField);

#[derive(Clone, Debug)]
struct Vec3Attrs {
    precision: usize,
    step: f32,
}

fn style_spinbox_group(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexStart)
        .column_gap(3);
}

fn style_spinbox(ss: &mut StyleBuilder) {
    ss.flex_grow(1.);
}

impl ViewTemplate for FieldEditVec3 {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let value = cx.create_memo(move |cx| match field.reflect(cx) {
            Some(value) if value.is::<Vec3>() => *value.downcast_ref::<Vec3>().unwrap(),
            _ => Vec3::splat(0.),
        });

        let field = self.0.clone();
        let mut slider_params = Vec3Attrs {
            precision: 2,
            step: 0.1,
        };

        if let Some(attrs) = field.attributes {
            if let Some(precision) = attrs.get::<Precision>() {
                slider_params.precision = precision.0;
            }
            if let Some(step) = attrs.get::<Step<f32>>() {
                slider_params.step = step.0;
            } else {
                slider_params.step = 10.0f32.powi(-(slider_params.precision as i32));
            }
        }

        Fragment::new((
            FieldLabel {
                field: field.clone(),
            },
            // Don't need `Cond` here because condition is not reactive; reflection data
            // is constant.
            Element::<NodeBundle>::new()
                .style(style_spinbox_group)
                .children((
                    // "x",
                    SpinBox::new()
                        .style(style_spinbox)
                        .precision(slider_params.precision)
                        .step(slider_params.step)
                        .value(cx.create_derived(move |cx| value.get(cx).x))
                        .on_change(cx.create_callback({
                            let field = self.0.clone();
                            move |cx, x: f32| {
                                let value = value.get(cx).with_x(x);
                                field.update(cx, &|reflect| {
                                    reflect.apply(value.as_reflect());
                                });
                            }
                        })),
                    // "y",
                    SpinBox::new()
                        .style(style_spinbox)
                        .precision(slider_params.precision)
                        .step(slider_params.step)
                        .value(cx.create_derived(move |cx| value.get(cx).y))
                        .on_change(cx.create_callback({
                            let field = self.0.clone();
                            move |cx, y: f32| {
                                let value = value.get(cx).with_y(y);
                                field.update(cx, &|reflect| {
                                    reflect.apply(value.as_reflect());
                                });
                            }
                        })),
                    // "z",
                    SpinBox::new()
                        .style(style_spinbox)
                        .precision(slider_params.precision)
                        .step(slider_params.step)
                        .value(cx.create_derived(move |cx| value.get(cx).z))
                        .on_change(cx.create_callback({
                            let field = self.0.clone();
                            move |cx, z: f32| {
                                let value = value.get(cx).with_z(z);
                                field.update(cx, &|reflect| {
                                    reflect.apply(value.as_reflect());
                                });
                            }
                        })),
                )),
        ))
    }
}
