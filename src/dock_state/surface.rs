use std::ops::{Index, IndexMut};

use crate::{Node, NodeIndex, Tree, WindowState};

/// A [`Surface`] is the highest level component in a [`DockState`](crate::DockState). [`Surface`]s represent an area
/// in which nodes are placed.
///
/// Typically, you're only using one surface, which is the main surface. However, if you drag
/// a tab out in a way which creates a window, you also create a new surface in which nodes can appear.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Surface<Tab> {
    /// An empty surface, with nothing inside (practically, a null surface).
    Empty,

    /// The main surface of a [`DockState`](crate::DockState), only one should exist at surface index 0 at any one time.
    Main(Tree<Tab>),

    /// A windowed surface with a state.
    Window(Tree<Tab>, WindowState),
}

impl<Tab> Index<NodeIndex> for Surface<Tab> {
    type Output = Node<Tab>;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        match self {
            Surface::Empty => panic!("indexed on empty surface"),
            Surface::Main(tree) | Surface::Window(tree, _) => &tree[index],
        }
    }
}
impl<Tab> IndexMut<NodeIndex> for Surface<Tab> {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        match self {
            Surface::Empty => panic!("indexed on empty surface"),
            Surface::Main(tree) | Surface::Window(tree, _) => &mut tree[index],
        }
    }
}

impl<Tab> Surface<Tab> {
    /// Is this surface [`Empty`](Self::Empty) (in practice null)?
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Get access to the node tree of this surface.
    pub fn node_tree(&self) -> Option<&Tree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Main(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }

    /// Get mutable access to the node tree of this surface.
    pub fn node_tree_mut(&mut self) -> Option<&mut Tree<Tab>> {
        match self {
            Surface::Empty => None,
            Surface::Main(tree) => Some(tree),
            Surface::Window(tree, _) => Some(tree),
        }
    }

    /// Returns an [`Iterator`] of nodes in this surface's tree.
    ///
    /// If the surface is [`Empty`](Self::Empty), then the returned [`Iterator`] will be empty.
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node<Tab>> {
        match self.node_tree() {
            Some(tree) => tree.iter(),
            None => core::slice::Iter::default(),
        }
    }

    /// Returns a mutable [`Iterator`] of nodes in this surface's tree.
    ///
    /// If the surface is [`Empty`](Self::Empty), then the returned [`Iterator`] will be empty.
    pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item = &mut Node<Tab>> {
        match self.node_tree_mut() {
            Some(tree) => tree.iter_mut(),
            None => core::slice::IterMut::default(),
        }
    }

    /// Returns an [`Iterator`] of **all** tabs in this surface's tree,
    /// and indices of containing nodes.
    pub fn iter_all_tabs(&self) -> impl Iterator<Item = (NodeIndex, &Tab)> {
        self.iter_nodes()
            .enumerate()
            .flat_map(|(index, node)| node.iter_tabs().map(move |tab| (NodeIndex(index), tab)))
    }

    /// Returns a mutable [`Iterator`] of **all** tabs in this surface's tree,
    /// and indices of containing nodes.
    pub fn iter_all_tabs_mut(&mut self) -> impl Iterator<Item = (NodeIndex, &mut Tab)> {
        self.iter_nodes_mut()
            .enumerate()
            .flat_map(|(index, node)| node.iter_tabs_mut().map(move |tab| (NodeIndex(index), tab)))
    }

    /// Returns a new [`Surface`] while mapping and filtering the tab type.
    /// Any remaining empty [`Node`]s and are removed, and if this [`Surface`] remains empty,
    /// it'll change to [`Surface::Empty`].
    pub fn filter_map_tabs<F, NewTab>(&self, function: F) -> Surface<NewTab>
    where
        F: FnMut(&Tab) -> Option<NewTab>,
    {
        match self {
            Surface::Empty => Surface::Empty,
            Surface::Main(tree) => Surface::Main(tree.filter_map_tabs(function)),
            Surface::Window(tree, window_state) => {
                let tree = tree.filter_map_tabs(function);
                if tree.is_empty() {
                    Surface::Empty
                } else {
                    Surface::Window(tree, window_state.clone())
                }
            }
        }
    }

    /// Returns a new [`Surface`] while mapping the tab type.
    pub fn map_tabs<F, NewTab>(&self, mut function: F) -> Surface<NewTab>
    where
        F: FnMut(&Tab) -> NewTab,
    {
        self.filter_map_tabs(move |tab| Some(function(tab)))
    }

    /// Returns a new [`Surface`] while filtering the tab type.
    /// Any remaining empty [`Node`]s and are removed, and if this [`Surface`] remains empty,
    /// it'll change to [`Surface::Empty`].
    pub fn filter_tabs<F>(&self, mut predicate: F) -> Surface<Tab>
    where
        F: FnMut(&Tab) -> bool,
        Tab: Clone,
    {
        self.filter_map_tabs(move |tab| predicate(tab).then(|| tab.clone()))
    }

    /// Removes all tabs for which `predicate` returns `false`.
    /// Any remaining empty [`Node`]s and are also removed, and if this [`Surface`] remains empty,
    /// it'll change to [`Surface::Empty`].
    pub fn retain_tabs<F>(&mut self, predicate: F)
    where
        F: FnMut(&mut Tab) -> bool,
    {
        if let Surface::Main(tree) | Surface::Window(tree, _) = self {
            tree.retain_tabs(predicate);
            if tree.is_empty() {
                *self = Surface::Empty;
            }
        }
    }
}
