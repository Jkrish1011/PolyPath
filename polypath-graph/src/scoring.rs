use crate::types::*;

#[derive(Debug, Clone)]
pub struct NormalizedPath {
    path: Path,
    normalized: NormalizedMetrics
}

#[derive(Debug, Clone)]
pub struct NormalizedMetrics {
    cost: f64,
    speed: f64,
    risk: f64,
    liquidity: f64
}

// Score normalizer for 0-1 scaling
#[derive(Debug)]
pub struct ScoreNormalizer;


impl ScoreNormalizer {
    pub fn normalize_path(
        &self,
        paths: &[Path]
    ) -> Vec<NormalizedPath> {
        if paths.is_empty() {
            return Vec::new();
        }

        // Find min/max for each dimension
        let (min_cost, max_cost) = paths.iter()
                                    .map(|p| p.total_cost)
                                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v| {
                                        (min.min(v), max.max(v))
                                    });
        
        let (min_time, max_time) = paths.iter()
                                    .map(|p| p.total_time)
                                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v| {
                                        (min.min(v), max.max(v))
                                    });

        let (min_risk, max_risk) = paths.iter()
                                    .map(|p| p.total_risk)
                                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v| {
                                        (min.min(v), max.max(v))
                                    });
        
        let (min_liq, max_liq) = paths.iter()
                                    .map(|p| p.min_liquidity)
                                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v| {
                                        (min.min(v), max.max(v))
                                    });
        
        paths.iter().map(|path| {
            let cost_norm = if max_cost > min_cost {
                1.0 - (path.total_cost - min_cost) / (max_cost - min_cost)
            } else{
                1.0
            };

            let time_norm = if max_time > min_time {
                1.0 - (path.total_time - min_time) / (max_time - min_time)
            } else {
                1.0
            };

            let risk_norm = if max_risk > min_risk {
                1.0 - (path.total_risk - min_risk) / (max_risk - min_risk)
            } else {
                1.0
            };

            let liq_norm = if max_liq > min_liq {
                1.0 - (path.min_liquidity - min_liq) / (max_liq - min_liq)
            } else {
                1.0
            };

            NormalizedPath {
                path: path.clone(),
                normalized: NormalizedMetrics { 
                    cost: cost_norm, 
                    speed: time_norm, 
                    risk: risk_norm, 
                    liquidity: liq_norm 
                }
            }
        }).collect()
                        
    }
}


#[derive(Debug, Clone)]
pub struct ScoredPath {
    path: Path,
    score: f64
}


// Multi-objective optimizer. Needs update.
#[derive(Debug)]
pub struct Optimizer;

impl Optimizer {
    // total sum weighed and optimized
    pub fn weighed_sum(
        &self, 
        normalized: &[NormalizedPath],
        params: &RoutingParams,
    ) -> Vec<ScoredPath> {
        normalized.iter().map(|np| {
            let score = params.alpha * np.normalized.cost
                                + params.beta * np.normalized.speed
                                + params.gamma * np.normalized.liquidity
                                + params.delta * (1.0 - np.normalized.risk);

            ScoredPath {
                path: np.path.clone(),
                score,
            }
        }).collect()
    }

    // Aproximate calculations. need further updates
    pub fn pareto_front(
        &self,
        normalized: &[NormalizedPath],
        max_results: usize
    ) -> Vec<ScoredPath> {
        let mut candidates: Vec<&NormalizedPath> = normalized.iter().collect();

        candidates.retain(|candidate| {
            !normalized.iter().any(|other| {
                other.normalized.cost <= candidate.normalized.cost 
                    && other.normalized.speed <= candidate.normalized.speed
                    && other.normalized.risk <= candidate.normalized.risk
                    && other.normalized.liquidity >= candidate.normalized.liquidity 
                    && (other.normalized.cost < candidate.normalized.cost
                        || other.normalized.speed < candidate.normalized.speed
                        || other.normalized.risk < candidate.normalized.risk
                        || other.normalized.liquidity > candidate.normalized.liquidity)
            })
        });

        let mut scored: Vec<ScoredPath> = candidates.iter().map(|np| {
            ScoredPath {
                path: np.path.clone(),
                score: np.normalized.cost + np.normalized.speed + np.normalized.liquidity - np.normalized.risk,
            }
        }).collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored.truncate(max_results);
        scored

    }
}

#[derive(Debug)]
pub struct Ranker;

impl Ranker {
    pub fn rank(
        &self,
        scored: Vec<ScoredPath>,
        max_results: usize,
    ) -> Vec<RankedPath> {
        let mut ranked: Vec<RankedPath> = scored.into_iter().enumerate().map(|(idx, sp)| {
            let metrics = &sp.path;
            RankedPath {
                path: sp.path.clone(),
                rank: idx+ 1,
                score_breakdown: ScoreBreakDown { 
                    cost_score: metrics.total_cost, 
                    speed_score: metrics.total_time, 
                    liquidity_score: metrics.min_liquidity, 
                    risk_score: metrics.total_risk, 
                    final_score: sp.score
                }
            }
        }).collect();

        ranked.truncate(max_results);
        ranked
    }
}


// Complete scoring Engine
pub struct ScoringEngine {
    normalizer: ScoreNormalizer,
    optimizer: Optimizer,
    ranker: Ranker
}


impl ScoringEngine {
    pub fn new() -> Self {
        Self {
            normalizer: ScoreNormalizer,
            optimizer: Optimizer,
            ranker: Ranker
        }
    }

    pub fn score_and_rank(
        &self,
        paths: Vec<Path>,
        params: &RoutingParams,
        max_results: usize,
    ) -> Vec<RankedPath> {
        if paths.is_empty() {
            return Vec::new();
        }

        // Normalize
        let normalized = self.normalizer.normalize_path(&paths);

        // Optimize
        let score = if params.alpha + params.beta + params.gamma + params.delta == 1.0 {
            // Weighted Sum
            self.optimizer.weighed_sum(&normalized, params)
        } else{
            // Pareto front
            self.optimizer.pareto_front(&normalized, max_results)
        };

        self.ranker.rank(score, max_results)

    }
}