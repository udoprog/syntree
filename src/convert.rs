use crate::builder::walk::Dir;
use crate::builder::TreeBuilder;
use crate::tree;

/// Construct a tree from a builder.
pub(crate) fn builder_to_tree<T>(b: &TreeBuilder<T>) -> tree::Tree<T>
where
    T: Clone,
{
    let mut tree = Vec::<tree::Links<T>>::new();
    let mut last = None;

    for step in b.walk() {
        match step {
            Dir::Next(next) => {
                if next.parent.is_none() {
                    // The last top-level item in the tree.
                    last = Some(next.id);
                }

                if next.links.next.is_none() {
                    if let Some(parent) = next.parent.and_then(|id| tree.get_mut(id.get())) {
                        parent.last = Some(next.id);
                    }
                }

                if let Some(node) = next.sibling.and_then(|id| tree.get_mut(id.get())) {
                    node.next = Some(next.id);
                }

                tree.push(tree::Links {
                    data: next.links.data.clone(),
                    kind: next.kind,
                    span: next.span,
                    prev: next.sibling,
                    next: None,
                    first: next.first,
                    last: None,
                });
            }
            Dir::Up(step) => {
                if let Some(parent) = step.parent.and_then(|id| tree.get_mut(id.get())) {
                    parent.span.end = step.cursor;
                }
            }
        }
    }

    tree::Tree::new(tree.into(), last)
}
