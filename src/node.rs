//! Types associated to nodes and in particular node walking.

// Ported to the flat token layout in phase 3 of layout C. Not built until then.
// mod ancestors;
// mod siblings;
// mod skip_tokens;
mod children;
mod element;
pub(crate) mod node_impl;
mod token;
mod walk;
mod walk_events;

pub use self::children::Children;
pub(crate) use self::element::Id;
pub use self::element::{Element, NodeOrToken};
pub use self::node_impl::Node;
pub use self::token::Token;
pub use self::walk::{Walk, WithDepths};
pub use self::walk_events::{Event, WalkEvents};
