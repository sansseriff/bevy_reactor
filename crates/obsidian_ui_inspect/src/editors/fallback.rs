use bevy_reactor::*;

use crate::{
    templates::{field_label::FieldLabel, field_readonly_value::FieldReadonlyValue},
    InspectableField,
};

/// Field editor for when no specific editor is available.
pub struct FieldEditFallback(pub(crate) InspectableField);

impl ViewTemplate for FieldEditFallback {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let reflect = field.reflect(cx);
        // let is_checked = cx.create_derived(move |cx| {
        //     let value = field.get_value(cx);
        //     if value.is::<bool>() {
        //         return *value.downcast_ref::<bool>().unwrap();
        //     }
        //     false
        // });

        // let field = self.field.clone();
        Fragment::new((
            FieldLabel {
                field: self.0.clone(),
            },
            FieldReadonlyValue::new().children(format!("TODO: {}", reflect.reflect_type_path())),
        ))
    }
}
