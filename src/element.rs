use bevy::{core::Name, prelude::*, render::view::Visibility, ui::node_bundles::NodeBundle};

use crate::{
    node_span::NodeSpan,
    view::{View, ViewContext},
    view_tuple::ViewTuple,
    IntoViewHandle, ViewHandle,
};

/// A basic UI element
#[derive(Default)]
pub struct Element {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// Children of this element.
    children: Vec<ViewHandle>,

    /// Children after they have been spawned as entities.
    child_entities: Vec<Entity>,
}

impl Element {
    pub fn new() -> Self {
        Self {
            node: None,
            children: Vec::new(),
            child_entities: Vec::new(),
        }
    }

    pub fn for_entity(node: Entity) -> Self {
        Self {
            node: Some(node),
            children: Vec::new(),
            child_entities: Vec::new(),
        }
    }

    pub fn children<V: ViewTuple>(mut self, views: V) -> Self {
        if !self.children.is_empty() {
            panic!("Children already set");
        }
        views.get_handles(&mut self.children);
        self
    }

    fn assemble(&mut self, world: &mut World) {
        let mut count: usize = 0;
        for child_ent in self.child_entities.iter_mut() {
            let child = world.entity_mut(*child_ent);
            let handle = child.get::<ViewHandle>().unwrap();
            count += handle.nodes().count();
        }

        let mut flat: Vec<Entity> = Vec::with_capacity(count);
        for child_ent in self.child_entities.iter() {
            let child = world.entity_mut(*child_ent);
            let handle = child.get::<ViewHandle>().unwrap();
            handle.nodes().flatten(&mut flat);
        }

        world.entity_mut(self.node.unwrap()).replace_children(&flat);
    }
}

impl View for Element {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Node(self.node.unwrap())
    }

    fn build(&mut self, _view_entity: Entity, vc: &mut ViewContext) {
        // Build element node
        assert!(self.node.is_none());
        self.node = Some(
            vc.world
                .spawn((
                    NodeBundle {
                        visibility: Visibility::Visible,
                        style: Style {
                            display: Display::Flex,
                            width: Val::Px(10.0),
                            height: Val::Px(10.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        border_color: BorderColor(Color::RED),
                        ..default()
                    },
                    Name::new("element"),
                ))
                .id(),
        );

        // Build child nodes.
        for child in self.children.drain(..) {
            let child_ent = vc.world.spawn(child);
            self.child_entities.push(child_ent.id());
            let child_view = child_ent.get::<ViewHandle>().unwrap();
            let child_inner = child_view.view.clone();
            child_inner.lock().unwrap().build(child_ent.id(), vc);
        }

        self.assemble(vc.world);
    }
}

impl IntoViewHandle for Element {
    fn into_view_handle(self) -> ViewHandle {
        ViewHandle::new(self)
    }
}
