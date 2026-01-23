use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Generated,
    Crossover,
    Mutated,
    Refined,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IdeaStatus {
    Active,
    Archived,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Facets {
    pub audience: String,
    pub jtbd: String,
    pub differentiator: String,
    pub monetization: String,
    pub distribution: String,
    pub risks: String,
}

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_serde() {
        let origin = Origin::Generated;
        let json = serde_json::to_string(&origin).unwrap();
        assert_eq!(json, "\"generated\"");

        let parsed: Origin = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Origin::Generated);
    }

    #[test]
    fn test_status_serde() {
        let status = IdeaStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");

        let parsed: IdeaStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, IdeaStatus::Active);
    }

    #[test]
    fn test_scores_default() {
        let scores = Scores::default();
        assert_eq!(scores.feasibility, 0.0);
        assert_eq!(scores.clarity, 0.0);
    }
}
