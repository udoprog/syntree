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
        if step.parent.is_none() {
            // The last top-level item in the tree.
            last = Some(step.id);
        }

        if let Some(parent) = step.parent.and_then(|id| tree.get_mut(id.get())) {
            parent.last = Some(step.id);
        }

        if let Some(node) = step.sibling.and_then(|id| tree.get_mut(id.get())) {
            node.next = Some(step.id);
        }

        tree.push(tree::Links {
            data: step.links.data.clone(),
            kind: step.links.kind,
            prev: step.sibling,
            next: None,
            first: step.first,
            last: None,
        });
    }

    tree::Tree::new(tree.into(), last)
}
