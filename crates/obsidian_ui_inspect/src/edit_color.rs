use bevy::{math::VectorSpace, prelude::*};
use bevy_reactor::*;
use obsidian_ui::{
    controls::{InspectorFieldReadonlyValue, Spacer, Swatch},
    size::Size,
};

use crate::{field_label::FieldLabel, InspectableField};

pub struct FieldEditSrgba {
    pub(crate) field: InspectableField,
}

impl ViewTemplate for FieldEditSrgba {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        let field = self.field.clone();
        let value = cx.create_derived(move |cx| {
            let value = field.get_value(cx);
            if value.is::<Srgba>() {
                return *value.downcast_ref::<Srgba>().unwrap();
            }
            Srgba::ZERO
        });

        Fragment::new((
            FieldLabel {
                field: self.field.clone(),
            },
            InspectorFieldReadonlyValue {
                children: (
                    Swatch {
                        color: value,
                        size: Size::Xxxs,
                        ..default()
                    },
                    Spacer,
                    text_computed(move |cx| {
                        let value = value.get(cx);
                        value.to_hex()
                    }),
                )
                    .fragment(),
                ..default()
            },
        ))
    }
}
