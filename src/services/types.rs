use serde::Deserialize;

/// Parsed owner/repo from a GitHub URL
#[derive(Debug, Clone)]
pub struct RepoIdentifier {
    pub owner: String,
    pub repo: String,
}

impl RepoIdentifier {
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

/// GitHub workflow file representation
#[derive(Debug, Clone, Deserialize)]
pub struct GithubContent {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub content: Option<String>,
    pub encoding: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
}

/// GitHub workflow run
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: Option<String>,
    pub status: Option<String>,
    pub conclusion: Option<String>,
    pub head_branch: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub run_started_at: Option<String>,
}

/// Response wrapper for workflow runs
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRunsResponse {
    pub total_count: u32,
    pub workflow_runs: Vec<WorkflowRun>,
}

/// Branch protection rules
#[derive(Debug, Clone, Deserialize)]
pub struct BranchProtection {
    pub required_pull_request_reviews: Option<serde_json::Value>,
    pub enforce_admins: Option<EnforceAdmins>,
    pub required_status_checks: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnforceAdmins {
    pub enabled: bool,
}

/// Repository metadata
#[derive(Debug, Clone, Deserialize)]
pub struct RepoMetadata {
    pub name: String,
    pub full_name: String,
    pub default_branch: String,
    pub private: bool,
    #[serde(default)]
    pub description: Option<String>,
}

/// Tree entry (for recursive file listing)
#[derive(Debug, Clone, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
}

/// Git tree response
#[derive(Debug, Clone, Deserialize)]
pub struct TreeResponse {
    pub sha: String,
    pub tree: Vec<TreeEntry>,
    pub truncated: bool,
}

/// GitHub release
#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub id: u64,
    pub tag_name: String,
    pub name: Option<String>,
    pub published_at: Option<String>,
}

/// Git commit list item
#[derive(Debug, Clone, Deserialize)]
pub struct CommitItem {
    pub sha: String,
    pub commit: CommitDetail,
}

/// Git commit detail
#[derive(Debug, Clone, Deserialize)]
pub struct CommitDetail {
    pub message: String,
}

/// API error
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GitHub API error {}: {}", self.status, self.message)
    }
}
