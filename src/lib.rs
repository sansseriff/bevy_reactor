//! A fine-grained reactive framework for Bevy.

#![warn(missing_docs)]

mod accessor;
mod callback;
mod cond;
mod cx;
mod element;
mod element_effect;
mod r#for;
mod for_each;
mod for_index;
mod fragment;
mod lcs;
mod mutable;
mod node_span;
mod plugin;
mod presenter;
mod reaction;
mod scope;
mod style;
mod text;
mod view;
mod view_tuple;

pub use cond::cond;
pub use cond::Cond;
pub use cx::Cx;
pub use cx::Rcx;
pub use cx::ReactiveContext;
pub use cx::ReactiveContextMut;
pub use cx::SetupContext;
pub use element::Element;
pub use for_each::ForEach;
pub use for_index::ForIndex;
pub use fragment::Fragment;
pub use mutable::Mutable;
pub use plugin::ReactorPlugin;
pub use presenter::*;
pub use r#for::For;
pub use reaction::*;
pub(crate) use scope::DespawnScopes;
pub(crate) use scope::TrackingScope;
pub use style::*;
pub use text::*;
pub use view::*;
pub use view_tuple::ViewTuple;
