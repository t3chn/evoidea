use serde::{Deserialize, Serialize};

/// Scoring weights for overall score calculation
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub feasibility: f32,
    pub speed_to_value: f32,
    pub differentiation: f32,
    pub market_size: f32,
    pub distribution: f32,
    pub moats: f32,
    pub risk: f32,
    pub clarity: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            feasibility: 1.0,
            speed_to_value: 1.0,
            differentiation: 1.0,
            market_size: 1.0,
            distribution: 1.0,
            moats: 1.0,
            risk: 1.0,
            clarity: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_weights_all_one() {
        let weights = ScoringWeights::default();
        assert_eq!(weights.feasibility, 1.0);
        assert_eq!(weights.speed_to_value, 1.0);
        assert_eq!(weights.differentiation, 1.0);
        assert_eq!(weights.market_size, 1.0);
        assert_eq!(weights.distribution, 1.0);
        assert_eq!(weights.moats, 1.0);
        assert_eq!(weights.risk, 1.0);
        assert_eq!(weights.clarity, 1.0);
    }
}
