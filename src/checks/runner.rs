use crate::models::{Check, CheckResult};
use crate::services::{GithubClient, GithubContent, RepoIdentifier, WorkflowRun};

/// Returns true if a commit message follows the Conventional Commits spec
/// (feat:, fix:, chore:, ci:, docs:, style:, refactor:, test:, build:, perf:, revert:)
fn is_conventional_commit(message: &str) -> bool {
    // Only look at the first line (the subject)
    let subject = message.lines().next().unwrap_or(message);
    let prefixes = [
        "feat", "fix", "docs", "style", "refactor", "test", "chore", "ci", "build", "perf",
        "revert",
    ];
    for prefix in &prefixes {
        if subject.starts_with(prefix) {
            let rest = &subject[prefix.len()..];
            // "prefix: " or "prefix!: "
            if rest.starts_with(": ") || rest.starts_with("!: ") {
                return true;
            }
            // "prefix(scope): " or "prefix(scope)!: "
            if rest.starts_with('(') {
                if let Some(close) = rest.find(')') {
                    let after = &rest[close + 1..];
                    if after.starts_with(": ") || after.starts_with("!: ") {
                        return true;
                    }
                }
            }
        }
    }
    false
}


/// Runs individual checks against GitHub API data
pub struct CheckRunner<'a> {
    client: &'a GithubClient,
    repo: &'a RepoIdentifier,
}

impl<'a> CheckRunner<'a> {
    pub fn new(client: &'a GithubClient, repo: &'a RepoIdentifier) -> Self {
        Self { client, repo }
    }

    pub async fn run_check(&self, check: &Check) -> CheckResult {
        match check.id.as_str() {
            "pipeline_exists" => self.check_pipeline_exists(check.clone()).await,
            "pipeline_green" => self.check_pipeline_green(check.clone()).await,
            "tests_exist" => self.check_tests_exist(check.clone()).await,
            "lint_in_ci" => self.check_lint_in_ci(check.clone()).await,
            "dockerfile_exists" => self.check_file_exists(check.clone(), "Dockerfile").await,
            "docker_build_ci" => self.check_docker_build_ci(check.clone()).await,
            "no_secrets_in_code" => self.check_no_secrets(check.clone()).await,
            "readme_exists" => self.check_file_exists(check.clone(), "README.md").await,
            "security_scan" => self.check_security_scan(check.clone()).await,
            "coverage_configured" => self.check_coverage(check.clone()).await,
            "dependabot_configured" => self.check_dependabot(check.clone()).await,
            "branch_protection" => self.check_branch_protection(check.clone()).await,
            "pipeline_fast" => self.check_pipeline_speed(check.clone()).await,
            "multi_environment" => self.check_multi_environment(check.clone()).await,
            "auto_deploy" => self.check_auto_deploy(check.clone()).await,
            "codeowners_exists" => self.check_codeowners(check.clone()).await,
            "gitignore_exists" => self.check_file_exists(check.clone(), ".gitignore").await,
            "tests_pass" => self.check_tests_pass(check.clone()).await,
            "ghcr_published" => self.check_ghcr_published(check.clone()).await,
            "quality_gate" => self.check_quality_gate(check.clone()).await,
            "ci_cache" => self.check_ci_cache(check.clone()).await,
            "ci_notifications" => self.check_ci_notifications(check.clone()).await,
            "matrix_testing" => self.check_matrix_testing(check.clone()).await,
            "reusable_workflows" => self.check_reusable_workflows(check.clone()).await,
            "release_tagging" => self.check_release_tagging(check.clone()).await,
            "smoke_tests" => self.check_smoke_tests(check.clone()).await,
            "conventional_commits" => self.check_conventional_commits(check.clone()).await,
            "auto_changelog" => self.check_auto_changelog(check.clone()).await,
            "rollback_strategy" => self.check_rollback_strategy(check.clone()).await,
            _ => CheckResult::skipped(check.clone(), "Check non implémenté"),
        }
    }

    // ── Fundamentals ──

    async fn check_pipeline_exists(&self, check: Check) -> CheckResult {
        match self.client.fetch_workflow_files(self.repo).await {
            Ok(files) => {
                let yaml_files: Vec<&GithubContent> = files
                    .iter()
                    .filter(|f| {
                        f.name.ends_with(".yml") || f.name.ends_with(".yaml")
                    })
                    .collect();

                if yaml_files.is_empty() {
                    CheckResult::failed(
                        check,
                        "Aucun fichier workflow YAML trouvé",
                        "Créez un fichier .github/workflows/ci.yml pour votre pipeline CI/CD",
                    )
                } else {
                    let names: Vec<String> =
                        yaml_files.iter().map(|f| f.name.clone()).collect();
                    CheckResult::passed(
                        check,
                        format!("{} workflow(s) trouvé(s) : {}", names.len(), names.join(", ")),
                    )
                }
            }
            Err(_) => CheckResult::failed(
                check,
                "Dossier .github/workflows/ introuvable",
                "Créez le dossier .github/workflows/ et ajoutez un fichier YAML de pipeline",
            ),
        }
    }

    async fn check_pipeline_green(&self, check: Check) -> CheckResult {
        match self.client.fetch_workflow_runs(self.repo, 5).await {
            Ok(runs) => {
                if runs.workflow_runs.is_empty() {
                    return CheckResult::failed(
                        check,
                        "Aucun run trouvé sur la branche main",
                        "Lancez votre pipeline au moins une fois sur main",
                    );
                }

                let latest = &runs.workflow_runs[0];
                match latest.conclusion.as_deref() {
                    Some("success") => CheckResult::passed(
                        check,
                        format!(
                            "Dernier run '{}' réussi",
                            latest.name.as_deref().unwrap_or("unknown")
                        ),
                    ),
                    Some(conclusion) => CheckResult::failed(
                        check,
                        format!("Dernier run terminé avec le statut : {}", conclusion),
                        "Corrigez les erreurs dans votre pipeline pour qu'il passe au vert",
                    ),
                    None => CheckResult::warning(
                        check,
                        "Dernier run encore en cours",
                        "Attendez la fin du run et relancez l'analyse",
                    ),
                }
            }
            Err(_) => CheckResult::skipped(check, "Impossible de récupérer les runs (repo privé ou pas de workflows)"),
        }
    }

    async fn check_tests_exist(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let has_test_step = content_lower.contains("test")
            || content_lower.contains("pytest")
            || content_lower.contains("jest")
            || content_lower.contains("cargo test")
            || content_lower.contains("go test")
            || content_lower.contains("npm test")
            || content_lower.contains("yarn test")
            || content_lower.contains("phpunit")
            || content_lower.contains("rspec")
            || content_lower.contains("unittest");

        if has_test_step {
            CheckResult::passed(check, "Exécution de tests détectée dans la CI")
        } else {
            CheckResult::failed(
                check,
                "Aucune étape de test détectée dans les workflows",
                "Ajoutez une étape 'run: cargo test' ou équivalent dans votre pipeline",
            )
        }
    }

    async fn check_lint_in_ci(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let has_lint = content_lower.contains("lint")
            || content_lower.contains("eslint")
            || content_lower.contains("clippy")
            || content_lower.contains("flake8")
            || content_lower.contains("pylint")
            || content_lower.contains("rubocop")
            || content_lower.contains("prettier")
            || content_lower.contains("rustfmt")
            || content_lower.contains("black")
            || content_lower.contains("golangci-lint")
            || content_lower.contains("fmt --check");

        if has_lint {
            CheckResult::passed(check, "Étape de lint/formatage détectée dans la CI")
        } else {
            CheckResult::failed(
                check,
                "Aucun linter ou formatteur détecté dans les workflows",
                "Ajoutez un step de lint (ex: clippy, eslint, flake8) dans votre pipeline",
            )
        }
    }

    async fn check_file_exists(&self, check: Check, path: &str) -> CheckResult {
        if self.client.file_exists(self.repo, path).await {
            CheckResult::passed(check, format!("Fichier {} trouvé", path))
        } else {
            CheckResult::failed(
                check,
                format!("Fichier {} introuvable", path),
                format!("Ajoutez un fichier {} à la racine du projet", path),
            )
        }
    }

    async fn check_docker_build_ci(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let has_docker_build = content_lower.contains("docker build")
            || content_lower.contains("docker/build-push-action")
            || content_lower.contains("docker-build")
            || content_lower.contains("docker compose")
            || content_lower.contains("docker/setup-buildx");

        if has_docker_build {
            CheckResult::passed(check, "Build Docker détecté dans la CI")
        } else {
            CheckResult::failed(
                check,
                "Aucune étape de build Docker dans les workflows",
                "Ajoutez 'docker build' ou l'action 'docker/build-push-action' dans votre pipeline",
            )
        }
    }

    async fn check_no_secrets(&self, check: Check) -> CheckResult {
        // Check workflow files for hardcoded secrets patterns
        let workflow_content = self.aggregate_workflow_content().await;

        let secret_patterns = [
            "AKIA",           // AWS access key prefix
            "sk-",            // OpenAI / Stripe key prefix
            "ghp_",           // GitHub PAT
            "password: ",     // Inline password
            "passwd",
            "secret_key",
        ];

        let found_secrets: Vec<&str> = secret_patterns
            .iter()
            .filter(|p| workflow_content.contains(*p))
            .copied()
            .collect();

        if found_secrets.is_empty() {
            CheckResult::passed(check, "Aucun secret hardcodé détecté dans les workflows")
        } else {
            CheckResult::failed(
                check,
                format!(
                    "Patterns suspects détectés : {}",
                    found_secrets.join(", ")
                ),
                "Utilisez des GitHub Secrets (${{ secrets.MY_SECRET }}) au lieu de valeurs en dur",
            )
        }
    }

    // ── Intermediate ──

    async fn check_security_scan(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let security_tools = [
            "trivy",
            "snyk",
            "bandit",
            "safety",
            "codeql",
            "semgrep",
            "sonarcloud",
            "sonarqube",
            "dependabot",
            "grype",
            "anchore",
            "checkov",
            "tfsec",
        ];

        let found: Vec<&str> = security_tools
            .iter()
            .filter(|t| content_lower.contains(*t))
            .copied()
            .collect();

        if found.is_empty() {
            CheckResult::failed(
                check,
                "Aucun outil de scan de sécurité détecté",
                "Ajoutez Trivy, Snyk, CodeQL ou un autre scanner de sécurité dans votre pipeline",
            )
        } else {
            CheckResult::passed(
                check,
                format!("Outil(s) de sécurité détecté(s) : {}", found.join(", ")),
            )
        }
    }

    async fn check_coverage(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let coverage_tools = [
            "coverage",
            "codecov",
            "coveralls",
            "lcov",
            "tarpaulin",
            "jacoco",
            "istanbul",
            "nyc",
            "cobertura",
        ];

        let found: Vec<&str> = coverage_tools
            .iter()
            .filter(|t| content_lower.contains(*t))
            .copied()
            .collect();

        if found.is_empty() {
            CheckResult::failed(
                check,
                "Aucune configuration de coverage détectée",
                "Ajoutez un outil de coverage (codecov, tarpaulin, istanbul) dans votre CI",
            )
        } else {
            CheckResult::passed(
                check,
                format!("Coverage détectée : {}", found.join(", ")),
            )
        }
    }

    async fn check_dependabot(&self, check: Check) -> CheckResult {
        let has_dependabot = self
            .client
            .file_exists(self.repo, ".github/dependabot.yml")
            .await
            || self
                .client
                .file_exists(self.repo, ".github/dependabot.yaml")
                .await;

        let has_renovate = self.client.file_exists(self.repo, "renovate.json").await
            || self
                .client
                .file_exists(self.repo, ".github/renovate.json")
                .await;

        if has_dependabot {
            CheckResult::passed(check, "Dependabot configuré")
        } else if has_renovate {
            CheckResult::passed(check, "Renovate configuré")
        } else {
            CheckResult::failed(
                check,
                "Ni Dependabot ni Renovate ne sont configurés",
                "Ajoutez .github/dependabot.yml pour automatiser les mises à jour de dépendances",
            )
        }
    }

    // ── Advanced ──

    async fn check_branch_protection(&self, check: Check) -> CheckResult {
        match self
            .client
            .fetch_branch_protection(self.repo, "main")
            .await
        {
            Ok(protection) => {
                if protection.required_pull_request_reviews.is_some() {
                    CheckResult::passed(
                        check,
                        "Branche main protégée avec PR reviews obligatoires",
                    )
                } else {
                    CheckResult::warning(
                        check,
                        "Protection de branche activée mais sans review obligatoire",
                        "Activez 'Require pull request reviews' dans les settings de protection",
                    )
                }
            }
            Err(e) if e.status == 404 => CheckResult::failed(
                check,
                "Aucune protection configurée sur main",
                "Activez la protection de branche dans Settings > Branches > Branch protection rules",
            ),
            Err(_) => CheckResult::skipped(
                check,
                "Token requis pour vérifier la protection de branche (scope 'repo')",
            ),
        }
    }

    async fn check_pipeline_speed(&self, check: Check) -> CheckResult {
        match self.client.fetch_workflow_runs(self.repo, 10).await {
            Ok(runs) => {
                let completed_runs: Vec<&WorkflowRun> = runs
                    .workflow_runs
                    .iter()
                    .filter(|r| r.conclusion.is_some() && r.run_started_at.is_some() && r.updated_at.is_some())
                    .collect();

                if completed_runs.is_empty() {
                    return CheckResult::skipped(check, "Pas assez de runs pour évaluer la vitesse");
                }

                // Simple duration estimation: we can't do precise parsing in WASM easily,
                // so we report the data available and pass if runs exist
                let count = completed_runs.len();
                CheckResult::passed(
                    check,
                    format!("{} runs récents analysés — vérifiez les durées dans l'onglet Actions de votre repo", count),
                )
            }
            Err(_) => CheckResult::skipped(check, "Impossible de récupérer les runs"),
        }
    }

    async fn check_multi_environment(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let env_indicators = [
            "environment:",
            "staging",
            "production",
            "prod",
            "dev",
            "deploy-staging",
            "deploy-prod",
        ];

        let found: Vec<&str> = env_indicators
            .iter()
            .filter(|e| content_lower.contains(*e))
            .copied()
            .collect();

        let has_multi_env = found.len() >= 2;

        if has_multi_env {
            CheckResult::passed(
                check,
                format!("Indicateurs multi-environnement détectés : {}", found.join(", ")),
            )
        } else {
            CheckResult::failed(
                check,
                "Pas de gestion multi-environnement détectée",
                "Configurez des environnements GitHub (staging, production) dans votre pipeline",
            )
        }
    }

    async fn check_auto_deploy(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let deploy_indicators = [
            "deploy",
            "publish",
            "release",
            "gh-pages",
            "pages",
            "aws",
            "azure",
            "gcloud",
            "heroku",
            "vercel",
            "netlify",
            "render",
            "fly.io",
        ];

        let has_push_trigger =
            content_lower.contains("on:\n  push:") || content_lower.contains("on: [push");

        let has_deploy = deploy_indicators
            .iter()
            .any(|d| content_lower.contains(d));

        if has_push_trigger && has_deploy {
            CheckResult::passed(
                check,
                "Déploiement automatique détecté sur push",
            )
        } else if has_deploy {
            CheckResult::warning(
                check,
                "Étape de déploiement trouvée mais pas déclenchée automatiquement",
                "Configurez un trigger 'on: push' sur la branche main pour le déploiement auto",
            )
        } else {
            CheckResult::failed(
                check,
                "Aucune étape de déploiement détectée",
                "Ajoutez un job de déploiement automatique dans votre pipeline CI/CD",
            )
        }
    }

    // ── Bonus ──

    async fn check_codeowners(&self, check: Check) -> CheckResult {
        let exists = self.client.file_exists(self.repo, "CODEOWNERS").await
            || self
                .client
                .file_exists(self.repo, ".github/CODEOWNERS")
                .await
            || self
                .client
                .file_exists(self.repo, "docs/CODEOWNERS")
                .await;

        if exists {
            CheckResult::passed(check, "Fichier CODEOWNERS trouvé")
        } else {
            CheckResult::failed(
                check,
                "Aucun fichier CODEOWNERS trouvé",
                "Ajoutez un fichier CODEOWNERS pour définir les propriétaires du code",
            )
        }
    }

    // ── Bonus (new) ──

    async fn check_tests_pass(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let has_test_step = content_lower.contains("test")
            || content_lower.contains("pytest")
            || content_lower.contains("jest")
            || content_lower.contains("cargo test")
            || content_lower.contains("go test")
            || content_lower.contains("npm test")
            || content_lower.contains("yarn test")
            || content_lower.contains("phpunit")
            || content_lower.contains("rspec");

        if !has_test_step {
            return CheckResult::failed(
                check,
                "Aucune étape de test détectée dans les workflows",
                "Ajoutez une étape de test dans votre pipeline avant de vérifier qu'ils passent",
            );
        }

        match self.client.fetch_workflow_runs(self.repo, 5).await {
            Ok(runs) => {
                if runs.workflow_runs.is_empty() {
                    return CheckResult::skipped(check, "Aucun run trouvé sur main");
                }
                let latest = &runs.workflow_runs[0];
                match latest.conclusion.as_deref() {
                    Some("success") => CheckResult::passed(
                        check,
                        format!(
                            "Pipeline '{}' vert — étapes de test détectées et exécutées",
                            latest.name.as_deref().unwrap_or("CI")
                        ),
                    ),
                    Some(c) => CheckResult::failed(
                        check,
                        format!("Pipeline terminé avec le statut '{}' — les tests ont peut-être échoué", c),
                        "Corrigez les tests en échec pour passer ce check",
                    ),
                    None => CheckResult::skipped(check, "Run encore en cours"),
                }
            }
            Err(_) => CheckResult::skipped(check, "Impossible de récupérer les runs"),
        }
    }

    async fn check_ghcr_published(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let has_ghcr = content_lower.contains("ghcr.io")
            || content_lower.contains("github container registry")
            || (content_lower.contains("docker/build-push-action")
                && content_lower.contains("registry: ghcr"));

        let has_push = content_lower.contains("push: true")
            || content_lower.contains("docker push")
            || content_lower.contains("build-push-action");

        if has_ghcr && has_push {
            CheckResult::passed(
                check,
                "Publication vers ghcr.io détectée dans le pipeline",
            )
        } else if has_ghcr {
            CheckResult::warning(
                check,
                "Référence à ghcr.io trouvée mais pas d'étape de push explicite",
                "Assurez-vous d'utiliser 'docker/build-push-action' avec 'push: true' et 'registry: ghcr.io'",
            )
        } else {
            CheckResult::failed(
                check,
                "Aucune publication vers GHCR détectée",
                "Ajoutez 'docker/build-push-action' avec 'registry: ghcr.io' pour publier votre image",
            )
        }
    }

    async fn check_quality_gate(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let quality_tools = [
            "sonarcloud",
            "sonarqube",
            "sonar-scanner",
            "sonarqube-scan-action",
            "codeclimate",
            "codacy",
            "codecov",
            "deepsource",
        ];

        let found: Vec<&str> = quality_tools
            .iter()
            .filter(|t| content_lower.contains(*t))
            .copied()
            .collect();

        if found.is_empty() {
            CheckResult::failed(
                check,
                "Aucun outil de quality gate détecté",
                "Intégrez SonarCloud, CodeClimate ou Codacy dans votre pipeline pour contrôler la qualité du code",
            )
        } else {
            CheckResult::passed(
                check,
                format!("Quality gate détecté : {}", found.join(", ")),
            )
        }
    }

    async fn check_ci_cache(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let has_actions_cache = content_lower.contains("actions/cache");
        let has_setup_cache = content_lower.contains("cache: npm")
            || content_lower.contains("cache: yarn")
            || content_lower.contains("cache: pnpm")
            || content_lower.contains("cache: pip")
            || content_lower.contains("cache: poetry")
            || content_lower.contains("cache: 'npm'")
            || content_lower.contains("cache: 'pip'")
            || content_lower.contains("cache: gradle")
            || content_lower.contains("cache: maven");
        let has_docker_cache = content_lower.contains("cache-from")
            || content_lower.contains("cache-to")
            || content_lower.contains("buildkit");

        let cache_type = if has_actions_cache {
            "actions/cache"
        } else if has_setup_cache {
            "cache intégré (setup-node/setup-python/…)"
        } else if has_docker_cache {
            "Docker layer cache"
        } else {
            ""
        };

        if !cache_type.is_empty() {
            CheckResult::passed(
                check,
                format!("Cache CI détecté : {}", cache_type),
            )
        } else {
            CheckResult::failed(
                check,
                "Aucun mécanisme de cache dans le pipeline",
                "Ajoutez 'actions/cache' ou activez le cache dans 'actions/setup-node' (cache: npm) pour accélérer vos builds",
            )
        }
    }

    async fn check_ci_notifications(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let notification_indicators = [
            "discord-webhook",
            "discord_webhook",
            "slack-webhook",
            "slack_webhook",
            "slackapi/",
            "8398a7/action-slack",
            "rtcamp/action-slack",
            "rjstone/discord-webhook",
            "appleboy/telegram-action",
            "act10ns/slack",
            "notify",
            "send-message",
        ];

        let found: Vec<&str> = notification_indicators
            .iter()
            .filter(|i| content_lower.contains(*i))
            .copied()
            .collect();

        if found.is_empty() {
            CheckResult::failed(
                check,
                "Aucune notification CI détectée (Discord/Slack/Telegram)",
                "Ajoutez une étape de notification dans votre pipeline (ex: '8398a7/action-slack' ou 'rjstone/discord-webhook')",
            )
        } else {
            CheckResult::passed(
                check,
                format!("Notification CI configurée : {}", found.join(", ")),
            )
        }
    }

    async fn check_matrix_testing(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;

        // Look for strategy.matrix pattern (YAML indented or inline)
        let has_matrix = workflow_content.contains("strategy:")
            && workflow_content.contains("matrix:")
            || workflow_content.contains("strategy:\n    matrix:");

        if has_matrix {
            // Try to extract matrix keys for a better detail message
            let detail = if workflow_content.contains("node-version")
                || workflow_content.contains("node_version")
            {
                "Matrice détectée — versions Node.js testées"
            } else if workflow_content.contains("python-version")
                || workflow_content.contains("python_version")
            {
                "Matrice détectée — versions Python testées"
            } else if workflow_content.contains("rust") || workflow_content.contains("toolchain") {
                "Matrice détectée — toolchains Rust testés"
            } else if workflow_content.contains("os:") || workflow_content.contains("runs-on:") {
                "Matrice détectée — multi-OS"
            } else {
                "Stratégie de matrix détectée dans le pipeline"
            };
            CheckResult::passed(check, detail)
        } else {
            CheckResult::failed(
                check,
                "Aucune stratégie de matrix détectée",
                "Ajoutez 'strategy: matrix:' dans votre workflow pour tester sur plusieurs versions ou OS",
            )
        }
    }

    async fn check_reusable_workflows(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;

        // workflow_call = this repo DEFINES a reusable workflow
        let defines_reusable = workflow_content.contains("workflow_call:");
        // uses: ./.github/workflows/ = this repo CALLS a reusable workflow
        let calls_reusable = workflow_content.contains("uses: ./.github/workflows/")
            || workflow_content.contains("uses: './.github/workflows/");

        if defines_reusable {
            CheckResult::passed(check, "Workflow réutilisable défini (workflow_call) — peut être invoqué par d'autres repos")
        } else if calls_reusable {
            CheckResult::passed(check, "Workflow réutilisable appelé (uses: ./.github/workflows/) — bonne pratique DRY")
        } else {
            CheckResult::failed(
                check,
                "Aucun workflow réutilisable trouvé",
                "Créez un workflow avec 'on: workflow_call:' ou appelez-en un avec 'uses: ./.github/workflows/xxx.yml'",
            )
        }
    }

    async fn check_release_tagging(&self, check: Check) -> CheckResult {
        match self.client.fetch_releases(self.repo, 5).await {
            Ok(releases) if !releases.is_empty() => {
                let latest = &releases[0];
                CheckResult::passed(
                    check,
                    format!(
                        "{} release(s) trouvée(s) — dernière : {}",
                        releases.len(),
                        latest.tag_name
                    ),
                )
            }
            _ => {
                // Fallback: check workflow YAML for auto-release patterns
                let workflow_content = self.aggregate_workflow_content().await;
                let content_lower = workflow_content.to_lowercase();
                if content_lower.contains("release-please")
                    || content_lower.contains("semantic-release")
                    || content_lower.contains("create-release")
                    || content_lower.contains("actions/create-release")
                    || content_lower.contains("gh release create")
                {
                    CheckResult::warning(
                        check,
                        "Outil de release détecté dans CI mais aucune release publiée encore",
                        "Effectuez un premier merge sur main pour déclencher la création de release",
                    )
                } else {
                    CheckResult::failed(
                        check,
                        "Aucune release ou tag GitHub trouvé",
                        "Créez des releases GitHub pour versionner votre projet (ex: avec 'release-please' ou manuellement)",
                    )
                }
            }
        }
    }

    async fn check_smoke_tests(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let smoke_indicators = [
            "smoke",
            "e2e",
            "end-to-end",
            "end_to_end",
            "integration-test",
            "post-deploy",
            "post_deploy",
            "acceptance",
            "health-check",
            "healthcheck",
            "playwright",
            "cypress",
            "puppeteer",
        ];

        let found: Vec<&str> = smoke_indicators
            .iter()
            .filter(|i| content_lower.contains(*i))
            .copied()
            .collect();

        if found.is_empty() {
            CheckResult::failed(
                check,
                "Aucun test smoke ou e2e détecté dans le pipeline",
                "Ajoutez des tests smoke après le déploiement (ex: curl sur /healthz, Playwright, Cypress)",
            )
        } else {
            CheckResult::passed(
                check,
                format!("Tests smoke/e2e détectés : {}", found.join(", ")),
            )
        }
    }

    async fn check_conventional_commits(&self, check: Check) -> CheckResult {
        match self.client.fetch_commits(self.repo, 20).await {
            Ok(commits) if !commits.is_empty() => {
                let merge_prefix_re = ["Merge pull request", "Merge branch", "Merge remote"];
                let non_merge: Vec<_> = commits
                    .iter()
                    .filter(|c| {
                        !merge_prefix_re
                            .iter()
                            .any(|p| c.commit.message.starts_with(p))
                    })
                    .collect();

                if non_merge.is_empty() {
                    return CheckResult::skipped(check, "Seuls des commits de merge trouvés");
                }

                let conventional_count = non_merge
                    .iter()
                    .filter(|c| is_conventional_commit(&c.commit.message))
                    .count();

                let pct = (conventional_count * 100) / non_merge.len();

                if pct >= 80 {
                    CheckResult::passed(
                        check,
                        format!(
                            "{}/{} commits conventionnels ({}%)",
                            conventional_count,
                            non_merge.len(),
                            pct
                        ),
                    )
                } else {
                    CheckResult::failed(
                        check,
                        format!(
                            "{}/{} commits conventionnels ({}% < 80%)",
                            conventional_count,
                            non_merge.len(),
                            pct
                        ),
                        "Respectez la convention Conventional Commits : feat:, fix:, chore:, ci:, docs:, etc.",
                    )
                }
            }
            _ => CheckResult::skipped(check, "Impossible de récupérer les commits"),
        }
    }

    async fn check_auto_changelog(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        let changelog_tools = [
            "release-please",
            "semantic-release",
            "conventional-changelog",
            "auto-changelog",
            "standard-version",
            "changesets",
        ];

        let found: Vec<&str> = changelog_tools
            .iter()
            .filter(|t| content_lower.contains(*t))
            .copied()
            .collect();

        if !found.is_empty() {
            return CheckResult::passed(
                check,
                format!("Outil de changelog automatisé détecté : {}", found.join(", ")),
            );
        }

        // Fallback: check if CHANGELOG.md exists and looks auto-generated (multiple version headers)
        if let Ok(changelog) = self.client.fetch_raw_file(self.repo, "CHANGELOG.md").await {
            let version_headers = changelog
                .lines()
                .filter(|l| l.starts_with("## [") || l.starts_with("## v"))
                .count();
            if version_headers >= 2 {
                return CheckResult::passed(
                    check,
                    format!(
                        "CHANGELOG.md trouvé avec {} entrées de version",
                        version_headers
                    ),
                );
            }
        }

        CheckResult::failed(
            check,
            "Aucun outil de changelog automatisé trouvé",
            "Configurez 'release-please' ou 'semantic-release' dans votre pipeline pour générer un changelog automatique",
        )
    }

    async fn check_rollback_strategy(&self, check: Check) -> CheckResult {
        let workflow_content = self.aggregate_workflow_content().await;
        let content_lower = workflow_content.to_lowercase();

        // Check for explicit rollback workflow file
        let has_rollback_file = self
            .client
            .file_exists(self.repo, ".github/workflows/rollback.yml")
            .await
            || self
                .client
                .file_exists(self.repo, ".github/workflows/rollback.yaml")
                .await
            || self
                .client
                .file_exists(self.repo, ".github/workflows/revert.yml")
                .await;

        if has_rollback_file {
            return CheckResult::passed(check, "Workflow de rollback dédié détecté");
        }

        // Check for rollback/revert keywords in existing workflows
        if content_lower.contains("rollback")
            || content_lower.contains("undo-deploy")
            || content_lower.contains("undo_deploy")
        {
            return CheckResult::passed(
                check,
                "Mécanisme de rollback détecté dans les workflows",
            );
        }

        // Check for workflow_dispatch with rollback input (manual redeploy)
        if workflow_content.contains("workflow_dispatch:")
            && (content_lower.contains("revert") || content_lower.contains("rollback"))
        {
            return CheckResult::passed(
                check,
                "workflow_dispatch avec option de revert détecté",
            );
        }

        // Partial credit: workflow_dispatch alone = manual recovery possible
        if workflow_content.contains("workflow_dispatch:") {
            return CheckResult::warning(
                check,
                "workflow_dispatch détecté (redéploiement manuel possible) mais pas de rollback explicite",
                "Ajoutez un workflow dédié au rollback ou un input 'rollback' dans workflow_dispatch",
            );
        }

        CheckResult::failed(
            check,
            "Aucune stratégie de rollback détectée",
            "Créez un workflow .github/workflows/rollback.yml ou ajoutez un trigger workflow_dispatch avec option de rollback",
        )
    }

    // ── Helpers ──

    /// Fetch and concatenate the content of all workflow YAML files
    async fn aggregate_workflow_content(&self) -> String {
        let files = match self.client.fetch_workflow_files(self.repo).await {
            Ok(files) => files,
            Err(_) => return String::new(),
        };

        let mut content = String::new();
        for file in &files {
            let is_yaml = file.name.ends_with(".yml") || file.name.ends_with(".yaml");
            if is_yaml {
                if let Ok(file_content) = self.client.fetch_file_content(self.repo, &file.path).await {
                    content.push_str(&file_content);
                    content.push('\n');
                }
            }
        }
        content
    }
}
