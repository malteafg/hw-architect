use world_api::Tree;

use utils::id::{IdManager, TreeId};

use glam::Vec3;
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap};

/// Maybe temporary, but specifies the clickable radius of a tree.
const TREE_RADIUS: f32 = 2.0;

/// The u128 represents a hash of a tree model. For now it is not used as there is only one tree
/// model.
pub type TreeMap = BTreeMap<u128, HashMap<TreeId, Tree>>;

#[derive(Serialize, Deserialize)]
pub struct Trees {
    tree_map: TreeMap,
    id_manager: IdManager<TreeId>,
}

impl Default for Trees {
    fn default() -> Self {
        Self {
            tree_map: BTreeMap::new(),
            id_manager: IdManager::new(),
        }
    }
}

impl Trees {
    pub fn get_tree_from_pos(&self, pos: Vec3) -> Option<TreeId> {
        for model_map in self.tree_map.values() {
            for (id, tree) in model_map.iter() {
                if (tree.pos() - pos).length() < TREE_RADIUS {
                    return Some(*id);
                }
            }
        }
        None
    }

    fn get_tree(&self, id: &TreeId) -> &Tree {
        for model_map in self.tree_map.values() {
            if let Some(tree) = model_map.get(id) {
                return tree;
            }
        }
        panic!("treeid should be in tree map");
    }
}

impl crate::TreeManipulator for Trees {
    fn add_tree(&mut self, tree: Tree, model_id: u128) -> TreeId {
        let tree_id = self.id_manager.gen();

        match self.tree_map.get_mut(&model_id) {
            Some(model_map) => {
                model_map.insert(tree_id, tree);
                ()
            }
            None => {
                let mut new_model_map = HashMap::new();
                new_model_map.insert(tree_id, tree);
                self.tree_map.insert(model_id, new_model_map);
            }
        };
        return tree_id;
    }

    fn remove_tree(&mut self, tree_id: TreeId) -> u128 {
        for (model_id, model_map) in self.tree_map.iter_mut() {
            if model_map.remove(&tree_id).is_some() {
                return *model_id;
            }
        }

        // This should not be able to happen.
        dbg!(tree_id);
        panic!("Tree id did not exists in tree map, when bulldozing");
    }

    // fn get_trees(&self) -> &TreeMap {
    //     &self.tree_map
    // }

    fn get_tree_pos(&self, id: TreeId) -> Vec3 {
        self.get_tree(&id).pos()
    }
}
