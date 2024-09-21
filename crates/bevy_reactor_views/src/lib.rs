mod cond;
mod effect;
mod element;
mod style;
mod text;
mod view;
mod view_template;

pub mod prelude {
    pub use crate::{
        Cond, Element, IntoView, ReactorViewsPlugin, TextComputed, TextStatic, View, ViewTemplate,
    };
}

use bevy::{
    app::{App, Plugin, Update},
    prelude::{Added, BuildChildren, Entity, IntoSystemConfigs, World},
    ui::GhostNode,
};
use bevy_mod_stylebuilder::StyleBuilderPlugin;
use bevy_reactor_signals::{ReactionSet, SignalsPlugin, TrackingScope};
pub use cond::Cond;
pub use element::Element;
pub use text::{TextComputed, TextStatic};
use view::ViewCell;
pub use view::ViewRoot;
pub use view::{IntoView, IntoViewVec, View};
pub use view_template::ViewTemplate;

pub struct ReactorViewsPlugin;

impl Plugin for ReactorViewsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SignalsPlugin)
            .add_plugins(StyleBuilderPlugin)
            .add_systems(
                Update,
                (
                    // run_view_reactions.in_set(ReactionSet),
                    build_added_view_roots.before(ReactionSet),
                    attach_child_views.after(ReactionSet),
                    // update_text_styles.after(attach_child_views),
                ),
            );
    }
}

pub(crate) fn build_added_view_roots(world: &mut World) {
    let tick = world.change_tick();
    // Need to copy query result to avoid double-borrow of world.
    let mut roots = world.query_filtered::<(Entity, &mut ViewCell), Added<ViewRoot>>();
    let roots_copy: Vec<Entity> = roots.iter(world).map(|(e, _)| e).collect();
    for root_entity in roots_copy.iter() {
        let Ok((_, cell)) = roots.get(world, *root_entity) else {
            continue;
        };
        let mut scope = TrackingScope::new(tick);
        let inner = cell.0.clone();
        let mut children = Vec::new();
        inner
            .lock()
            .unwrap()
            .build(*root_entity, world, &mut scope, &mut children);
        world
            .entity_mut(*root_entity)
            .insert((scope, GhostNode))
            .replace_children(&children);
    }
}

fn attach_child_views() {}
