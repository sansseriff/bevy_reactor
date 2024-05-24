use std::sync::Arc;

use bevy::reflect::{ParsedPath, ReflectKind};
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextSetup};
use obsidian_ui::controls::Spacer;

use crate::{
    inspectors::{
        r#struct::{StructFieldList, StructInspectorHeaderControls},
        tuple_struct::TupleStructElements,
    },
    templates::inspector_panel::InspectorPanel,
    Inspectable, InspectableRoot,
};

pub struct Inspector {
    // Reference to the entity being inspected
    target: Arc<dyn InspectableRoot>,
}

impl Inspector {
    pub fn new(target: Arc<dyn InspectableRoot>) -> Self {
        Self { target }
    }

    fn create_fields(&self, cx: &mut Cx, inspectable: Arc<Inspectable>) -> ViewRef {
        let access = inspectable.clone();
        let field_type =
            cx.create_memo(move |cx| access.reflect(cx).unwrap().reflect_kind().to_owned());
        DynamicKeyed::new(
            move |cx| field_type.get(cx),
            move |ftype| match ftype {
                ReflectKind::Struct => StructFieldList(inspectable.clone()).into_view(),
                ReflectKind::TupleStruct => TupleStructElements(inspectable.clone()).into_view(),
                ReflectKind::Tuple => todo!(),
                ReflectKind::List => todo!(),
                ReflectKind::Array => todo!(),
                ReflectKind::Map => todo!(),
                ReflectKind::Enum => todo!(),
                ReflectKind::Value => todo!(),
            },
        )
        .into_view()
    }
}

impl ViewTemplate for Inspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let path = ParsedPath(vec![]);
        let inspectable = Arc::new(Inspectable {
            root: self.target.clone(),
            name: self.target.name(cx).clone(),
            value_path: path.clone(),
            field_path: path,
            can_remove: true,
            attributes: None,
        });
        InspectorPanel::new()
            .title((
                self.target.name(cx),
                Spacer,
                StructInspectorHeaderControls {
                    target: inspectable.clone(),
                },
            ))
            .body(self.create_fields(cx, inspectable))
            .expanded(true)
    }
}
