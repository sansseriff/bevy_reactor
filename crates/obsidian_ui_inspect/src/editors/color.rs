use bevy::prelude::*;
use bevy_reactor::*;
use obsidian_ui::{
    controls::{Spacer, Swatch},
    size::Size,
};

use crate::{
    templates::{field_label::FieldLabel, field_readonly_value::FieldReadonlyValue},
    InspectableField,
};

pub struct FieldEditSrgba {
    pub(crate) field: InspectableField,
}

impl ViewTemplate for FieldEditSrgba {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.field.clone();
        let value = cx.create_derived(move |cx| {
            let value = field.get_value(cx);
            if value.is::<Srgba>() {
                return *value.downcast_ref::<Srgba>().unwrap();
            }
            Srgba::NONE
        });

        Fragment::new((
            FieldLabel {
                field: self.field.clone(),
            },
            FieldReadonlyValue::new().children((
                Swatch::new(value).size(Size::Xxxs),
                Spacer,
                text_computed(move |cx| {
                    let value = value.get(cx);
                    value.to_hex()
                }),
            )),
        ))
    }
}
