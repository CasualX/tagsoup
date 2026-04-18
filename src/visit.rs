
/// Visitor control flow for tree traversal.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum VisitControl {
	/// Descend into the children of the current node.
	#[default]
	Descend,
	/// Skip the children of the current node and continue with the next sibling.
	Continue,
	/// Stop visiting the tree entirely.
	Stop,
}
