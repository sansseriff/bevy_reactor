use bevy::prelude::*;
use bevy_reactor::*;

use crate::colors;

fn style_node_graph(ss: &mut StyleBuilder) {
    ss.background_color(colors::U1)
        .background_image("obsidian_ui://textures/dot-grid.png");
}

/// An editable graph of nodes, connected by edges.
#[derive(Default)]
pub struct NodeGraph {
    /// The content of the dialog.
    pub children: ViewRef,

    /// Additional styles to be applied to the graph element.
    pub style: StyleHandle,
}

impl ViewTemplate for NodeGraph {
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        Element::<NodeBundle>::new()
            .named("NodeGraph")
            .with_styles((style_node_graph, self.style.clone()))
            .with_children(self.children.clone())
    }
}
