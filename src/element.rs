use bevy::{core::Name, prelude::*, render::view::Visibility, ui::node_bundles::NodeBundle};

use crate::{
    node_span::NodeSpan,
    view::{View, ViewContext},
    view_tuple::ViewTuple,
    IntoView, ViewHandle,
};

/// A basic UI element
pub struct Element {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// View entities that are children of this element.
    children: Vec<Box<dyn View + Send + Sync>>,
}

impl Element {
    pub fn new() -> Self {
        Self {
            node: None,
            children: Vec::new(),
        }
    }

    pub fn for_entity(node: Entity) -> Self {
        Self {
            node: Some(node),
            children: Vec::new(),
        }
    }

    pub fn children<V: ViewTuple>(self, views: V) -> Self {
        if !self.children.is_empty() {
            panic!("Children already set");
        }
        let mut result = Self {
            node: None,
            children: Vec::new(),
        };
        views.gather(&mut result.children);
        result
    }
}

impl View for Element {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Node(self.node.unwrap())
    }

    fn build(&mut self, vc: &mut ViewContext) {
        if self.node.is_none() {
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
        }

        let mut count: usize = 0;
        for child in self.children.iter_mut() {
            child.build(vc);
            count += child.nodes().count();
        }

        let mut flat: Vec<Entity> = Vec::with_capacity(count);
        for child in self.children.iter() {
            child.nodes().flatten(&mut flat);
        }

        vc.world
            .entity_mut(self.node.unwrap())
            .replace_children(&flat);
    }

    // fn assemble(&self, bc: &mut BuildContext, state: &mut Self::State) -> NodeSpan {
    //     let nodes = self.inner.assemble(bc, &mut state.0);
    //     let children = self.items.assemble_spans(bc, &mut state.1);
    //     if let NodeSpan::Node(parent) = nodes {
    //         // Attach child view outputs to parent.
    //         let mut flat: Vec<Entity> = Vec::with_capacity(children.count());
    //         children.flatten(&mut flat);

    //         let mut em = bc.entity_mut(parent);
    //         if let Some(children) = em.get::<Children>() {
    //             // See if children changed
    //             if !children.eq(&flat) {
    //                 em.replace_children(&flat);
    //             }
    //         } else {
    //             // No children, unconditional replace
    //             em.replace_children(&flat);
    //         }
    //     } else if nodes != NodeSpan::Empty {
    //         panic!("Children can only be parented to a single node");
    //     }
    //     nodes
    // }
}

impl IntoView for Element {
    fn into_view(self) -> Box<dyn View + Send + Sync> {
        Box::new(self)
    }

    fn into_handle(self, world: &mut World) -> Entity {
        world.spawn(ViewHandle::new(self)).id()
    }
}
