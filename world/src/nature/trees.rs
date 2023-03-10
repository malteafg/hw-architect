use utils::id::{IdManager, TreeId};

use glam::Vec3;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Tree {
    pos: Vec3,
    y_rot: f32,
}

#[derive(Serialize, Deserialize)]
pub struct TreeMap {
    tree_map: HashMap<TreeId, Vec<Tree>>,
    // id_manager: IdManager<TreeId>,
}

impl Default for TreeMap {
    fn default() -> Self {
        Self {
            tree_map: HashMap::new(),
            // id_manager: IdManager::new(),
        }
    }
}

impl TreeMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_tree(&mut self, tree: Tree, id: TreeId) {
        if !self.tree_map.contains_key(&id) {
            self.tree_map.insert(id, Vec::new());
        }

        let Some(tree_vec) = self.tree_map.get_mut(&id) else {
            // Maybe see hashmap .try_insert() to get rid of this?
            unreachable!()
        };

        tree_vec.push(tree);
    }

    pub fn remove_tree(&mut self, pos: Vec3) {
        // TODO find nearest tree and remove it?
        // Take in an area range or circle range?
    }
}
