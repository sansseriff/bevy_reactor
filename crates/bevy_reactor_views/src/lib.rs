mod cond;
mod effect;
mod element;
mod style;
mod text;
mod view;
mod view_template;

pub mod prelude {
    pub use crate::element::Element;
    pub use crate::view::View;
    pub use crate::view_template::ViewTemplate;
    pub use crate::ReactorViewsPlugin;
}

use bevy::{
    app::{App, Plugin, Update},
    prelude::{Added, Entity, IntoSystemConfigs, World},
};
use bevy_mod_stylebuilder::StyleBuilderPlugin;
use bevy_reactor_signals::{ReactionSet, SignalsPlugin};
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
    // Need to copy query result to avoid double-borrow of world.
    let mut roots = world.query_filtered::<(Entity, &ViewRoot, &mut ViewCell), Added<ViewRoot>>();
    let roots_copy: Vec<Entity> = roots.iter(world).map(|(e, _, _)| e).collect();
    for root_entity in roots_copy.iter() {
        let Ok((_, _, cell)) = roots.get(world, *root_entity) else {
            continue;
        };
        let inner = cell.0.clone();
        inner.lock().unwrap().build(*root_entity, world);
    }
}

fn attach_child_views() {}
