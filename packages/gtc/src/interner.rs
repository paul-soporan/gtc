//! Node interner storing full node record (key + payload).

use crate::core::NodeId;
use std::collections::HashMap;

/// Node record holds the user-provided key (label) and arbitrary payload.
#[derive(Clone, Debug)]
pub struct NodeRecord<K, D> {
    pub key: K,
    pub data: D,
}

impl<K, D> NodeRecord<K, D> {
    pub fn new(key: K, data: D) -> Self {
        Self { key, data }
    }
}

/// Simple HashMap-based interner storing NodeRecord<K,D>.
/// Keys must implement Eq + Hash + Clone; duplicates by key are collapsed.
#[derive(Clone)]
pub struct NodeInterner<K, D>
where
    K: Eq + std::hash::Hash + Clone,
{
    pub records: Vec<NodeRecord<K, D>>, // NodeId -> NodeRecord
    pub index: HashMap<K, NodeId>,      // key -> NodeId
}

impl<K, D> NodeInterner<K, D>
where
    K: Eq + std::hash::Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Intern key + data. If key already exists, returns existing NodeId (does not update data).
    pub fn intern(&mut self, key: K, data: D) -> NodeId {
        if let Some(&id) = self.index.get(&key) {
            return id;
        }
        let id = NodeId(self.records.len());
        self.records.push(NodeRecord::new(key.clone(), data));
        self.index.insert(key, id);
        id
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn get(&self, id: NodeId) -> &NodeRecord<K, D> {
        &self.records[id.0]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut NodeRecord<K, D> {
        &mut self.records[id.0]
    }

    pub fn get_id(&self, key: &K) -> Option<NodeId> {
        self.index.get(key).cloned()
    }

    /// Consume interner to parts (useful for conversions)
    pub fn into_parts(self) -> (Vec<NodeRecord<K, D>>, HashMap<K, NodeId>) {
        (self.records, self.index)
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &NodeRecord<K, D>)> {
        self.records.iter().enumerate().map(|(i, r)| (NodeId(i), r))
    }
}
