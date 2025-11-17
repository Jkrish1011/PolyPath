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
#[derive(Debug)]
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
        bridge_name: &str,
        metrics: EdgeMetrics,
        min_amount: Option<f64>,
        max_amount: Option<f64>
    ) -> Result<bool> {

        let edge = Arc::new(Edge::new(from, to, bridge_name.to_string(), metrics, min_amount, max_amount));

        // Adding outgoing edges (shard by source)
        let from_shard = &self.outgoing_edges[self.shard_index(from)];
        from_shard.entry(from).or_insert(Vec::new()).push(Arc::clone(&edge));

        // Adding incoming edges (shard by destination)

        let to_shard = &self.incoming_edges[self.shard_index(to)];
        to_shard.entry(to).or_insert(Vec::new()).push(Arc::clone(&edge));

        self.version.fetch_add(1, Ordering::Relaxed);

        Ok(true)
    }

    pub fn update_edge_metrics(
        &self,
        from: NodeId,
        to: NodeId,
        bridge_name: &str,
        metrics: EdgeMetrics,
    ) -> Result<bool> {
        let shard = &self.outgoing_edges[self.shard_index(from)];

        if let Some(edges) = shard.get(&from) {
            for edge in edges.value() {
                if edge.to == to && edge.bridge_name == bridge_name {
                    edge.metrics.update(metrics);
                    self.version.fetch_add(1, Ordering::Release);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    } 

    // Get all the outgoing edges from a given Node.
    pub fn get_outgoing_edges(&self, from: NodeId) -> Vec<Arc<Edge>> {
        let shard = &self.outgoing_edges[self.shard_index(from)];
        
        let res = shard.get(&from)
                                                        .map(|entry| entry
                                                                                                        .value().iter()
                                                                                                        .filter(|edge| edge.is_active())
                                                                                                        .map(|edge| Arc::clone(edge))
                                                                                                        .collect()
                                                        ).unwrap_or_default();
        res
    }

    pub fn get_incoming_edges(&self, to: NodeId) -> Vec<Arc<Edge>> {
        let shard = &self.incoming_edges[self.shard_index(to)];

        let res = shard.get(&to).map(|entry| entry.value().iter()
                                                                                                    .filter(|edge| edge.is_active())
                                                                                                    .map(|edge| Arc::clone(edge))
                                                                                                    .collect()
                                                                                                        ).unwrap_or_default();
        res
    }

    // Get neighbours with weights for pathfinding.
    pub fn neighbours(
        &self, 
        node_id: NodeId, 
        params: &RoutingParams
    ) -> Vec<(NodeId, f64)> {
        self.get_outgoing_edges(node_id)
            .into_iter()
            .map(|edge| {
                let metrics = edge.get_metrics();
                let weight = compute_edge_weight(&metrics, params);
                (edge.to, weight)
            })
            .collect()
    }

}

fn compute_edge_weight(
    metrics: &EdgeMetrics,
    params: &RoutingParams
) -> f64 {
    let cost_component = params.alpha * metrics.cost;
    let speed_component = params.beta * metrics.speed;
    let liquidity_component = params.gamma * metrics.liquidity;
    let risk_component = params.delta * metrics.risk;

    cost_component + speed_component + liquidity_component + risk_component
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn graph_creation() {
        let shard_count = 64;
        let graph = Graph::new(shard_count);

        let stargate_eth_node_id = graph.get_or_create_exchange_node("stargate", "ethereum");
        let stargate_pol_node_id = graph.get_or_create_exchange_node("stargate", "polygon");
        let stargate_arb_node_id = graph.get_or_create_exchange_node("stargate", "arbitrum");
        let stargate_base_node_id = graph.get_or_create_exchange_node("stargate", "base");

        let eth_usdc_node_id = graph.get_or_create_asset_node("ethereum", "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", "USDC");
        let pol_usdc_node_id = graph.get_or_create_asset_node("polygon", "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359", "USDC");
        let base_usdc_node_id = graph.get_or_create_asset_node("base", "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", "USDC");
        let mut edge_metrics = EdgeMetrics {
            cost: 1000.0,
            speed: 192.9,
            liquidity: 100.00,
            risk: 1.2
        };
        graph.add_edge(stargate_eth_node_id, eth_usdc_node_id, "stargate", edge_metrics.clone(), Some(100.0), Some(1000.0));
        graph.add_edge(stargate_pol_node_id, eth_usdc_node_id, "stargate", edge_metrics.clone(), Some(100.0), Some(1000.0));

        edge_metrics.cost = 1500.0;
        edge_metrics.risk = 2.2;
        let update_res = graph.update_edge_metrics(
            stargate_pol_node_id,
            eth_usdc_node_id,
            "stargate",
            edge_metrics.clone()
        ).unwrap();
        // println!("Updation result: {}", update_res);
        // println!("{:?}", graph);

        let outgoing_edge = graph.get_outgoing_edges(stargate_pol_node_id);
        println!("Outgoing Edge!");
        for edge in outgoing_edge {
            println!("{:?}", &edge);
        }

        let incoming_edge = graph.get_incoming_edges(stargate_pol_node_id);
        println!("Incoming Edge!"); // nothing to be found.
        for edge in incoming_edge {
            println!("{:?}", &edge);
        }

        let incoming_edge_2 = graph.get_incoming_edges(eth_usdc_node_id);
        println!("Incoming Edge!"); // nothing to be found.
        for edge in incoming_edge_2 {
            println!("{:?}", &edge);
        }
    }
}