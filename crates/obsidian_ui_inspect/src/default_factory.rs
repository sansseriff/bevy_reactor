use crate::{
    edit_bool::FieldEditBool, edit_color::FieldEditSrgba, edit_fallback::FieldEditFallback,
    InspectableField, InspectorFactory,
};
use bevy::{prelude::*, reflect::ReflectRef};
use bevy_reactor::*;
use obsidian_ui::controls::{InspectorFieldLabel, InspectorFieldReadonlyValue};

#[derive(Default)]
pub struct DefaultInspectorFactory;

impl InspectorFactory for DefaultInspectorFactory {
    fn create_inspector(
        &self,
        name: &str,
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
                    InspectorFieldLabel {
                        children: name.into_view(),
                        ..default()
                    }
                    .into_view(),
                );
                views.push(
                    InspectorFieldReadonlyValue::new()
                        .children("TupleStruct:TODO")
                        .into_view(),
                );
            }
            ReflectRef::Tuple(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into_view(),
                        ..default()
                    }
                    .into_view(),
                );
                views.push(
                    InspectorFieldReadonlyValue::new()
                        .children("Tuple:TODO")
                        .into_view(),
                );
            }
            ReflectRef::List(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into_view(),
                        ..default()
                    }
                    .into_view(),
                );
                views.push(
                    InspectorFieldReadonlyValue::new()
                        .children("List:TODO")
                        .into_view(),
                );
            }
            ReflectRef::Array(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into_view(),
                        ..default()
                    }
                    .into_view(),
                );
                views.push(
                    InspectorFieldReadonlyValue::new()
                        .children("Array:TODO")
                        .into_view(),
                );
            }
            ReflectRef::Map(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into_view(),
                        ..default()
                    }
                    .into_view(),
                );
                views.push(
                    InspectorFieldReadonlyValue::new()
                        .children("Map:TODO")
                        .into_view(),
                );
            }
            ReflectRef::Enum(_) => {
                views.push(
                    InspectorFieldLabel {
                        children: name.into_view(),
                        ..default()
                    }
                    .into_view(),
                );
                views.push(
                    InspectorFieldReadonlyValue::new()
                        .children("Enum:TODO")
                        .into_view(),
                );
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
