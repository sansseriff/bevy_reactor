//! A fine-grained reactive framework for Bevy.

#![warn(missing_docs)]

// mod compositor;
mod cond;
mod dynamic;
mod dynamic_keyed;
mod effect_target;
mod element;
mod r#for;
mod for_each;
mod for_index;
mod fragment;
mod hover;
mod lcs;
mod node_span;
mod parent_view;
mod plugin;
mod portal;
// mod style;
mod text;
mod view;
mod with_styles;

// pub use compositor::Compositor;
pub use cond::Cond;
pub use dynamic::Dynamic;
pub use dynamic_keyed::DynamicKeyed;
pub use effect_target::EffectTarget;
pub use effect_target::EntityEffect;
pub use element::Element;
pub use for_each::ForEach;
pub use for_index::ForIndex;
pub use fragment::Fragment;
pub use hover::CreateHoverSignal;
pub use node_span::NodeSpan;
pub use parent_view::ChildArray;
pub use parent_view::ChildView;
pub use parent_view::ChildViewTuple;
pub use parent_view::ParentView;
pub use plugin::ReactorPlugin;
pub use portal::Portal;
pub use r#for::For;
pub use text::*;
pub use view::*;
pub use with_styles::WithStyles;
// pub use style::StyleBuilderTextureAtlas;
