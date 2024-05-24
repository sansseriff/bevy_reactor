use std::sync::Arc;

use crate::{
    inspectors::{
        bool::BooleanFieldInspector, color::SrgbaFieldInspector, f32::F32FieldInspector,
        fallback::FallbackInspector, list::ListInspector, r#struct::NestedStruct,
        tuple_struct::NestedTupleStruct, vec3::Vec3FieldInspector,
    },
    templates::{field_label::FieldLabel, field_readonly_value::FieldReadonlyValue},
    Inspectable, InspectorFactory,
};
use bevy::reflect::ReflectRef;
use bevy_reactor::*;
use bevy_reactor_signals::Cx;

#[derive(Default)]
pub struct DefaultInspectorFactory;

impl InspectorFactory for DefaultInspectorFactory {
    fn create_inspector(&self, cx: &Cx, field: Arc<Inspectable>) -> Option<ViewRef> {
        let reflect = field.reflect(cx)?;
        match reflect.reflect_ref() {
            ReflectRef::Struct(s) => match s.reflect_type_path() {
                "bevy_color::srgba::Srgba" => Some(SrgbaFieldInspector(field.clone()).into_view()),
                "glam::Vec3" => Some(Vec3FieldInspector(field.clone()).into_view()),
                _ => Some(NestedStruct(field.clone()).into_view()),
            },
            ReflectRef::TupleStruct(_) => Some(NestedTupleStruct(field.clone()).into_view()),
            ReflectRef::Tuple(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Tuple:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::List(_) => Some(ListInspector(field.clone()).into_view()),
            ReflectRef::Array(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Array:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::Map(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Map:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::Enum(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Enum:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::Value(v) => match v.reflect_type_path() {
                "bool" => Some(BooleanFieldInspector(field.clone()).into_view()),
                "f32" => Some(F32FieldInspector(field.clone()).into_view()),
                _ => Some(FallbackInspector(field.clone()).into_view()),
            },
        }
    }
}
