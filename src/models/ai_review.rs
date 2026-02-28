use serde::{Deserialize, Serialize};

/// Priority level for an AI recommendation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiPriority {
    High,
    Medium,
    Low,
}

impl AiPriority {
    pub fn label(&self) -> &'static str {
        match self {
            Self::High => "Haute priorité",
            Self::Medium => "Priorité moyenne",
            Self::Low => "Faible priorité",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::High => "#ff4e42",
            Self::Medium => "#ffa400",
            Self::Low => "#0cce6b",
        }
    }
}

/// A single AI-generated recommendation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiRecommendation {
    pub title: String,
    pub description: String,
    pub priority: AiPriority,
}

/// Full AI review result for a repository CI/CD pipeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiReview {
    pub summary: String,
    pub recommendations: Vec<AiRecommendation>,
}

/// State machine for the AI review lifecycle
#[derive(Debug, Clone, PartialEq)]
pub enum AiReviewState {
    /// Waiting for user to trigger the review
    Idle,
    /// API call in progress
    Loading,
    /// Successfully received and parsed the AI review
    Done(AiReview),
    /// No GitHub token was provided — feature unavailable
    Unavailable,
    /// An error occurred during the API call or parsing
    Error(String),
}
