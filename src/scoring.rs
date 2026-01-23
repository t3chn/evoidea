use crate::config::ScoringWeights;
use crate::data::{Idea, IdeaStatus, Scores};
use rand::seq::SliceRandom;

/// Calculate overall score using weighted sum.
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

/// Select ideas: elite (top by score) + diversity (random from mid-rank 30%-70%)
pub fn select_ideas(
    ideas: &mut [Idea],
    elite_count: usize,
    population_size: usize,
) -> Vec<uuid::Uuid> {
    // Filter to active ideas only
    let mut active_ideas: Vec<&mut Idea> = ideas
        .iter_mut()
        .filter(|i| i.status == IdeaStatus::Active && i.overall_score.is_some())
        .collect();

    if active_ideas.is_empty() {
        return Vec::new();
    }

    // Sort by overall_score descending
    active_ideas.sort_by(|a, b| {
        b.overall_score
            .partial_cmp(&a.overall_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut selected_ids = Vec::new();

    // Select elite
    let elite_to_take = elite_count.min(active_ideas.len());
    for idea in active_ideas.iter().take(elite_to_take) {
        selected_ids.push(idea.id);
    }

    // Calculate diversity slots
    let diversity_slots = population_size.saturating_sub(elite_to_take);

    if diversity_slots > 0 && active_ideas.len() > elite_to_take {
        // Mid-rank: 30%-70% of the sorted list
        let start_idx = (active_ideas.len() as f32 * 0.3).ceil() as usize;
        let end_idx = (active_ideas.len() as f32 * 0.7).floor() as usize;

        if start_idx < end_idx && start_idx < active_ideas.len() {
            let mid_rank: Vec<_> = active_ideas[start_idx..end_idx.min(active_ideas.len())]
                .iter()
                .filter(|i| !selected_ids.contains(&i.id))
                .collect();

            let mut rng = rand::thread_rng();
            let diversity_to_take = diversity_slots.min(mid_rank.len());

            // Random selection from mid-rank
            let indices: Vec<usize> = (0..mid_rank.len()).collect();
            let selected_indices: Vec<_> = indices
                .choose_multiple(&mut rng, diversity_to_take)
                .cloned()
                .collect();

            for idx in selected_indices {
                selected_ids.push(mid_rank[idx].id);
            }
        }
    }

    selected_ids
}

/// Check if threshold stop condition is met
pub fn check_threshold_stop(best_score: Option<f32>, threshold: f32) -> bool {
    best_score.is_some_and(|score| score >= threshold)
}

/// Check if stagnation stop condition is met
pub fn check_stagnation_stop(stagnation_counter: u32, patience: u32) -> bool {
    stagnation_counter >= patience
}

/// Check if max rounds stop condition is met
pub fn check_max_rounds_stop(iteration: u32, max_rounds: u32) -> bool {
    iteration >= max_rounds
}

/// Update stagnation counter based on score improvement
pub fn update_stagnation(
    current_best: Option<f32>,
    previous_best: Option<f32>,
    stagnation_counter: u32,
) -> u32 {
    match (current_best, previous_best) {
        (Some(current), Some(previous)) if current > previous => 0,
        (Some(_), None) => 0,
        _ => stagnation_counter + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Facets, Origin};

    fn make_test_idea(title: &str, score: f32) -> Idea {
        let facets = Facets {
            audience: "test".into(),
            jtbd: "test".into(),
            differentiator: "test".into(),
            monetization: "test".into(),
            distribution: "test".into(),
            risks: "test".into(),
        };
        let mut idea = Idea::new(title.into(), "summary".into(), facets, 1, Origin::Generated);
        idea.overall_score = Some(score);
        idea
    }

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

    #[test]
    fn test_select_ideas_elite_preserved() {
        let mut ideas = vec![
            make_test_idea("Best", 9.0),
            make_test_idea("Second", 8.0),
            make_test_idea("Third", 7.0),
            make_test_idea("Fourth", 6.0),
            make_test_idea("Fifth", 5.0),
        ];

        let selected = select_ideas(&mut ideas, 2, 4);

        // Top 2 should be selected as elite
        assert!(selected.contains(&ideas[0].id));
        assert!(selected.contains(&ideas[1].id));
    }

    #[test]
    fn test_select_ideas_population_bounded() {
        let mut ideas: Vec<_> = (0..20)
            .map(|i| make_test_idea(&format!("Idea {}", i), 10.0 - i as f32 * 0.5))
            .collect();

        let selected = select_ideas(&mut ideas, 4, 8);

        assert!(
            selected.len() <= 8,
            "Selection should be bounded by population_size"
        );
    }

    #[test]
    fn test_select_ideas_diversity_not_top_only() {
        let mut ideas: Vec<_> = (0..10)
            .map(|i| make_test_idea(&format!("Idea {}", i), 10.0 - i as f32))
            .collect();

        // Run selection multiple times to check diversity comes from mid-rank
        let mut found_mid_rank = false;
        for _ in 0..10 {
            let selected = select_ideas(&mut ideas, 2, 5);
            // Check if any selected idea is from mid-rank (indices 3-6 roughly)
            for id in &selected {
                for (idx, idea) in ideas.iter().enumerate() {
                    if idea.id == *id && idx >= 3 && idx <= 6 {
                        found_mid_rank = true;
                    }
                }
            }
        }

        assert!(
            found_mid_rank,
            "Diversity selection should pick from mid-rank"
        );
    }

    #[test]
    fn test_threshold_stop_met() {
        assert!(check_threshold_stop(Some(9.0), 8.7));
        assert!(check_threshold_stop(Some(8.7), 8.7));
    }

    #[test]
    fn test_threshold_stop_not_met() {
        assert!(!check_threshold_stop(Some(8.0), 8.7));
        assert!(!check_threshold_stop(None, 8.7));
    }

    #[test]
    fn test_stagnation_stop() {
        assert!(check_stagnation_stop(2, 2));
        assert!(check_stagnation_stop(3, 2));
        assert!(!check_stagnation_stop(1, 2));
    }

    #[test]
    fn test_max_rounds_stop() {
        assert!(check_max_rounds_stop(6, 6));
        assert!(check_max_rounds_stop(7, 6));
        assert!(!check_max_rounds_stop(5, 6));
    }

    #[test]
    fn test_update_stagnation_improvement() {
        assert_eq!(update_stagnation(Some(8.0), Some(7.0), 2), 0);
    }

    #[test]
    fn test_update_stagnation_no_improvement() {
        assert_eq!(update_stagnation(Some(7.0), Some(7.0), 2), 3);
        assert_eq!(update_stagnation(Some(6.0), Some(7.0), 2), 3);
    }

    #[test]
    fn test_update_stagnation_first_score() {
        assert_eq!(update_stagnation(Some(7.0), None, 0), 0);
    }
}
