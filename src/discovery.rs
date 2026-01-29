use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeAvailable {
    H4to8,
    H10to16,
    H20Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusinessModel {
    Saas,
    Api,
    OneTime,
    Marketplace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetAudience {
    Developers,
    Business,
    Creators,
    Freelancers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechApproach {
    LlmBased,
    LlmAssisted,
    NoLlm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryAnswers {
    pub skills: Vec<String>,
    pub time_available: TimeAvailable,
    pub business_model: BusinessModel,
    pub target_audience: TargetAudience,
    pub tech_approach: TechApproach,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedConstraints {
    pub timeline_weeks: u32,
    pub required_skills: Vec<String>,
    pub must_include: Vec<String>,
    pub forbidden: Vec<String>,
}

pub fn derive_constraints(answers: &DiscoveryAnswers) -> DerivedConstraints {
    let timeline_weeks = match answers.time_available {
        TimeAvailable::H4to8 => 1,
        TimeAvailable::H10to16 => 2,
        TimeAvailable::H20Plus => 4,
    };

    let required_skills = normalize_tokens(&answers.skills);

    let must_include = normalize_tokens(&[
        match answers.business_model {
            BusinessModel::Saas => "saas",
            BusinessModel::Api => "api",
            BusinessModel::OneTime => "one-time",
            BusinessModel::Marketplace => "marketplace",
        }
        .to_string(),
        match answers.target_audience {
            TargetAudience::Developers => "developers",
            TargetAudience::Business => "business",
            TargetAudience::Creators => "creators",
            TargetAudience::Freelancers => "freelancers",
        }
        .to_string(),
    ]);

    let forbidden = match answers.tech_approach {
        TechApproach::NoLlm => normalize_tokens(&["llm".to_string(), "ai".to_string()]),
        TechApproach::LlmBased | TechApproach::LlmAssisted => Vec::new(),
    };

    DerivedConstraints {
        timeline_weeks,
        required_skills,
        must_include,
        forbidden,
    }
}

fn normalize_tokens(tokens: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = tokens
        .iter()
        .map(|token| token.trim().to_lowercase())
        .filter(|token| !token.is_empty())
        .collect();
    normalized.sort();
    normalized.dedup();
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_mapping_basic() {
        let answers = DiscoveryAnswers {
            skills: vec!["Dev".to_string(), " design ".to_string()],
            time_available: TimeAvailable::H4to8,
            business_model: BusinessModel::Saas,
            target_audience: TargetAudience::Developers,
            tech_approach: TechApproach::LlmAssisted,
        };

        let derived = derive_constraints(&answers);
        assert_eq!(derived.timeline_weeks, 1);
        assert_eq!(
            derived.required_skills,
            vec!["design".to_string(), "dev".to_string()]
        );
        assert_eq!(
            derived.must_include,
            vec!["developers".to_string(), "saas".to_string()]
        );
        assert_eq!(derived.forbidden, Vec::<String>::new());
    }

    #[test]
    fn test_discovery_mapping_no_llm_sets_forbidden() {
        let answers = DiscoveryAnswers {
            skills: vec!["dev".to_string()],
            time_available: TimeAvailable::H20Plus,
            business_model: BusinessModel::Api,
            target_audience: TargetAudience::Business,
            tech_approach: TechApproach::NoLlm,
        };

        let derived = derive_constraints(&answers);
        assert_eq!(derived.timeline_weeks, 4);
        assert_eq!(derived.forbidden, vec!["ai".to_string(), "llm".to_string()]);
    }
}
