use std::{
    sync::{
        Arc, 
        RwLock,
        atomic::{
            AtomicU64,
            AtomicBool,
            Ordering
        }
    },
    collections::{
        HashMap,
        hash_map::DefaultHasher
    },
    time::SystemTime,
    hash::{
        Hash,
        Hasher
    }
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn from_parts(chain: &str, identifier: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        chain.hash(&mut hasher);
        identifier.hash(&mut hasher);
        NodeId(hasher.finish())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Asset {
        chain: String, 
        token_address: String,
        token_symbol: String
    },
    Exchange {
        name: String,
        chain: String
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub metadata: HashMap<String, String>,
    pub created_at: SystemTime
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetrics {
    pub cost: f64,
    pub speed: f64,
    pub liquidity: f64,
    pub risk: f64
}

// Designed for lock free reads
#[derive(Debug)]
pub struct EdgeMetricsAtomic {
    // Store as fixed-point integers to avoid floating-point atomics
    // cost: f64 * 1e6 as u64
    cost: AtomicU64,
    // milliseconds as u64
    speed: AtomicU64,
    // scaled value (wei or scaled integer)
    liquidity: AtomicU64,
    // f64 * 1e6 as u64
    risk: AtomicU64,

    last_updated: AtomicU64
}


impl EdgeMetricsAtomic {
    pub fn new(metrics: EdgeMetrics) -> Self {
        Self {
            cost: AtomicU64::new((metrics.cost * 1_000_000.0) as u64),
            speed: AtomicU64::new((metrics.speed * 1_000.0) as u64),
            liquidity: AtomicU64::new((metrics.liquidity) as u64),
            risk: AtomicU64::new((metrics.risk * 1_000_000.0) as u64),
            last_updated: AtomicU64::new(SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
        }
    }

    pub fn read(&self) -> EdgeMetrics {
        EdgeMetrics { 
            cost: self.cost.load(Ordering::Acquire) as f64 / 1_000_000.0, 
            speed: self.speed.load(Ordering::Acquire) as f64 / 1_000.0, 
            liquidity: self.liquidity.load(Ordering::Acquire) as f64, 
            risk: self.risk.load(Ordering::Acquire) as f64 / 1_000_000.0, 
        }
    }

    pub fn update(&self, metrics: EdgeMetrics) -> bool {
        self.cost.store((metrics.cost * 1_000_000.0) as u64, Ordering::Release);
        self.speed.store((metrics.speed * 1_000.0) as u64, Ordering::Release);
        self.liquidity.store((metrics.liquidity) as u64, Ordering::Release);
        self.risk.store((metrics.risk * 1_000_000.0) as u64, Ordering::Release);
        self.last_updated.store(
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Release,
        );
        true
    }
}

#[derive(Debug)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub bridge_name: String,
    pub metrics: Arc<EdgeMetricsAtomic>,
    pub is_active: Arc<AtomicBool>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
}

impl Edge {
    pub fn new(
        from: NodeId,
        to: NodeId,
        bridge_name: String,
        metrics: EdgeMetrics,
        min_amount: Option<f64>,
        max_amount: Option<f64>
    ) -> Self {
        Self {
            from: from,
            to: to,
            bridge_name: bridge_name,
            metrics: Arc::new(EdgeMetricsAtomic::new(metrics)),
            is_active: Arc::new(AtomicBool::new(true)),
            min_amount: min_amount,
            max_amount: max_amount
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Acquire)
    }

    pub fn get_metrics(&self) -> EdgeMetrics {
        self.metrics.read()
    }
}