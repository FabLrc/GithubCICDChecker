use serde::{Deserialize, Serialize};

use super::check::{CheckCategory, CheckResult};

/// Score for a specific category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryScore {
    pub category: CheckCategory,
    /// Number of checks that passed or warned (counted as pass)
    pub passed: u32,
    /// Total evaluated checks (excludes Skipped)
    pub total: u32,
    pub results: Vec<CheckResult>,
}

impl CategoryScore {
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }
}

/// Complete score report for a repository
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreReport {
    pub repository: String,
    /// Total checks passed (Passed + Warning) across all categories
    pub passed: u32,
    /// Total evaluated checks (excludes Skipped) across all categories
    pub total: u32,
    pub categories: Vec<CategoryScore>,
    pub analyzed_at: String,
}

impl ScoreReport {
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }

    /// Color label matching PageSpeed Insights grading
    pub fn grade_color(&self) -> &'static str {
        let pct = self.percentage();
        if pct >= 90.0 {
            "#0cce6b" // green
        } else if pct >= 50.0 {
            "#ffa400" // orange
        } else {
            "#ff4e42" // red
        }
    }

    pub fn grade_label(&self) -> &'static str {
        let pct = self.percentage();
        if pct >= 90.0 {
            "Excellent"
        } else if pct >= 70.0 {
            "Bon"
        } else if pct >= 50.0 {
            "À améliorer"
        } else {
            "Insuffisant"
        }
    }
}
