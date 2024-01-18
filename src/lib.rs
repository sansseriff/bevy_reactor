//! A fine-grained reactive framework for Bevy.

#![warn(missing_docs)]

mod accessor;
mod bundle;
mod cx;
mod element;
mod mutable;
mod node_span;
mod plugin;
mod presenter;
mod reaction;
mod scope;
mod text;
mod view;
mod view_tuple;

pub use cx::Cx;
pub use cx::Re;
pub use cx::ReactiveContext;
pub use cx::ReactiveContextMut;
pub use element::Element;
pub use mutable::Mutable;
pub use plugin::ReactorPlugin;
pub use presenter::*;
pub use reaction::*;
pub use text::*;
pub use view::*;
