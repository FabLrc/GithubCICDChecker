use crate::models::{Check, CheckCategory};

/// Returns all check definitions organized by category
pub fn all_checks() -> Vec<Check> {
    vec![
        // ── Fundamentals ──
        Check {
            id: "pipeline_exists".into(),
            name: "Pipeline CI existe".into(),
            description: "Au moins un workflow YAML présent dans .github/workflows/".into(),
            category: CheckCategory::Fundamentals,
        },
        Check {
            id: "pipeline_green".into(),
            name: "Pipeline vert sur main".into(),
            description: "Le dernier run du workflow sur main est en succès".into(),
            category: CheckCategory::Fundamentals,
        },
        Check {
            id: "tests_exist".into(),
            name: "Tests présents".into(),
            description: "Des fichiers de test existent et sont exécutés dans la CI".into(),
            category: CheckCategory::Fundamentals,
        },
        Check {
            id: "tests_pass".into(),
            name: "Tests passent dans CI".into(),
            description: "Le pipeline est vert ET une étape de test a été détectée et exécutée".into(),
            category: CheckCategory::Fundamentals,
        },
        Check {
            id: "lint_in_ci".into(),
            name: "Lint dans la CI".into(),
            description: "Un step de lint/format est configuré dans le pipeline".into(),
            category: CheckCategory::Fundamentals,
        },
        Check {
            id: "dockerfile_exists".into(),
            name: "Dockerfile présent".into(),
            description: "Un Dockerfile existe à la racine du projet".into(),
            category: CheckCategory::Fundamentals,
        },
        Check {
            id: "docker_build_ci".into(),
            name: "Docker build dans CI".into(),
            description: "Le pipeline inclut une étape de build Docker".into(),
            category: CheckCategory::Fundamentals,
        },
        Check {
            id: "no_secrets_in_code".into(),
            name: "Pas de secrets dans le code".into(),
            description: "Aucun secret hardcodé détecté dans les fichiers source".into(),
            category: CheckCategory::Fundamentals,
        },
        Check {
            id: "readme_exists".into(),
            name: "README présent".into(),
            description: "Un fichier README.md existe à la racine".into(),
            category: CheckCategory::Fundamentals,
        },
        // ── Intermediate ──
        Check {
            id: "security_scan".into(),
            name: "Scan de sécurité".into(),
            description: "Un outil de scan sécurité (Trivy, Snyk, Bandit, etc.) dans la CI".into(),
            category: CheckCategory::Intermediate,
        },
        Check {
            id: "coverage_configured".into(),
            name: "Coverage configurée".into(),
            description: "La couverture de code est configurée dans le pipeline".into(),
            category: CheckCategory::Intermediate,
        },
        Check {
            id: "dependabot_configured".into(),
            name: "Dependabot / Renovate".into(),
            description: "Mise à jour automatique des dépendances configurée".into(),
            category: CheckCategory::Intermediate,
        },
        Check {
            id: "ghcr_published".into(),
            name: "Image publiée sur GHCR".into(),
            description: "L'image Docker est poussée sur GitHub Container Registry (ghcr.io)".into(),
            category: CheckCategory::Intermediate,
        },
        Check {
            id: "quality_gate".into(),
            name: "Quality gate (SonarCloud, etc.)".into(),
            description: "Un outil d'analyse qualité (SonarCloud, CodeClimate, Codacy) est intégré dans la CI".into(),
            category: CheckCategory::Intermediate,
        },
        // ── Advanced ──
        Check {
            id: "branch_protection".into(),
            name: "Protection de branche".into(),
            description: "La branche main est protégée avec PR obligatoire".into(),
            category: CheckCategory::Advanced,
        },
        Check {
            id: "pipeline_fast".into(),
            name: "Pipeline rapide (< 5 min)".into(),
            description: "La durée moyenne des derniers runs est inférieure à 5 minutes".into(),
            category: CheckCategory::Advanced,
        },
        Check {
            id: "multi_environment".into(),
            name: "Multi-environnements".into(),
            description: "La CI/CD gère plusieurs environnements (staging, prod, etc.)".into(),
            category: CheckCategory::Advanced,
        },
        Check {
            id: "auto_deploy".into(),
            name: "Déploiement automatique".into(),
            description: "Un déploiement automatique est configuré sur push/merge main".into(),
            category: CheckCategory::Advanced,
        },
        Check {
            id: "ci_cache".into(),
            name: "Cache CI optimisé".into(),
            description: "Le pipeline utilise un mécanisme de cache (actions/cache, Docker layer cache, etc.) pour accélérer les builds".into(),
            category: CheckCategory::Advanced,
        },
        Check {
            id: "ci_notifications".into(),
            name: "Notifications CI (Discord/Slack)".into(),
            description: "Des notifications sont envoyées sur Discord ou Slack en cas de succès ou d'échec du pipeline".into(),
            category: CheckCategory::Advanced,
        },
        Check {
            id: "matrix_testing".into(),
            name: "Tests en matrice (multi-version)".into(),
            description: "Le pipeline utilise une stratégie de matrix pour tester sur plusieurs versions ou OS".into(),
            category: CheckCategory::Advanced,
        },
        Check {
            id: "reusable_workflows".into(),
            name: "Workflows réutilisables".into(),
            description: "Le dépôt utilise ou définit des workflows réutilisables (workflow_call)".into(),
            category: CheckCategory::Advanced,
        },
        // ── Bonus ──
        Check {
            id: "codeowners_exists".into(),
            name: "CODEOWNERS présent".into(),
            description: "Un fichier CODEOWNERS est configuré".into(),
            category: CheckCategory::Bonus,
        },
        Check {
            id: "gitignore_exists".into(),
            name: ".gitignore présent".into(),
            description: "Un fichier .gitignore est configuré pour le projet".into(),
            category: CheckCategory::Bonus,
        },
        Check {
            id: "release_tagging".into(),
            name: "Releases / Tags GitHub".into(),
            description: "Au moins une release ou un tag GitHub existe pour versionner le projet".into(),
            category: CheckCategory::Bonus,
        },
        Check {
            id: "smoke_tests".into(),
            name: "Tests smoke / e2e post-déploiement".into(),
            description: "Des tests smoke ou e2e sont exécutés après le déploiement pour valider l'environnement".into(),
            category: CheckCategory::Bonus,
        },
        Check {
            id: "conventional_commits".into(),
            name: "Commits conventionnels (≥ 80%)".into(),
            description: "Au moins 80% des commits suivent la convention Conventional Commits (feat:, fix:, chore:, etc.)".into(),
            category: CheckCategory::Bonus,
        },
        Check {
            id: "auto_changelog".into(),
            name: "Changelog automatisé".into(),
            description: "Un outil de génération de changelog (release-please, semantic-release, etc.) est configuré".into(),
            category: CheckCategory::Bonus,
        },
        Check {
            id: "rollback_strategy".into(),
            name: "Stratégie de rollback".into(),
            description: "Le dépôt dispose d'un mécanisme de rollback (workflow dédié, workflow_dispatch, revert automatique)".into(),
            category: CheckCategory::Bonus,
        },
    ]
}
