use serde::{Deserialize, Serialize};

/// Status of a single check after evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CheckStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

/// Category grouping checks by difficulty level
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CheckCategory {
    Fundamentals,
    Intermediate,
    Advanced,
    Bonus,
}

impl CheckCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fundamentals => "Fondamentaux",
            Self::Intermediate => "Intermédiaire",
            Self::Advanced => "Avancé",
            Self::Bonus => "Bonus",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Fundamentals => "🟢",
            Self::Intermediate => "🔵",
            Self::Advanced => "🟡",
            Self::Bonus => "⭐",
        }
    }

    pub fn max_points(&self) -> u32 {
        match self {
            Self::Fundamentals => 50,
            Self::Intermediate => 30,
            Self::Advanced => 35,
            Self::Bonus => 10,
        }
    }
}

/// Definition of a check to perform
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CheckCategory,
    pub max_points: u32,
}

/// Result of running a check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    pub check: Check,
    pub status: CheckStatus,
    pub points_earned: u32,
    pub detail: String,
    pub suggestion: Option<String>,
}

impl CheckResult {
    pub fn passed(check: Check, detail: impl Into<String>) -> Self {
        let points = check.max_points;
        Self {
            check,
            status: CheckStatus::Passed,
            points_earned: points,
            detail: detail.into(),
            suggestion: None,
        }
    }

    pub fn failed(
        check: Check,
        detail: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            check,
            status: CheckStatus::Failed,
            points_earned: 0,
            detail: detail.into(),
            suggestion: Some(suggestion.into()),
        }
    }

    pub fn warning(
        check: Check,
        points: u32,
        detail: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            check,
            status: CheckStatus::Warning,
            points_earned: points,
            detail: detail.into(),
            suggestion: Some(suggestion.into()),
        }
    }

    pub fn skipped(check: Check, reason: impl Into<String>) -> Self {
        Self {
            check,
            status: CheckStatus::Skipped,
            points_earned: 0,
            detail: reason.into(),
            suggestion: None,
        }
    }
}
