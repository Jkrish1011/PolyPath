use crate::graph::Graph;
use crate::types::*;
use core::f64;
use std::{
    sync::Arc,
    cmp::Ordering,
    collections::{
        BinaryHeap, HashMap, HashSet
    }
};

pub struct RoutingEngine {
    graph: Arc<Graph>,
    max_hops: usize,
}

#[derive(Clone, PartialEq)]
struct State {
    node: NodeId,
    g_score: f64, // Cost from start
    f_score: f64, // Estimated total cost
    hops: usize
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.partial_cmp(&self.f_score).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RoutingEngine {
    pub fn new(graph: Arc<Graph>, max_hops: usize) -> Self {
        Self {
            graph,
            max_hops
        }
    }

    // Using A* algorithm
    pub fn find_path(
        &self, 
        start: NodeId,
        end: NodeId,
        params: &RoutingParams
    ) -> Option<Path> {

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<NodeId, (NodeId, Arc<Edge>)> = HashMap::new();
        let mut g_score: HashMap<NodeId, f64> = HashMap::new();
        let mut visited = HashSet::new();

        g_score.insert(start, 0.0);
        open_set.push(State {
            node: start,
            g_score: 0.0,
            f_score: 0.0,
            hops: 0,
        });

        while let Some(current) = open_set.pop() {
            if current.node == end {
                return Some(self.reconstruct_path(start, end, &came_from));
            }

            if visited.contains(&current.node) || current.hops >= self.max_hops {
                continue;
            }

            visited.insert(current.node);

            let neighbours = self.graph.neighbours(current.node, params);

            for (neighbor, edge_weight) in neighbours {
                if visited.contains(&neighbor) {
                    continue;
                }

                let tentative_g = current.g_score + edge_weight;

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&f64::INFINITY) {
                    // get actual edge for reconstruction
                    let edges = self.graph.get_outgoing_edges(current.node);
                    let edge = edges.iter().find(|e| e.to == neighbor)?;

                    came_from.insert(neighbor, (current.node, Arc::clone(edge)));
                    g_score.insert(neighbor, tentative_g);

                    let h_score = self.heuristic(neighbor, end);
                    let f_score = tentative_g + h_score;

                    open_set.push(State {
                        node: neighbor,
                        g_score: tentative_g,
                        f_score,
                        hops: current.hops + 1
                    });
                }
            }

        }

        None
    }

    pub fn reconstruct_path(
        &self, 
        start: NodeId,
        end: NodeId,
        came_from: &HashMap<NodeId, (NodeId, Arc<Edge>)>
    ) -> Path {
        let mut hops = Vec::new();
        let mut current = end;
        let mut total_cost = 0.0;
        let mut total_time = 0.0;
        let mut total_risk = 0.0;
        let mut min_liquidity = f64::INFINITY;

        while current != start {
            if let Some((from, edge)) = came_from.get(&current) {
                let metrics = edge.get_metrics();
                hops.push(Hop {
                    from: *from,
                    to: current,
                    bridge_name: edge.bridge_name.clone(),
                    metrics: metrics.clone()
                });
                total_cost += metrics.cost;
                total_time += metrics.speed;
                total_risk += metrics.risk;
                min_liquidity = min_liquidity.min(metrics.liquidity);
                current = *from;
            }
            else {
                break;
            }
        } 
        hops.reverse();

        Path {
            hops, 
            total_cost,
            total_time,
            total_risk,
            min_liquidity,
            aggregate_score: 0.0 // Will be computed later by scoring algorithm
        }
    }

    fn heuristic(&self, from: NodeId, to: NodeId) -> f64 {
        // 0.0 for now. Can enable chain-based heuristic. 
        // Learn about chain-based heuristics
        // this algorithm with 0.0 will behave like Dijisktra
        0.0
    }
}

