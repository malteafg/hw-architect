use utils::id::{IdManager, TreeId};

use glam::Vec3;
use serde::{Deserialize, Serialize};

use rand::Rng;
use std::collections::{BTreeMap, HashMap};

/// Maybe temporary, but specifies the clickable radius of a tree.
const TREE_RADIUS: f32 = 2.0;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Tree {
    pos: Vec3,
    yrot: f32,
}

impl Tree {
    pub fn new(pos: Vec3) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            pos,
            yrot: rng.gen_range(0.0..3.14),
        }
    }

    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn yrot(&self) -> f32 {
        self.yrot
    }
}

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
    pub fn new() -> Self {
        Self::default()
    }

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
}

impl crate::TreeManipulator for Trees {
    fn add_tree(&mut self, tree: Tree, model_id: u128) {
        let tree_id = self.id_manager.gen();

        let Some(model_map) = self.tree_map.get_mut(&model_id) else {
            let mut new_model_map = HashMap::new();
            new_model_map.insert(tree_id, tree);
            self.tree_map.insert(model_id, new_model_map);
            return;
        };

        model_map.insert(tree_id, tree);
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

    fn get_trees(&self) -> &TreeMap {
        &self.tree_map
    }
}
