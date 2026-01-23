use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub run_id: Uuid,
    pub mode: String,
    pub prompt: String,
    pub language: String,
    pub max_rounds: u32,
    pub population_size: u32,
    pub elite_count: u32,
    pub mutation_count: u32,
    pub crossover_count: u32,
    pub wildcard_count: u32,
    pub stagnation_patience: u32,
    pub score_threshold: f32,
    pub search_enabled: bool,
    pub scoring_weights: ScoringWeights,
    pub output_dir: String,
}

impl RunConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        prompt: String,
        mode: String,
        max_rounds: u32,
        population_size: u32,
        elite_count: u32,
        score_threshold: f32,
        stagnation_patience: u32,
        output_dir: String,
    ) -> Self {
        Self {
            run_id: Uuid::new_v4(),
            mode,
            prompt,
            language: "en".into(),
            max_rounds,
            population_size,
            elite_count,
            mutation_count: 4,
            crossover_count: 4,
            wildcard_count: 1,
            stagnation_patience,
            score_threshold,
            search_enabled: false,
            scoring_weights: ScoringWeights::default(),
            output_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_json_roundtrip() {
        let config = RunConfig::new(
            "Generate startup ideas".into(),
            "mock".into(),
            6,
            12,
            4,
            8.7,
            2,
            "runs".into(),
        );

        let json = serde_json::to_string(&config).unwrap();
        let parsed: RunConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.prompt, parsed.prompt);
        assert_eq!(config.max_rounds, parsed.max_rounds);
        assert_eq!(config.run_id, parsed.run_id);
    }

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
