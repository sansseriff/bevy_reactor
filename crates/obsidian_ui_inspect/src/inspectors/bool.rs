use bevy::prelude::*;
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextSetup};
use obsidian_ui::controls::Checkbox;

use crate::{templates::field_label::FieldLabel, Inspectable};

pub struct BooleanFieldInspector(pub(crate) Inspectable);

impl ViewTemplate for BooleanFieldInspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let is_checked = cx.create_memo(move |cx| {
            if let Some(value) = field.reflect(cx) {
                if value.is::<bool>() {
                    return *value.downcast_ref::<bool>().unwrap();
                }
            }
            false
        });

        let field = self.0.clone();
        Fragment::new((
            FieldLabel {
                field: field.clone(),
            },
            Checkbox {
                checked: is_checked,
                on_change: Some(cx.create_callback(move |cx: &mut Cx, value: bool| {
                    field.set_value(cx, value.as_reflect());
                })),
                style: StyleHandle::new(|ss: &mut StyleBuilder| {
                    ss.justify_self(JustifySelf::Start);
                }),
                ..default()
            },
        ))
    }
}
