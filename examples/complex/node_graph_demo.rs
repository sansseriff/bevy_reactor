use bevy::{color::palettes, prelude::*};
use bevy_reactor::*;
use obsidian_ui::{
    colors,
    controls::{InputConnector, NodeGraph, NodeGraphNode, OutputConnector, Slider},
};

#[derive(Clone, Debug, PartialEq, Component)]
pub struct NodePosition(pub Vec2);

#[derive(Clone, Debug, PartialEq, Component)]
pub struct NodeTitle(pub String);

#[derive(Debug, Resource)]
pub struct DemoGraphRoot {
    nodes: Vec<Entity>,
    edges: Vec<Entity>,
}

impl FromWorld for DemoGraphRoot {
    fn from_world(world: &mut World) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        nodes.push(
            world
                .spawn((
                    NodePosition(Vec2::new(100., 100.)),
                    NodeTitle("Node 1".to_string()),
                ))
                .id(),
        );
        nodes.push(
            world
                .spawn((
                    NodePosition(Vec2::new(200., 220.)),
                    NodeTitle("Node 2".to_string()),
                ))
                .id(),
        );
        nodes.push(
            world
                .spawn((
                    NodePosition(Vec2::new(300., 100.)),
                    NodeTitle("Node 3".to_string()),
                ))
                .id(),
        );
        Self { nodes, edges }
    }
}

pub struct NodeGraphDemo {}

fn style_node_graph(ss: &mut StyleBuilder) {
    ss.flex_grow(1.).border_left(1).border_color(Color::BLACK);
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.min_width(150.);
}

impl ViewTemplate for NodeGraphDemo {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        NodeGraph {
            children: (For::each(
                |cx| {
                    let graph = cx.use_resource::<DemoGraphRoot>();
                    graph.nodes.clone().into_iter()
                },
                |id| NodeTemplate { id: *id },
            ),)
                .fragment(),
            style: StyleHandle::new(style_node_graph),
        }
    }
}

pub struct NodeTemplate {
    id: Entity,
}

impl ViewTemplate for NodeTemplate {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        let id = self.id;
        let position = cx.create_derived(move |cx| cx.use_component::<NodePosition>(id).unwrap().0);
        let title =
            cx.create_derived(move |cx| cx.use_component::<NodeTitle>(id).unwrap().0.clone());
        NodeGraphNode {
            position,
            title,
            children: (
                OutputConnector {
                    label: "RGB".to_string(),
                    color: colors::RESOURCE,
                },
                InputConnector {
                    color: colors::LIGHT,
                    control: Slider {
                        value: Signal::Constant(50.),
                        min: Signal::Constant(0.),
                        max: Signal::Constant(100.),
                        label: Some("Base".to_string()),
                        style: StyleHandle::new(style_slider),
                        // on_change: todo!(),
                        ..default()
                    }
                    .into(),
                },
                InputConnector {
                    color: colors::LIGHT,
                    control: Slider {
                        value: Signal::Constant(80.),
                        min: Signal::Constant(0.),
                        max: Signal::Constant(100.),
                        label: Some("Mask".to_string()),
                        style: StyleHandle::new(style_slider),
                        // on_change: todo!(),
                        ..default()
                    }
                    .into(),
                },
            )
                .fragment(),
            selected: Signal::Constant(false),
            on_drag: Some(cx.create_callback(move |cx, new_pos| {
                let mut entt = cx.world_mut().entity_mut(id);
                let mut pos = entt.get_mut::<NodePosition>().unwrap();
                pos.0 = new_pos;
            })),
        }
    }
}
