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
            max_points: 5,
        },
        Check {
            id: "pipeline_green".into(),
            name: "Pipeline vert sur main".into(),
            description: "Le dernier run du workflow sur main est en succès".into(),
            category: CheckCategory::Fundamentals,
            max_points: 5,
        },
        Check {
            id: "tests_exist".into(),
            name: "Tests présents".into(),
            description: "Des fichiers de test existent et sont exécutés dans la CI".into(),
            category: CheckCategory::Fundamentals,
            max_points: 10,
        },
        Check {
            id: "lint_in_ci".into(),
            name: "Lint dans la CI".into(),
            description: "Un step de lint/format est configuré dans le pipeline".into(),
            category: CheckCategory::Fundamentals,
            max_points: 5,
        },
        Check {
            id: "dockerfile_exists".into(),
            name: "Dockerfile présent".into(),
            description: "Un Dockerfile existe à la racine du projet".into(),
            category: CheckCategory::Fundamentals,
            max_points: 5,
        },
        Check {
            id: "docker_build_ci".into(),
            name: "Docker build dans CI".into(),
            description: "Le pipeline inclut une étape de build Docker".into(),
            category: CheckCategory::Fundamentals,
            max_points: 5,
        },
        Check {
            id: "no_secrets_in_code".into(),
            name: "Pas de secrets dans le code".into(),
            description: "Aucun secret hardcodé détecté dans les fichiers source".into(),
            category: CheckCategory::Fundamentals,
            max_points: 10,
        },
        Check {
            id: "readme_exists".into(),
            name: "README présent".into(),
            description: "Un fichier README.md existe à la racine".into(),
            category: CheckCategory::Fundamentals,
            max_points: 5,
        },
        // ── Intermediate ──
        Check {
            id: "security_scan".into(),
            name: "Scan de sécurité".into(),
            description: "Un outil de scan sécurité (Trivy, Snyk, Bandit, etc.) dans la CI".into(),
            category: CheckCategory::Intermediate,
            max_points: 10,
        },
        Check {
            id: "coverage_configured".into(),
            name: "Coverage configurée".into(),
            description: "La couverture de code est configurée dans le pipeline".into(),
            category: CheckCategory::Intermediate,
            max_points: 10,
        },
        Check {
            id: "dependabot_configured".into(),
            name: "Dependabot / Renovate".into(),
            description: "Mise à jour automatique des dépendances configurée".into(),
            category: CheckCategory::Intermediate,
            max_points: 10,
        },
        // ── Advanced ──
        Check {
            id: "branch_protection".into(),
            name: "Protection de branche".into(),
            description: "La branche main est protégée avec PR obligatoire".into(),
            category: CheckCategory::Advanced,
            max_points: 10,
        },
        Check {
            id: "pipeline_fast".into(),
            name: "Pipeline rapide (< 5 min)".into(),
            description: "La durée moyenne des derniers runs est inférieure à 5 minutes".into(),
            category: CheckCategory::Advanced,
            max_points: 5,
        },
        Check {
            id: "multi_environment".into(),
            name: "Multi-environnements".into(),
            description: "La CI/CD gère plusieurs environnements (staging, prod, etc.)".into(),
            category: CheckCategory::Advanced,
            max_points: 10,
        },
        Check {
            id: "auto_deploy".into(),
            name: "Déploiement automatique".into(),
            description: "Un déploiement automatique est configuré sur push/merge main".into(),
            category: CheckCategory::Advanced,
            max_points: 10,
        },
        // ── Bonus ──
        Check {
            id: "codeowners_exists".into(),
            name: "CODEOWNERS présent".into(),
            description: "Un fichier CODEOWNERS est configuré".into(),
            category: CheckCategory::Bonus,
            max_points: 5,
        },
        Check {
            id: "gitignore_exists".into(),
            name: ".gitignore présent".into(),
            description: "Un fichier .gitignore est configuré pour le projet".into(),
            category: CheckCategory::Bonus,
            max_points: 5,
        },
    ]
}
