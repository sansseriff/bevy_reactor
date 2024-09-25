use std::sync::Arc;

use bevy::{
    ecs::reflect::AppTypeRegistry,
    reflect::{
        std_traits::ReflectDefault, DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant,
        OffsetAccess, ReflectRef, TypeInfo, TypeRegistry, VariantInfo, VariantType,
    },
};
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextRead, RunContextSetup};
use obsidian_ui::{
    controls::{MenuButton, MenuItem, MenuPopup},
    floating::{FloatAlign, FloatSide},
    size::Size,
};

use crate::{templates::field_label::FieldLabel, Inspectable, InspectorFactoryRegistry};

pub struct EnumInspector(pub(crate) Arc<Inspectable>);

impl ViewTemplate for EnumInspector {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        Fragment::new((
            FieldLabel {
                field: self.0.clone(),
            },
            VariantSelector {
                target: self.0.clone(),
            },
            Dynamic::new(move |cx| {
                let factories = cx.read_resource::<InspectorFactoryRegistry>();
                if let Some(reflect) = field.reflect(cx) {
                    if let ReflectRef::Enum(en) = reflect.reflect_ref() {
                        let variant = en.variant_type();
                        return match variant {
                            VariantType::Struct => {
                                let mut fields: Vec<ViewRef> = Vec::new();
                                for findex in 0..en.field_len() {
                                    let name = en.name_at(findex).unwrap().to_string();
                                    let mut path = field.value_path.clone();
                                    path.0.push(OffsetAccess {
                                        access: bevy::reflect::Access::Field(name.clone().into()),
                                        offset: None,
                                    });

                                    let access = Arc::new(Inspectable {
                                        root: field.root.clone(),
                                        name: name.clone(),
                                        value_path: path,
                                        field_path: field.value_path.clone(),
                                        can_remove: false,
                                        attributes: field.attributes,
                                    });
                                    if let Some(view_ref) = factories.create_inspector(cx, access) {
                                        fields.push(view_ref);
                                    }
                                }
                                Fragment::new(fields).into_view()
                            }

                            VariantType::Tuple => {
                                let mut fields: Vec<ViewRef> = Vec::new();
                                for findex in 0..en.field_len() {
                                    // let variant = en.field_at(findex).unwrap();
                                    let mut path = field.value_path.clone();
                                    path.0.push(OffsetAccess {
                                        access: bevy::reflect::Access::TupleIndex(findex),
                                        offset: None,
                                    });

                                    let access = Arc::new(Inspectable {
                                        root: field.root.clone(),
                                        name: if en.field_len() > 1 {
                                            format!("{}", findex)
                                        } else {
                                            "".to_string()
                                        },
                                        value_path: path.clone(),
                                        field_path: path,
                                        can_remove: false,
                                        attributes: field.attributes,
                                    });
                                    if let Some(view_ref) = factories.create_inspector(cx, access) {
                                        fields.push(view_ref);
                                    }
                                }
                                Fragment::new(fields).into_view()
                            }

                            VariantType::Unit => ().into_view(),
                        };
                    }
                }
                ().into_view()
            }),
        ))
    }
}

pub struct VariantSelector {
    pub target: Arc<Inspectable>,
}

impl ViewTemplate for VariantSelector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let target = self.target.clone();
        let variant_name = cx.create_memo(move |cx| {
            if let Some(reflect) = target.reflect(cx) {
                if let ReflectRef::Enum(en) = reflect.reflect_ref() {
                    return en.variant_name().to_string();
                }
            }
            "".to_string()
        });

        let target = self.target.clone();
        Dynamic::new(move |cx| {
            match target
                .reflect(cx)
                .unwrap()
                .get_represented_type_info()
                .unwrap()
            {
                TypeInfo::Enum(en) => {
                    let num_variants = en.variant_len();
                    let mut items: Vec<ViewRef> = Vec::new();
                    let registry = cx.world().resource::<AppTypeRegistry>().0.clone();
                    let registry_lock = registry.read();
                    for findex in 0..num_variants {
                        let variant = en.variant_at(findex).unwrap();
                        let variant_default = variant_default_value(variant, &registry_lock);
                        if variant_default.is_none() {
                            continue;
                        }
                        items.push(
                            SetVariantItem {
                                field: target.clone(),
                                variant_name: variant.name().to_string(),
                                variant_index: findex,
                            }
                            .into_view(),
                        );
                    }

                    if !items.is_empty() {
                        let variant_name = variant_name.clone();
                        MenuButton::new()
                            .children(TextComputed::new(move |cx| variant_name.get_clone(cx)))
                            .popup(
                                MenuPopup::new()
                                    .side(FloatSide::Bottom)
                                    .align(FloatAlign::End)
                                    .children(Fragment::from_slice(&items).into_view()),
                            )
                            .size(Size::Sm)
                            .into_view()
                    } else {
                        ().into_view()
                    }
                }
                _ => {
                    println!(
                        "Fallback: {}",
                        target.reflect(cx).unwrap().reflect_type_path()
                    );
                    ().into_view()
                }
            }
        })
    }
}

struct SetVariantItem {
    field: Arc<Inspectable>,
    variant_name: String,
    variant_index: usize,
}

impl ViewTemplate for SetVariantItem {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.field.clone();
        let variant_index = self.variant_index;
        let callback = cx.create_callback(move |cx, _| {
            let Some(field_reflect) = field.reflect(cx) else {
                return;
            };
            let Some(TypeInfo::Enum(enum_info)) = field_reflect.get_represented_type_info() else {
                panic!("Expected TypeInfo::Enum");
            };

            let registry = cx.world().resource::<AppTypeRegistry>().0.clone();
            let variant = enum_info.variant_at(variant_index).unwrap();
            let registry_lock = registry.read();
            let variant_default = variant_default_value(variant, &registry_lock);
            if let Some(def) = variant_default {
                field.set_value(cx, &def);
            } else {
                println!("Can't find ReflectDefault for: {:?}", variant.name());
            }
        });
        MenuItem::new()
            .label(self.variant_name.clone())
            .on_click(callback)
    }
}

fn variant_default_value(variant: &VariantInfo, registry: &TypeRegistry) -> Option<DynamicEnum> {
    match variant {
        bevy::reflect::VariantInfo::Struct(st) => {
            let mut ds = DynamicStruct::default();
            for field in 0..st.field_len() {
                let f = st.field_at(field).unwrap();
                // let field_type = registry.get_type_info(f.type_id()).unwrap();
                let field_type_default = registry.get_type_data::<ReflectDefault>(f.type_id());
                if let Some(default) = field_type_default {
                    let default = default.default();
                    ds.insert_boxed(f.name(), default);
                } else {
                    return None;
                }
            }
            Some(DynamicEnum::new(variant.name(), ds))
        }
        bevy::reflect::VariantInfo::Tuple(tpl) => {
            let mut dt = DynamicTuple::default();
            for field in 0..tpl.field_len() {
                let f = tpl.field_at(field).unwrap();
                // let field_type = registry.get_type_info(f.type_id()).unwrap();
                let field_type_default = registry.get_type_data::<ReflectDefault>(f.type_id());
                if let Some(default) = field_type_default {
                    let default = default.default();
                    dt.insert_boxed(default);
                } else {
                    return None;
                }
            }
            Some(DynamicEnum::new(variant.name(), dt))
        }
        bevy::reflect::VariantInfo::Unit(_) => {
            Some(DynamicEnum::new(variant.name(), DynamicVariant::Unit))
        }
    }
}
