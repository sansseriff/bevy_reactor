use std::sync::Arc;

use bevy::reflect::{ParsedPath, ReflectKind, ReflectRef};
use bevy_reactor::*;
use obsidian_ui::{
    colors,
    controls::{Icon, InspectorGroup, MenuButton, MenuDivider, MenuItem, MenuPopup, Spacer},
    floating::FloatAlign,
    size::Size,
};

use crate::{Inspectable, InspectableField, InspectorFactoryRegistry};

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
            ReflectRef::Struct(str) => {
                // TODO: Make list of fields reactive.
                let factories = cx.use_resource::<InspectorFactoryRegistry>();
                let num_fields = str.field_len();
                let mut fields: Vec<ViewRef> = Vec::with_capacity(num_fields);
                for findex in 0..num_fields {
                    let field = str.field_at(findex).unwrap();
                    let name = str.name_at(findex).unwrap();
                    if field.reflect_kind() == ReflectKind::Enum
                        && field
                            .reflect_type_path()
                            .starts_with("core::option::Option")
                    {
                        let ReflectRef::Enum(enum_ref) = field.reflect_ref() else {
                            panic!("Expected ReflectRef::Enum");
                        };
                        let inner = enum_ref.field_at(0);
                        if enum_ref.variant_name() == "None" {
                            println!("Skipping None for {}", name);
                            continue;
                        } else {
                            // for factory in factories.0.iter().rev() {
                            //     if factory.create_inspector(name, inner.unwrap(), &mut fields) {
                            //         break;
                            //     }
                            // }
                        }
                    } else {
                        let field = str.field_at(findex).unwrap();
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
                Fragment::from_slice(&fields).into()
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
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        InspectorGroup::new()
            .title((
                self.target.name(cx),
                Spacer,
                MenuButton::new()
                    .children(
                        Icon::new("obsidian_ui://icons/add_box.png")
                            .color(colors::DIM.into())
                            .style(style_close_icon),
                    )
                    .popup(MenuPopup::new().align(FloatAlign::End).children((
                        MenuItem::new().label("Open"),
                        MenuItem::new().label("Close"),
                        MenuDivider,
                        MenuItem::new().label("Quit..."),
                    )))
                    .size(Size::Xxs)
                    .minimal(true),
            ))
            .body(self.create_fields(cx, self.target.clone()))
            .expanded(Signal::Constant(true))
    }
}

fn style_close_icon(ss: &mut StyleBuilder) {
    ss.margin((4, 0));
}

// fn style_close_icon_small(ss: &mut StyleBuilder) {
//     ss.height(10)
//         .width(10)
//         .background_image("obsidian_ui://icons/close.png")
//         .background_image_color(colors::DIM)
//         .margin((2, 0));
// }
