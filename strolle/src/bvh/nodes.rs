use std::{mem, ops};

use super::{BvhNode, BvhNodeId};

#[derive(Debug)]
pub struct BvhNodes {
    nodes: Vec<BvhNode>,
    free_nodes: Vec<BvhNodeId>,
}

impl BvhNodes {
    pub fn add(&mut self, node: BvhNode) -> BvhNodeId {
        if let Some(id) = self.free_nodes.pop() {
            let prev_node = mem::replace(&mut self[id], node);

            if let BvhNode::Internal {
                left_id, right_id, ..
            } = prev_node
            {
                self.free_nodes.push(left_id);
                self.free_nodes.push(right_id);
            }

            id
        } else {
            self.nodes.push(node);

            BvhNodeId::new((self.nodes.len() - 1) as u32)
        }
    }

    pub fn remove(&mut self, id: BvhNodeId) -> BvhNode {
        self.free_nodes.push(id);

        mem::take(&mut self[id])
    }

    pub fn remove_tree(&mut self, id: BvhNodeId) {
        self.free_nodes.push(id);
    }

    pub fn update_root(&mut self, node: BvhNode) -> Option<BvhNode> {
        if self.nodes.is_empty() {
            self.nodes.push(node);
            None
        } else {
            Some(mem::replace(&mut self[BvhNodeId::root()], node))
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for BvhNodes {
    fn default() -> Self {
        Self {
            nodes: vec![BvhNode::default()],
            free_nodes: Default::default(),
        }
    }
}

impl ops::Index<BvhNodeId> for BvhNodes {
    type Output = BvhNode;

    fn index(&self, index: BvhNodeId) -> &Self::Output {
        &self.nodes[index.get() as usize]
    }
}

impl ops::IndexMut<BvhNodeId> for BvhNodes {
    fn index_mut(&mut self, index: BvhNodeId) -> &mut Self::Output {
        &mut self.nodes[index.get() as usize]
    }
}
