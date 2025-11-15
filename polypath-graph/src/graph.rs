use crate::types::*;
use dashmap::DashMap;
use std::{
    collections::HashMap, hash::Hash, sync::{
        Arc, atomic::{
            AtomicU64, Ordering
        }
    }, time::SystemTime
};
use anyhow::Result;

// Main graph implementation
pub struct Graph {
    nodes: Arc<DashMap<NodeId, Arc<Node>>>,

    // Sharded edge storage for outgoing edges (key is source node)
    outgoing_edges: Vec<Arc<DashMap<NodeId, Vec<Arc<Edge>>>>>,

    // Sharded edge storage for incoming edges (key is destination node)
    incoming_edges: Vec<Arc<DashMap<NodeId, Vec<Arc<Edge>>>>>,

    // Shard count (power of 2, for efficient hashing)
    shard_count: usize,

    version: Arc<AtomicU64>,

    // Node ID Generator
    next_node_id: Arc<AtomicU64>,
}


impl Graph {

    // Create a graph with specific number of shards
    pub fn new(shard_count: usize) -> Self {
        assert!(shard_count > 0 && shard_count.is_power_of_two(), "shard count must be a power of 2");
        
        let mut outgoing:Vec<Arc<DashMap<NodeId, Vec<Arc<Edge>>>>> = Vec::with_capacity(shard_count);
        let mut incoming: Vec<Arc<DashMap<NodeId, Vec<Arc<Edge>>>>> = Vec::with_capacity(shard_count);

        for _ in 0..shard_count {
            outgoing.push(Arc::new(DashMap::new()));
            incoming.push(Arc::new(DashMap::new()));
        }

        Self {
            nodes: Arc::new(DashMap::new()),
            outgoing_edges: outgoing,
            incoming_edges: incoming,
            shard_count: shard_count,
            version: Arc::new(AtomicU64::new(0)),
            next_node_id: Arc::new(AtomicU64::new(1))
        }
    }

    #[inline]
    fn shard_index(&self, node_id: NodeId) -> usize {
        (node_id.0 as usize) & (self.shard_count - 1)
    }

    pub fn get_or_create_asset_node(
        &self, 
        chain: &str,
        token_address: &str,
        token_symbol: &str
    ) -> NodeId {
        let node_id = NodeId::from_parts(chain, token_address);
        
        // Check if node exists and return it. Otherwise, get and return it.
        if self.nodes.contains_key(&node_id) {
            return node_id;
        }

        let node = Arc::new( Node {
            id: node_id,
            node_type: NodeType::Asset { 
                chain: chain.to_string(), 
                token_address: token_address.to_string(), 
                token_symbol: token_symbol.to_string()
            },
            metadata: HashMap::new(),
            created_at: SystemTime::now()   
        });

        self.nodes.insert(node_id, node);
        node_id
    }

    pub fn get_or_create_exchange_node(
        &self, 
        name: &str, 
        chain: &str
    ) -> NodeId {
        let identifier = format!("{}:{}", name, chain);
        let node_id = NodeId::from_parts("exchange", &identifier);

        if self.nodes.contains_key(&node_id) {
            return node_id;
        }

        let node = Arc::new(Node {
            id: node_id,
            node_type: NodeType::Exchange { 
                name: name.to_string(), 
                chain: chain.to_string()
            },
            metadata: HashMap::new(),
            created_at: SystemTime::now()
        });

        self.nodes.insert(node_id, node);
        node_id
    }

    pub fn get_node(&self, node_id: NodeId) -> Option<Arc<Node>> {
        self.nodes.get(&node_id).map(|entry| Arc::clone(entry.value()))
    }

    pub fn add_edge(
        &self,
        from: NodeId,
        to: NodeId,
        bridge_name: String,
        metrics: EdgeMetrics,
        min_amount: Option<f64>,
        max_amount: Option<f64>
    ) -> Result<bool> {

        let edge = Arc::new(Edge::new(from, to, bridge_name, metrics, min_amount, max_amount));

        // Adding outgoing edges (shard by source)
        let from_shard = &self.outgoing_edges[self.shard_index(from)];
        from_shard.entry(from).or_insert(Vec::new()).push(Arc::clone(&edge));

        // Adding incoming edges (shard by destination)

        let to_shard = &self.incoming_edges[self.shard_index(to)];
        to_shard.entry(to).or_insert(Vec::new()).push(Arc::clone(&edge));

        self.version.fetch_add(1, Ordering::Relaxed);

        Ok(true)
    }

}


