use std::sync::Arc;

use bevy_reactor::*;
use bevy_reactor_signals::Cx;

use crate::{
    templates::{field_label::FieldLabel, field_readonly_value::FieldReadonlyValue},
    Inspectable,
};

/// Field editor for when no specific editor is available.
pub struct FallbackInspector(pub(crate) Arc<Inspectable>);

impl ViewTemplate for FallbackInspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let Some(reflect) = field.reflect(cx) else {
            return ().into_view();
        };
        Fragment::new((
            FieldLabel {
                field: self.0.clone(),
            },
            FieldReadonlyValue::new()
                .children(format!("Fallback: {}", reflect.reflect_type_path())),
        ))
        .into_view()
    }
}
