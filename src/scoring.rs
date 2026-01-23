use crate::config::ScoringWeights;
use crate::data::Scores;

/// Calculate overall score using weighted sum.
#[allow(dead_code)]
/// Risk is inverted: (10 - risk) * weight
pub fn calculate_overall_score(scores: &Scores, weights: &ScoringWeights) -> f32 {
    let weighted_sum = scores.feasibility * weights.feasibility
        + scores.speed_to_value * weights.speed_to_value
        + scores.differentiation * weights.differentiation
        + scores.market_size * weights.market_size
        + scores.distribution * weights.distribution
        + scores.moats * weights.moats
        + (10.0 - scores.risk) * weights.risk  // Invert risk
        + scores.clarity * weights.clarity;

    let total_weight = weights.feasibility
        + weights.speed_to_value
        + weights.differentiation
        + weights.market_size
        + weights.distribution
        + weights.moats
        + weights.risk
        + weights.clarity;

    weighted_sum / total_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_overall_score_all_weights_one() {
        let scores = Scores {
            feasibility: 8.0,
            speed_to_value: 7.0,
            differentiation: 6.0,
            market_size: 9.0,
            distribution: 7.0,
            moats: 5.0,
            risk: 3.0, // Low risk = high contribution
            clarity: 8.0,
        };
        let weights = ScoringWeights::default();

        let overall = calculate_overall_score(&scores, &weights);

        // Expected: (8 + 7 + 6 + 9 + 7 + 5 + (10-3) + 8) / 8 = (8+7+6+9+7+5+7+8)/8 = 57/8 = 7.125
        assert!((overall - 7.125).abs() < 0.001);
    }

    #[test]
    fn test_calculate_overall_score_risk_inversion() {
        let low_risk_scores = Scores {
            feasibility: 5.0,
            speed_to_value: 5.0,
            differentiation: 5.0,
            market_size: 5.0,
            distribution: 5.0,
            moats: 5.0,
            risk: 2.0, // Low risk
            clarity: 5.0,
        };

        let high_risk_scores = Scores {
            feasibility: 5.0,
            speed_to_value: 5.0,
            differentiation: 5.0,
            market_size: 5.0,
            distribution: 5.0,
            moats: 5.0,
            risk: 8.0, // High risk
            clarity: 5.0,
        };

        let weights = ScoringWeights::default();

        let low_risk_overall = calculate_overall_score(&low_risk_scores, &weights);
        let high_risk_overall = calculate_overall_score(&high_risk_scores, &weights);

        assert!(
            low_risk_overall > high_risk_overall,
            "Low risk should yield higher overall score"
        );
    }
}
