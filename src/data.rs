use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Generated,
    Crossover,
    Mutated,
    Refined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IdeaStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Facets {
    pub audience: String,
    pub jtbd: String,
    pub differentiator: String,
    pub monetization: String,
    pub distribution: String,
    pub risks: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scores {
    pub feasibility: f32,
    pub speed_to_value: f32,
    pub differentiation: f32,
    pub market_size: f32,
    pub distribution: f32,
    pub moats: f32,
    pub risk: f32,
    pub clarity: f32,
}

impl Default for Scores {
    fn default() -> Self {
        Self {
            feasibility: 0.0,
            speed_to_value: 0.0,
            differentiation: 0.0,
            market_size: 0.0,
            distribution: 0.0,
            moats: 0.0,
            risk: 0.0,
            clarity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Idea {
    pub id: Uuid,
    pub gen: u32,
    pub origin: Origin,
    pub parents: Vec<Uuid>,
    pub title: String,
    pub summary: String,
    pub facets: Facets,
    pub scores: Scores,
    pub overall_score: Option<f32>,
    pub judge_notes: Option<String>,
    pub status: IdeaStatus,
}

impl Idea {
    pub fn new(title: String, summary: String, facets: Facets, gen: u32, origin: Origin) -> Self {
        Self {
            id: Uuid::new_v4(),
            gen,
            origin,
            parents: Vec::new(),
            title,
            summary,
            facets,
            scores: Scores::default(),
            overall_score: None,
            judge_notes: None,
            status: IdeaStatus::Active,
        }
    }

    pub fn with_parents(mut self, parents: Vec<Uuid>) -> Self {
        self.parents = parents;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub run_id: Uuid,
    pub iteration: u32,
    pub ideas: Vec<Idea>,
    pub best_idea_id: Option<Uuid>,
    pub best_score: Option<f32>,
    pub stagnation_counter: u32,
}

impl State {
    pub fn new(run_id: Uuid) -> Self {
        Self {
            run_id,
            iteration: 0,
            ideas: Vec::new(),
            best_idea_id: None,
            best_score: None,
            stagnation_counter: 0,
        }
    }

    pub fn active_ideas(&self) -> impl Iterator<Item = &Idea> {
        self.ideas.iter().filter(|i| i.status == IdeaStatus::Active)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Generated,
    Scored,
    Selected,
    Crossover,
    Mutated,
    Refined,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub ts: DateTime<Utc>,
    pub iteration: u32,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub payload: serde_json::Value,
}

impl Event {
    pub fn new(iteration: u32, event_type: EventType, payload: serde_json::Value) -> Self {
        Self {
            ts: Utc::now(),
            iteration,
            event_type,
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerUp {
    pub idea_id: Uuid,
    pub title: String,
    pub overall_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalBest {
    pub idea_id: Uuid,
    pub title: String,
    pub summary: String,
    pub facets: Facets,
    pub scores: Scores,
    pub overall_score: f32,
    pub why_won: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalResult {
    pub run_id: Uuid,
    pub best: FinalBest,
    pub runners_up: Vec<RunnerUp>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idea_json_roundtrip() {
        let facets = Facets {
            audience: "developers".into(),
            jtbd: "automate testing".into(),
            differentiator: "AI-powered".into(),
            monetization: "SaaS subscription".into(),
            distribution: "developer communities".into(),
            risks: "competition from big tech".into(),
        };

        let idea = Idea::new(
            "Test Automation Tool".into(),
            "An AI-powered test automation tool".into(),
            facets,
            1,
            Origin::Generated,
        );

        let json = serde_json::to_string(&idea).unwrap();
        let parsed: Idea = serde_json::from_str(&json).unwrap();

        assert_eq!(idea.title, parsed.title);
        assert_eq!(idea.summary, parsed.summary);
        assert_eq!(idea.origin, parsed.origin);
        assert_eq!(idea.facets, parsed.facets);
    }

    #[test]
    fn test_state_json_roundtrip() {
        let run_id = Uuid::new_v4();
        let mut state = State::new(run_id);
        state.iteration = 3;
        state.stagnation_counter = 1;

        let json = serde_json::to_string(&state).unwrap();
        let parsed: State = serde_json::from_str(&json).unwrap();

        assert_eq!(state.run_id, parsed.run_id);
        assert_eq!(state.iteration, parsed.iteration);
        assert_eq!(state.stagnation_counter, parsed.stagnation_counter);
    }

    #[test]
    fn test_event_json_roundtrip() {
        let event = Event::new(1, EventType::Generated, serde_json::json!({"count": 5}));

        let json = serde_json::to_string(&event).unwrap();
        let parsed: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event.iteration, parsed.iteration);
    }

    #[test]
    fn test_final_result_json_roundtrip() {
        let facets = Facets {
            audience: "developers".into(),
            jtbd: "automate testing".into(),
            differentiator: "AI-powered".into(),
            monetization: "SaaS subscription".into(),
            distribution: "developer communities".into(),
            risks: "competition".into(),
        };

        let final_result = FinalResult {
            run_id: Uuid::new_v4(),
            best: FinalBest {
                idea_id: Uuid::new_v4(),
                title: "Winner".into(),
                summary: "The best idea".into(),
                facets,
                scores: Scores::default(),
                overall_score: 8.5,
                why_won: vec!["High feasibility".into(), "Large market".into()],
            },
            runners_up: vec![RunnerUp {
                idea_id: Uuid::new_v4(),
                title: "Second place".into(),
                overall_score: 7.5,
            }],
        };

        let json = serde_json::to_string(&final_result).unwrap();
        let parsed: FinalResult = serde_json::from_str(&json).unwrap();

        assert_eq!(final_result.best.title, parsed.best.title);
        assert_eq!(final_result.runners_up.len(), parsed.runners_up.len());
    }

    #[test]
    fn test_idea_invariants_generated_no_parents() {
        let facets = Facets {
            audience: "test".into(),
            jtbd: "test".into(),
            differentiator: "test".into(),
            monetization: "test".into(),
            distribution: "test".into(),
            risks: "test".into(),
        };

        let idea = Idea::new("Test".into(), "Test".into(), facets, 1, Origin::Generated);
        assert!(
            idea.parents.is_empty(),
            "Generated ideas should have no parents"
        );
    }

    #[test]
    fn test_idea_invariants_crossover_has_parents() {
        let facets = Facets {
            audience: "test".into(),
            jtbd: "test".into(),
            differentiator: "test".into(),
            monetization: "test".into(),
            distribution: "test".into(),
            risks: "test".into(),
        };

        let parent1 = Uuid::new_v4();
        let parent2 = Uuid::new_v4();
        let idea = Idea::new("Test".into(), "Test".into(), facets, 2, Origin::Crossover)
            .with_parents(vec![parent1, parent2]);

        assert!(
            !idea.parents.is_empty(),
            "Crossover ideas should have parents"
        );
        assert_eq!(idea.parents.len(), 2);
    }

    #[test]
    fn test_state_active_ideas_filter() {
        let run_id = Uuid::new_v4();
        let mut state = State::new(run_id);

        let facets = Facets {
            audience: "test".into(),
            jtbd: "test".into(),
            differentiator: "test".into(),
            monetization: "test".into(),
            distribution: "test".into(),
            risks: "test".into(),
        };

        let idea1 = Idea::new(
            "Active".into(),
            "Active".into(),
            facets.clone(),
            1,
            Origin::Generated,
        );
        let mut idea2 = Idea::new(
            "Archived".into(),
            "Archived".into(),
            facets,
            1,
            Origin::Generated,
        );
        idea2.status = IdeaStatus::Archived;

        state.ideas.push(idea1);
        state.ideas.push(idea2);

        let active: Vec<_> = state.active_ideas().collect();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].title, "Active");
    }
}
