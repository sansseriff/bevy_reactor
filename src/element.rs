use std::sync::{Arc, Mutex};

use bevy::{core::Name, prelude::*, render::view::Visibility, ui::node_bundles::NodeBundle};

use crate::{
    node_span::NodeSpan,
    view::{View, ViewContext},
    view_tuple::ViewTuple,
    IntoView, ViewHandle, ViewRef,
};

/// A basic UI element
#[derive(Default)]
pub struct Element {
    /// The visible UI node for this element.
    display: Option<Entity>,

    /// Children of this element.
    children: Vec<ViewRef>,

    /// Children after they have been spawned as entities.
    child_entities: Vec<Entity>,
}

impl Element {
    /// Construct a new `Element`.
    pub fn new() -> Self {
        Self {
            display: None,
            children: Vec::new(),
            child_entities: Vec::new(),
        }
    }

    /// Construct a new `Element` with a given entity id.
    pub fn for_entity(node: Entity) -> Self {
        Self {
            display: Some(node),
            children: Vec::new(),
            child_entities: Vec::new(),
        }
    }

    /// Set the child views for this element.
    pub fn children<V: ViewTuple>(mut self, views: V) -> Self {
        if !self.children.is_empty() {
            panic!("Children already set");
        }
        views.get_handles(&mut self.children);
        self
    }

    /// Attach the children to the node. Note that each child view may produce multiple nodes,
    /// or none.
    fn attach_children(&mut self, world: &mut World) {
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

        world
            .entity_mut(self.display.unwrap())
            .replace_children(&flat);
    }
}

impl View for Element {
    fn nodes(&self) -> NodeSpan {
        match self.display {
            None => NodeSpan::Empty,
            Some(node) => NodeSpan::Node(node),
        }
    }

    fn build(&mut self, _view_entity: Entity, vc: &mut ViewContext) {
        // Build element node
        assert!(self.display.is_none());
        self.display = Some(
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
        for child in self.children.iter() {
            let child_ent = vc.world.spawn(ViewHandle {
                view: child.clone(),
            });
            self.child_entities.push(child_ent.id());
            let child_view = child_ent.get::<ViewHandle>().unwrap();
            let child_inner = child_view.view.clone();
            child_inner.lock().unwrap().build(child_ent.id(), vc);
        }

        self.attach_children(vc.world);
    }

    fn raze(&mut self, _view_entity: Entity, world: &mut World) {
        assert!(self.display.is_some());
        // Raze all child views
        for child_ent in self.child_entities.drain(..) {
            let child = world.entity_mut(child_ent);
            let handle = child.get::<ViewHandle>().unwrap();
            let inner = handle.view.clone();
            inner.lock().unwrap().raze(child_ent, world);
        }
        // Delete the display node.
        world.entity_mut(self.display.unwrap()).despawn();
        self.display = None;
    }
}

impl IntoView for Element {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}
