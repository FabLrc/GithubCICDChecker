use gloo_net::http::{Request, RequestBuilder};

use super::types::*;

const GITHUB_API_BASE: &str = "https://api.github.com";

/// Client for interacting with the GitHub REST API
#[derive(Debug, Clone)]
pub struct GithubClient {
    token: Option<String>,
}

impl GithubClient {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }

    /// Parse a GitHub URL into owner/repo
    pub fn parse_repo_url(url: &str) -> Result<RepoIdentifier, String> {
        let url = url.trim().trim_end_matches('/');

        // Handle formats: "owner/repo", "https://github.com/owner/repo"
        let parts: Vec<&str> = if url.contains("github.com") {
            let after_github = url
                .split("github.com/")
                .nth(1)
                .ok_or_else(|| "Invalid GitHub URL".to_string())?;
            after_github.split('/').collect()
        } else {
            url.split('/').collect()
        };

        if parts.len() < 2 {
            return Err("URL must contain owner/repo".to_string());
        }

        // Filter out trailing segments like .git, tree/main, etc.
        let owner = parts[0].to_string();
        let repo = parts[1].trim_end_matches(".git").to_string();

        if owner.is_empty() || repo.is_empty() {
            return Err("Owner and repo name cannot be empty".to_string());
        }

        Ok(RepoIdentifier { owner, repo })
    }

    fn build_request(&self, url: &str) -> RequestBuilder {
        let req = Request::get(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "github-cicd-checker");

        if let Some(ref token) = self.token {
            req.header("Authorization", &format!("Bearer {}", token))
        } else {
            req
        }
    }

    async fn fetch_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, ApiError> {
        let response = self
            .build_request(url)
            .send()
            .await
            .map_err(|e| ApiError {
                status: 0,
                message: format!("Network error: {}", e),
            })?;

        let status = response.status();
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError {
                status,
                message: format!("HTTP {}: {}", status, body),
            });
        }

        response.json::<T>().await.map_err(|e| ApiError {
            status: 200,
            message: format!("Parse error: {}", e),
        })
    }

    async fn fetch_text(&self, url: &str) -> Result<String, ApiError> {
        let response = self
            .build_request(url)
            .send()
            .await
            .map_err(|e| ApiError {
                status: 0,
                message: format!("Network error: {}", e),
            })?;

        let status = response.status();
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError {
                status,
                message: format!("HTTP {}: {}", status, body),
            });
        }

        response.text().await.map_err(|e| ApiError {
            status: 200,
            message: format!("Read error: {}", e),
        })
    }

    /// Check if repo exists and fetch metadata
    pub async fn fetch_repo_metadata(
        &self,
        repo: &RepoIdentifier,
    ) -> Result<RepoMetadata, ApiError> {
        let url = format!("{}/repos/{}/{}", GITHUB_API_BASE, repo.owner, repo.repo);
        self.fetch_json(&url).await
    }

    /// List files in .github/workflows/
    pub async fn fetch_workflow_files(
        &self,
        repo: &RepoIdentifier,
    ) -> Result<Vec<GithubContent>, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/contents/.github/workflows",
            GITHUB_API_BASE, repo.owner, repo.repo
        );
        self.fetch_json(&url).await
    }

    /// Fetch a single file's content (base64 encoded)
    pub async fn fetch_file_content(
        &self,
        repo: &RepoIdentifier,
        path: &str,
    ) -> Result<String, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/contents/{}",
            GITHUB_API_BASE, repo.owner, repo.repo, path
        );
        let content: GithubContent = self.fetch_json(&url).await?;

        match content.content {
            Some(encoded) => {
                let cleaned = encoded.replace('\n', "").replace('\r', "");
                let decoded = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    &cleaned,
                )
                .map_err(|e| ApiError {
                    status: 0,
                    message: format!("Base64 decode error: {}", e),
                })?;
                String::from_utf8(decoded).map_err(|e| ApiError {
                    status: 0,
                    message: format!("UTF-8 decode error: {}", e),
                })
            }
            None => Err(ApiError {
                status: 0,
                message: "No content in response".to_string(),
            }),
        }
    }

    /// Fetch raw file (no base64, uses raw media type)
    pub async fn fetch_raw_file(
        &self,
        repo: &RepoIdentifier,
        path: &str,
    ) -> Result<String, ApiError> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/HEAD/{}",
            repo.owner, repo.repo, path
        );
        self.fetch_text(&url).await
    }

    /// Fetch recent workflow runs
    pub async fn fetch_workflow_runs(
        &self,
        repo: &RepoIdentifier,
        per_page: u32,
    ) -> Result<WorkflowRunsResponse, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/actions/runs?per_page={}&branch=main",
            GITHUB_API_BASE, repo.owner, repo.repo, per_page
        );
        self.fetch_json(&url).await
    }

    /// Check all workflow runs (not branch-filtered)
    pub async fn fetch_all_workflow_runs(
        &self,
        repo: &RepoIdentifier,
        per_page: u32,
    ) -> Result<WorkflowRunsResponse, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/actions/runs?per_page={}",
            GITHUB_API_BASE, repo.owner, repo.repo, per_page
        );
        self.fetch_json(&url).await
    }

    /// Fetch branch protection rules (requires token)
    pub async fn fetch_branch_protection(
        &self,
        repo: &RepoIdentifier,
        branch: &str,
    ) -> Result<BranchProtection, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/branches/{}/protection",
            GITHUB_API_BASE, repo.owner, repo.repo, branch
        );
        self.fetch_json(&url).await
    }

    /// Check if a file exists in the repo
    pub async fn file_exists(
        &self,
        repo: &RepoIdentifier,
        path: &str,
    ) -> bool {
        let url = format!(
            "{}/repos/{}/{}/contents/{}",
            GITHUB_API_BASE, repo.owner, repo.repo, path
        );
        let response = self.build_request(&url).send().await;
        matches!(response, Ok(r) if r.status() == 200)
    }

    /// Fetch the full file tree (recursive) for the repo
    pub async fn fetch_tree(
        &self,
        repo: &RepoIdentifier,
        branch: &str,
    ) -> Result<TreeResponse, ApiError> {
        let url = format!(
            "{}/repos/{}/{}/git/trees/{}?recursive=1",
            GITHUB_API_BASE, repo.owner, repo.repo, branch
        );
        self.fetch_json(&url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let result = GithubClient::parse_repo_url("https://github.com/rust-lang/rust").unwrap();
        assert_eq!(result.owner, "rust-lang");
        assert_eq!(result.repo, "rust");
    }

    #[test]
    fn test_parse_short_form() {
        let result = GithubClient::parse_repo_url("owner/repo").unwrap();
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
    }

    #[test]
    fn test_parse_trailing_slash() {
        let result =
            GithubClient::parse_repo_url("https://github.com/owner/repo/").unwrap();
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(GithubClient::parse_repo_url("not-a-url").is_err());
    }
}
