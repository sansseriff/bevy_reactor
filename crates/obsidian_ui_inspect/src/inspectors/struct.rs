use std::sync::Arc;

use bevy::{
    color::Color,
    ecs::reflect::AppTypeRegistry,
    reflect::{
        std_traits::ReflectDefault, DynamicEnum, DynamicTuple, OffsetAccess, ReflectKind,
        ReflectRef, TypeInfo, VariantInfo,
    },
    ui::{self, node_bundles::NodeBundle},
};
use bevy_mod_stylebuilder::*;
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextRead, RunContextSetup};
use obsidian_ui::{
    colors,
    controls::{DisclosureToggle, Icon, MenuButton, MenuItem, MenuPopup},
    floating::FloatAlign,
    size::Size,
};

use crate::{templates::field_label::FieldLabelWide, Inspectable, InspectorFactoryRegistry};

pub struct NestedStruct(pub(crate) Arc<Inspectable>);

fn style_field_list(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .grid_auto_flow(ui::GridAutoFlow::Row)
        .grid_template_columns(vec![
            ui::RepeatedGridTrack::auto(1),
            ui::RepeatedGridTrack::flex(1, 1.),
        ])
        .column_gap(4)
        .row_gap(2)
        .align_items(ui::AlignItems::Stretch)
        .grid_column_span(2)
        .min_width(64)
        .color(colors::DIM)
        .margin_left(16)
        .margin_top(4)
        .margin_bottom(4);
}

impl ViewTemplate for NestedStruct {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let expanded = cx.create_mutable(false);

        Fragment::new((
            FieldLabelWide::new(field.clone())
                .name(Fragment::new((
                    DisclosureToggle::new()
                        .size(Size::Xs)
                        .expanded(expanded)
                        .on_change(cx.create_callback(move |cx, value: bool| {
                            expanded.set(cx, value);
                        })),
                    field.name.clone(),
                )))
                .buttons(StructInspectorHeaderControls {
                    target: self.0.clone(),
                    // expanded,
                }),
            Cond::new(
                expanded.signal(),
                {
                    let field = self.0.clone();
                    move || {
                        Element::<NodeBundle>::new()
                            .style(style_field_list)
                            .children(StructFieldList(field.clone()))
                    }
                },
                || (),
            ),
        ))
    }
}

pub struct StructFieldList(pub Arc<Inspectable>);

impl ViewTemplate for StructFieldList {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let target = self.0.clone();
        let reflect = self.0.reflect(cx).unwrap();
        let info = reflect.get_represented_type_info().unwrap();

        // Get the memoized field names of the struct, minus missing optionals. This should
        // isolate the field editors from each other so that they don't constantly update.
        // We will still need to memoize the individual field values.
        let field_names = cx.create_memo(move |cx| {
            let ReflectRef::Struct(st) = target.reflect(cx).unwrap().reflect_ref() else {
                panic!("Expected ReflectRef::Struct")
            };
            let num_fields = st.field_len();
            let mut names = Vec::with_capacity(num_fields);
            // Filter out field names for fields with a value of `None`.
            for findex in 0..num_fields {
                let field = st.field_at(findex).unwrap();
                // let info = st.get_represented_type_info().unwrap()
                if field.reflect_kind() == ReflectKind::Enum
                    && field
                        .reflect_type_path()
                        .starts_with("core::option::Option")
                {
                    let ReflectRef::Enum(enum_ref) = field.reflect_ref() else {
                        panic!("Expected ReflectRef::Enum");
                    };
                    if enum_ref.variant_name() != "None" {
                        names.push(st.name_at(findex).unwrap().to_string());
                    }
                } else {
                    names.push(st.name_at(findex).unwrap().to_string());
                }
            }
            names
        });
        let target = self.0.clone();
        For::each(
            move |cx| field_names.get_clone(cx).into_iter(),
            move |name| {
                let mut path = target.field_path.clone();
                path.0.push(OffsetAccess {
                    access: bevy::reflect::Access::Field(name.clone().into()),
                    offset: None,
                });
                let TypeInfo::Struct(st_info) = info else {
                    panic!("Expected StructInfo");
                };
                let field_info = st_info.field(name).unwrap();
                let attrs = field_info.custom_attributes();
                let field = Arc::new(Inspectable {
                    root: target.root.clone(),
                    name: name.to_string(),
                    value_path: path.clone(),
                    field_path: path,
                    can_remove: false,
                    attributes: Some(attrs),
                });
                NamedFieldInspector { field }.into_view()
            },
        )
    }
}

struct NamedFieldInspector {
    field: Arc<Inspectable>,
}

impl ViewTemplate for NamedFieldInspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let factories = cx.read_resource::<InspectorFactoryRegistry>();
        let field = self.field.clone();
        let Some(reflect) = field.reflect(cx) else {
            return ().into_view();
        };

        // If the field is Option<T>, and not None, then unwrap the value and inspect the
        // inner value.
        if reflect.reflect_kind() == ReflectKind::Enum
            && reflect
                .reflect_type_path()
                .starts_with("core::option::Option")
        {
            let ReflectRef::Enum(enum_ref) = reflect.reflect_ref() else {
                panic!("Expected ReflectRef::Enum");
            };
            if enum_ref.variant_name() != "None" {
                let mut path = field.value_path.clone();
                path.0.push(OffsetAccess {
                    access: bevy::reflect::Access::TupleIndex(0),
                    offset: None,
                });

                let access = Arc::new(Inspectable {
                    root: field.root.clone(),
                    name: field.name.clone(),
                    value_path: path,
                    field_path: field.value_path.clone(),
                    can_remove: true,
                    attributes: field.attributes,
                });
                if let Some(view_ref) = factories.create_inspector(cx, access) {
                    return view_ref;
                }
            }
        } else if let Some(view_ref) = factories.create_inspector(cx, field) {
            return view_ref;
        }

        // No inspector found, don't render anything. Note that default factory already
        // has a fallback, so this should never be reached.
        ().into_view()
    }
}

pub struct StructInspectorHeaderControls {
    pub target: Arc<Inspectable>,
}

impl ViewTemplate for StructInspectorHeaderControls {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        let target = self.target.clone();
        let base_path = target.field_path.clone();
        Dynamic::new(move |cx| match target.reflect(cx).unwrap().reflect_ref() {
            ReflectRef::Struct(st) => {
                let num_fields = st.field_len();
                let mut items: Vec<ViewRef> = Vec::new();
                let registry = cx.world().resource::<AppTypeRegistry>().0.clone();
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
                        let Some(TypeInfo::Enum(enum_info)) = field.get_represented_type_info()
                        else {
                            panic!("Expected TypeInfo::Enum");
                        };

                        if enum_ref.variant_name() == "None" {
                            let some_variant = enum_info.variant("Some").unwrap();
                            let VariantInfo::Tuple(tuple_info) = some_variant else {
                                panic!()
                            };
                            let some_field = tuple_info.field_at(0).unwrap();
                            let some_type_id = some_field.type_id();
                            let registry_lock = registry.read();
                            let some_type = registry_lock.get_type_info(some_type_id).unwrap();
                            let some_default =
                                registry_lock.get_type_data::<ReflectDefault>(some_type_id);
                            if some_default.is_some() {
                                let mut path = base_path.clone();
                                path.0.push(OffsetAccess {
                                    access: bevy::reflect::Access::Field(name.to_string().into()),
                                    offset: None,
                                });
                                items.push(
                                    AddStructFieldItem {
                                        field: Arc::new(Inspectable {
                                            root: target.root.clone(),
                                            name: name.to_string(),
                                            value_path: path.clone(),
                                            field_path: path,
                                            can_remove: false,
                                            attributes: None,
                                        }),
                                    }
                                    .into_view(),
                                );
                            } else {
                                println!(
                                    "Can't find ReflectDefault for: {:?}",
                                    some_type.type_path()
                                );
                            }
                        }
                    }
                }

                if !items.is_empty() {
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
                        .size(Size::Xs)
                        .minimal(true)
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
        })
    }
}

struct AddStructFieldItem {
    field: Arc<Inspectable>,
}

impl ViewTemplate for AddStructFieldItem {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.field.clone();
        let callback = cx.create_callback(move |cx, _| {
            let Some(field_reflect) = field.reflect(cx) else {
                return;
            };
            let Some(TypeInfo::Enum(enum_info)) = field_reflect.get_represented_type_info() else {
                panic!("Expected TypeInfo::Enum");
            };

            // let field = target.get_field(cx, &path);
            let registry = cx.world().resource::<AppTypeRegistry>().0.clone();
            let some_variant = enum_info.variant("Some").unwrap();
            let VariantInfo::Tuple(tuple_info) = some_variant else {
                panic!("Expected VariantInfo::Tuple");
            };
            let some_field = tuple_info.field_at(0).unwrap();
            let some_type_id = some_field.type_id();
            let registry_lock = registry.read();
            let some_type = registry_lock.get_type_info(some_type_id).unwrap();
            if some_type.is::<bool>() {
                // For Option<bool> we assume that the user wants a default of 'true', because
                // that's the most common use case. This is because for most fields, `Some(false)`
                // is the same as `None`.
                let mut data = DynamicTuple::default();
                data.insert_boxed(Box::new(true));
                let dynamic_enum = DynamicEnum::new("Some", data);
                field.set_value(cx, &dynamic_enum);
            } else {
                let some_default = registry_lock.get_type_data::<ReflectDefault>(some_type_id);
                if some_default.is_some() {
                    // The value that needs to get wrapped in `Some`.
                    let default = some_default.unwrap().default();
                    let mut data = DynamicTuple::default();
                    data.insert_boxed(default);
                    let dynamic_enum = DynamicEnum::new("Some", data);
                    field.set_value(cx, &dynamic_enum);
                } else {
                    println!("Can't find ReflectDefault for: {:?}", some_type.type_path());
                }
            }
        });
        MenuItem::new()
            .label(self.field.name.clone())
            .on_click(callback)
    }
}

fn style_menu_icon(ss: &mut StyleBuilder) {
    ss.margin((4, 0));
}
