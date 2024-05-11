use std::sync::Arc;

use bevy::{
    color::Color,
    reflect::{OffsetAccess, ParsedPath, ReflectKind, ReflectRef},
};
use bevy_reactor::*;
use obsidian_ui::{
    colors,
    controls::{Icon, MenuButton, MenuItem, MenuPopup, Spacer},
    floating::FloatAlign,
    size::Size,
};

use crate::{
    templates::inspector_panel::InspectorPanel, Inspectable, InspectableField,
    InspectorFactoryRegistry,
};

pub struct Inspector {
    // Need a reference to the entity being inspected
    target: Arc<dyn Inspectable>,
}

impl Inspector {
    pub fn new(target: Arc<dyn Inspectable>) -> Self {
        Self { target }
    }

    fn create_fields(&self, cx: &mut Cx, target: Arc<dyn Inspectable>) -> ViewRef {
        match target.reflect(cx).reflect_ref() {
            ReflectRef::Struct(st) => {
                // TODO: Make list of fields reactive.
                let factories = cx.use_resource::<InspectorFactoryRegistry>();
                let num_fields = st.field_len();
                let mut fields: Vec<ViewRef> = Vec::with_capacity(num_fields);
                for findex in 0..num_fields {
                    let field = st.field_at(findex).unwrap();
                    let name = st.name_at(findex).unwrap();
                    if field.reflect_kind() == ReflectKind::Enum
                        && field
                            .reflect_type_path()
                            .starts_with("core::option::Option")
                    {
                        let ReflectRef::Enum(enum_ref) = field.reflect_ref() else {
                            panic!("Expected ReflectRef::Enum");
                        };
                        if enum_ref.variant_name() != "None" {
                            let mut path = ParsedPath::parse(name).unwrap();
                            path.0.push(OffsetAccess {
                                access: bevy::reflect::Access::TupleIndex(0),
                                offset: None,
                            });

                            let access = Arc::new(InspectableField {
                                root: target.clone(),
                                name: name.to_string(),
                                path,
                                can_remove: true,
                            });
                            for factory in factories.0.iter().rev() {
                                if factory.create_inspector(name, field, &access, &mut fields) {
                                    break;
                                }
                            }
                        }
                    } else {
                        let field = st.field_at(findex).unwrap();
                        let access = Arc::new(InspectableField {
                            root: target.clone(),
                            name: name.to_string(),
                            path: ParsedPath::parse(name).unwrap(),
                            can_remove: false,
                        });
                        for factory in factories.0.iter().rev() {
                            if factory.create_inspector(name, field, &access, &mut fields) {
                                break;
                            }
                        }
                    }
                }
                Fragment::from_slice(&fields).into_view()
            }
            ReflectRef::TupleStruct(_) => todo!(),
            ReflectRef::Tuple(_) => todo!(),
            ReflectRef::List(_) => todo!(),
            ReflectRef::Array(_) => todo!(),
            ReflectRef::Map(_) => todo!(),
            ReflectRef::Enum(_) => todo!(),
            ReflectRef::Value(_) => todo!(),
        }
    }
}

impl ViewTemplate for Inspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        InspectorPanel::new()
            .title((
                self.target.name(cx),
                Spacer,
                AddFieldsButton {
                    target: self.target.clone(),
                },
            ))
            .body(self.create_fields(cx, self.target.clone()))
            .expanded(true)
    }
}

struct AddFieldsButton {
    target: Arc<dyn Inspectable>,
}

impl ViewTemplate for AddFieldsButton {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        let target = self.target.clone();
        Dynamic::new(move |cx| match target.reflect(cx).reflect_ref() {
            ReflectRef::Struct(st) => {
                let mut missing_fields: Vec<String> = Vec::new();
                let num_fields = st.field_len();
                for findex in 0..num_fields {
                    let field = st.field_at(findex).unwrap();
                    let name = st.name_at(findex).unwrap();
                    if field.reflect_kind() == ReflectKind::Enum
                        && field
                            .reflect_type_path()
                            .starts_with("core::option::Option")
                    {
                        let ReflectRef::Enum(enum_ref) = field.reflect_ref() else {
                            panic!("Expected ReflectRef::Enum");
                        };
                        if enum_ref.variant_name() == "None" {
                            missing_fields.push(name.to_string());
                        }
                    }
                }

                if !missing_fields.is_empty() {
                    let mut items: Vec<ViewRef> = Vec::with_capacity(missing_fields.len());
                    for field in missing_fields.iter() {
                        items.push(
                            AddStructFieldItem {
                                target: target.clone(),
                                path: ParsedPath::parse(field).unwrap(),
                                name: field.to_string(),
                            }
                            .into_view(),
                        );
                    }
                    MenuButton::new()
                        .children(
                            Icon::new("obsidian_ui://icons/add_box.png")
                                .color(Color::from(colors::DIM))
                                .style(style_menu_icon),
                        )
                        .popup(
                            MenuPopup::new()
                                .align(FloatAlign::End)
                                .children(Fragment::from_slice(&items).into_view()),
                        )
                        .size(Size::Xxs)
                        .minimal(true)
                        .into_view()
                } else {
                    ().into_view()
                }
            }
            _ => {
                println!("Fallback: {}", target.reflect(cx).reflect_type_path());
                ().into_view()
            }
        })
    }
}

struct AddStructFieldItem {
    target: Arc<dyn Inspectable>,
    path: ParsedPath,
    name: String,
}

impl ViewTemplate for AddStructFieldItem {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let callback = cx.create_callback(|cx, _| {
            println!("Add field");
        });
        MenuItem::new().label(self.name.clone()).on_click(callback)
    }
}

fn style_menu_icon(ss: &mut StyleBuilder) {
    ss.margin((4, 0));
}

// fn style_close_icon_small(ss: &mut StyleBuilder) {
//     ss.height(10)
//         .width(10)
//         .background_image("obsidian_ui://icons/close.png")
//         .background_image_color(colors::DIM)
//         .margin((2, 0));
// }
