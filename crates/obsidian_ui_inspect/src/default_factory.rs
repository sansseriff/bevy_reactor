use crate::{
    editors::{bool::FieldEditBool, color::FieldEditSrgba, fallback::FieldEditFallback},
    templates::{field_label::FieldLabel, field_readonly_value::FieldReadonlyValue},
    InspectableField, InspectorFactory,
};
use bevy::{prelude::*, reflect::ReflectRef};
use bevy_reactor::*;

#[derive(Default)]
pub struct DefaultInspectorFactory;

impl InspectorFactory for DefaultInspectorFactory {
    fn create_inspector(
        &self,
        _name: &str,
        reflect: &dyn Reflect,
        field: &InspectableField,
        views: &mut Vec<ViewRef>,
    ) -> bool {
        match reflect.reflect_ref() {
            ReflectRef::Struct(s) => {
                match s.reflect_type_path() {
                    "bevy_color::srgba::Srgba" => {
                        views.push(
                            FieldEditSrgba {
                                field: field.clone(),
                            }
                            .into_view(),
                        );
                        return true;
                    }

                    _ => {
                        views.push(FieldEditFallback(field.clone()).into_view());
                    }
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
            ReflectRef::TupleStruct(_) => {
                views.push(
                    FieldLabel {
                        field: field.clone(),
                    }
                    .into_view(),
                );
                views.push(
                    FieldReadonlyValue::new()
                        .children("TupleStruct:TODO")
                        .into_view(),
                );
            }
            ReflectRef::Tuple(_) => {
                views.push(
                    FieldLabel {
                        field: field.clone(),
                    }
                    .into_view(),
                );
                views.push(FieldReadonlyValue::new().children("Tuple:TODO").into_view());
            }
            ReflectRef::List(_) => {
                views.push(
                    FieldLabel {
                        field: field.clone(),
                    }
                    .into_view(),
                );
                views.push(FieldReadonlyValue::new().children("List:TODO").into_view());
            }
            ReflectRef::Array(_) => {
                views.push(
                    FieldLabel {
                        field: field.clone(),
                    }
                    .into_view(),
                );
                views.push(FieldReadonlyValue::new().children("Array:TODO").into_view());
            }
            ReflectRef::Map(_) => {
                views.push(
                    FieldLabel {
                        field: field.clone(),
                    }
                    .into_view(),
                );
                views.push(FieldReadonlyValue::new().children("Map:TODO").into_view());
            }
            ReflectRef::Enum(_) => {
                views.push(
                    FieldLabel {
                        field: field.clone(),
                    }
                    .into_view(),
                );
                views.push(FieldReadonlyValue::new().children("Enum:TODO").into_view());
            }
            ReflectRef::Value(v) => match v.reflect_type_path() {
                "bool" => {
                    views.push(FieldEditBool(field.clone()).into_view());
                }

                _ => {
                    views.push(FieldEditFallback(field.clone()).into_view());
                }
            },
        }
        true
    }
}
