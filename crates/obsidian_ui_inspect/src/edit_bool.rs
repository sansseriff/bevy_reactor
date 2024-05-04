use bevy::prelude::*;
use bevy_reactor::*;
use obsidian_ui::controls::Checkbox;

use crate::{field_label::FieldLabel, InspectableField};

pub struct FieldEditBool(pub(crate) InspectableField);

impl ViewTemplate for FieldEditBool {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        let field = self.0.clone();
        let is_checked = cx.create_derived(move |cx| {
            let value = field.get_value(cx);
            if value.is::<bool>() {
                return *value.downcast_ref::<bool>().unwrap();
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
