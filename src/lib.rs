//! A fine-grained reactive framework for Bevy.

#![warn(missing_docs)]

mod effect_target;
mod element;
mod fragment;
mod node_span;
mod parent_view;
mod plugin;
mod text;
mod view;
mod with_styles;

pub use effect_target::EffectTarget;
pub use effect_target::EntityEffect;
pub use element::Element;
pub use fragment::Fragment;
pub use node_span::NodeSpan;
pub use parent_view::ChildArray;
pub use parent_view::ChildView;
pub use parent_view::ChildViewTuple;
pub use parent_view::ParentView;
// pub use plugin::ReactorPlugin;
pub use text::*;
pub use view::*;
pub use with_styles::WithStyles;
