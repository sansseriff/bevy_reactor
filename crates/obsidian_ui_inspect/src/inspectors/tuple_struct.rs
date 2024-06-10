use std::sync::Arc;

use bevy::{
    reflect::{OffsetAccess, ReflectRef},
    ui::{self, node_bundles::NodeBundle},
};
use bevy_mod_stylebuilder::*;
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextRead, RunContextSetup};
use obsidian_ui::{colors, controls::DisclosureToggle, size::Size};

use crate::{templates::field_label::FieldLabelWide, Inspectable, InspectorFactoryRegistry};

pub struct NestedTupleStruct(pub(crate) Arc<Inspectable>);

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

impl ViewTemplate for NestedTupleStruct {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let expanded = cx.create_mutable(false);

        Fragment::new((
            FieldLabelWide::new(field.clone()).name(Fragment::new((
                DisclosureToggle::new()
                    .size(Size::Xs)
                    .expanded(expanded)
                    .on_change(cx.create_callback(move |cx, value: bool| {
                        expanded.set(cx, value);
                    })),
                field.name.clone(),
            ))),
            Cond::new(
                expanded.signal(),
                {
                    let field = self.0.clone();
                    move || {
                        Element::<NodeBundle>::new()
                            .style(style_field_list)
                            .children(TupleStructElements(field.clone()))
                    }
                },
                || (),
            ),
        ))
    }
}

pub struct TupleStructElements(pub Arc<Inspectable>);

impl ViewTemplate for TupleStructElements {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let length = if let ReflectRef::TupleStruct(list) = field.reflect(cx).unwrap().reflect_ref()
        {
            list.field_len()
        } else {
            0
        };

        // If there's only one field, then hoist it up a level and don't show field index.
        // TODO: If the singular field is a struct or vector, hoist that as well.
        if length == 1 {
            let factories = cx.use_resource::<InspectorFactoryRegistry>();
            let mut path = field.value_path.clone();
            path.0.push(OffsetAccess {
                access: bevy::reflect::Access::TupleIndex(0),
                offset: None,
            });
            let access = Arc::new(Inspectable {
                root: field.root.clone(),
                name: "0".to_string(),
                value_path: path,
                field_path: field.value_path.clone(),
                can_remove: false,
                attributes: field.attributes,
            });
            if let Some(view_ref) = factories.create_inspector(cx, access) {
                return view_ref;
            }
        }

        Element::<NodeBundle>::new()
            .style(style_field_list)
            .children(For::index(
                move |_cx| 0..length,
                move |_, index| {
                    let mut path = field.value_path.clone();
                    path.0.push(OffsetAccess {
                        access: bevy::reflect::Access::TupleIndex(index),
                        offset: None,
                    });
                    let access = Arc::new(Inspectable {
                        root: field.root.clone(),
                        name: format!("{}", index),
                        value_path: path,
                        field_path: field.value_path.clone(),
                        can_remove: false,
                        attributes: field.attributes,
                    });
                    TupleItemInspector { field: access }.into_view()
                },
            ))
            .into_view()
    }
}

struct TupleItemInspector {
    field: Arc<Inspectable>,
}

impl ViewTemplate for TupleItemInspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let factories = cx.use_resource::<InspectorFactoryRegistry>();
        // Either create an inspector for the field, or return an empty view.
        factories
            .create_inspector(cx, self.field.clone())
            .unwrap_or_else(|| ().into_view())
    }
}
