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

}

/// Definition of a check to perform
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CheckCategory,
}

/// Result of running a check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    pub check: Check,
    pub status: CheckStatus,
    pub detail: String,
    pub suggestion: Option<String>,
}

impl CheckResult {
    pub fn passed(check: Check, detail: impl Into<String>) -> Self {
        Self {
            check,
            status: CheckStatus::Passed,
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
            detail: detail.into(),
            suggestion: Some(suggestion.into()),
        }
    }

    pub fn warning(
        check: Check,
        detail: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            check,
            status: CheckStatus::Warning,
            detail: detail.into(),
            suggestion: Some(suggestion.into()),
        }
    }

    pub fn skipped(check: Check, reason: impl Into<String>) -> Self {
        Self {
            check,
            status: CheckStatus::Skipped,
            detail: reason.into(),
            suggestion: None,
        }
    }
}
