use crate::{
    inspectors::{
        bool::BooleanFieldInspector, color::SrgbaFieldInspector, f32::F32FieldInspector,
        fallback::FieldEditFallback, list::FieldEditList, vec3::Vec3FieldInspector,
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
    fn create_inspector(&self, cx: &Cx, field: &Inspectable) -> Option<ViewRef> {
        let reflect = field.reflect(cx)?;
        match reflect.reflect_ref() {
            ReflectRef::Struct(s) => {
                match s.reflect_type_path() {
                    "bevy_color::srgba::Srgba" => Some(
                        SrgbaFieldInspector {
                            field: field.clone(),
                        }
                        .into_view(),
                    ),

                    "glam::Vec3" => Some(Vec3FieldInspector(field.clone()).into_view()),

                    _ => Some(FieldEditFallback(field.clone()).into_view()),
                }
                // println!("Struct: {}", s.reflect_type_path());
                // views.push(
                //     InspectorFieldLabel {
                //         children: (
                //             name,
                //             Spacer,
                //             // Button {
                //             //     children: Element::<NodeBundle>::new()
                //             //         .with_styles(style_close_icon_small)
                //             //         .into(),
                //             //     size: Size::Xxxs,
                //             //     minimal: true,
                //             //     ..default()
                //             // },
                //         )
                //             .to_ref(),
                //         ..default()
                //     }
                //     .to_ref(),
                // );
                // views.push(
                //     InspectorFieldReadonlyValue {
                //         children: "Struct:TODO".into(),
                //         ..default()
                //     }
                //     .to_ref(),
                // );
            }
            ReflectRef::TupleStruct(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("TupleStruct:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::Tuple(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Tuple:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::List(_) => Some(FieldEditList(field.clone()).into_view()),
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

                _ => Some(FieldEditFallback(field.clone()).into_view()),
            },
        }
    }
}
