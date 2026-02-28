use std::collections::HashMap;

use crate::models::{CategoryScore, CheckCategory, CheckResult, ScoreReport};
use crate::services::{GithubClient, RepoIdentifier};

use super::definitions::all_checks;
use super::runner::CheckRunner;

/// Orchestrates all checks and produces a ScoreReport
pub struct CheckEngine {
    client: GithubClient,
}

impl CheckEngine {
    pub fn new(client: GithubClient) -> Self {
        Self { client }
    }

    /// Run all checks against a repository and return a full report
    pub async fn analyze(&self, repo: &RepoIdentifier) -> Result<ScoreReport, String> {
        // Verify repo exists
        self.client
            .fetch_repo_metadata(repo)
            .await
            .map_err(|e| format!("Impossible d'accéder au repo : {}", e))?;

        let checks = all_checks();
        let runner = CheckRunner::new(&self.client, repo);

        let mut results: Vec<CheckResult> = Vec::new();
        for check in &checks {
            let result = runner.run_check(check).await;
            results.push(result);
        }

        // Group results by category
        let mut grouped: HashMap<CheckCategory, Vec<CheckResult>> = HashMap::new();
        for result in results {
            grouped
                .entry(result.check.category.clone())
                .or_default()
                .push(result);
        }

        // Build category scores
        let category_order = [
            CheckCategory::Fundamentals,
            CheckCategory::Intermediate,
            CheckCategory::Advanced,
            CheckCategory::Bonus,
        ];

        let mut categories = Vec::new();
        let mut global_passed: u32 = 0;
        let mut global_total: u32 = 0;

        for cat in &category_order {
            let cat_results = grouped.remove(cat).unwrap_or_default();
            // Warnings count as passes; Skipped checks are excluded from the total
            let passed: u32 = cat_results
                .iter()
                .filter(|r| matches!(r.status, crate::models::CheckStatus::Passed | crate::models::CheckStatus::Warning))
                .count() as u32;
            let total: u32 = cat_results
                .iter()
                .filter(|r| !matches!(r.status, crate::models::CheckStatus::Skipped))
                .count() as u32;

            global_passed += passed;
            global_total += total;

            categories.push(CategoryScore {
                category: cat.clone(),
                passed,
                total,
                results: cat_results,
            });
        }

        Ok(ScoreReport {
            repository: repo.full_name(),
            passed: global_passed,
            total: global_total,
            categories,
            analyzed_at: js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default(),
        })
    }
}
