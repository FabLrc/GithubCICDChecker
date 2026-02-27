use serde::{Deserialize, Serialize};

use super::check::{CheckCategory, CheckResult};

/// Score for a specific category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryScore {
    pub category: CheckCategory,
    pub earned: u32,
    pub max: u32,
    pub results: Vec<CheckResult>,
}

impl CategoryScore {
    pub fn percentage(&self) -> f64 {
        if self.max == 0 {
            return 0.0;
        }
        (self.earned as f64 / self.max as f64) * 100.0
    }
}

/// Complete score report for a repository
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreReport {
    pub repository: String,
    pub total_score: u32,
    pub max_score: u32,
    pub categories: Vec<CategoryScore>,
    pub analyzed_at: String,
}

impl ScoreReport {
    pub fn percentage(&self) -> f64 {
        if self.max_score == 0 {
            return 0.0;
        }
        (self.total_score as f64 / self.max_score as f64) * 100.0
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
