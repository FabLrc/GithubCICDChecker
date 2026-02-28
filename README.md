# ⚡ GitHub CI/CD Checker

> Analysez et notez la qualité CI/CD de n'importe quel dépôt GitHub — directement depuis votre navigateur.

[![Deploy to GitHub Pages](https://img.shields.io/badge/deploy-GitHub%20Pages-blue?logo=github)](https://github.com/)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust%20%2B%20WASM-orange?logo=rust)](https://www.rust-lang.org/)
[![Yew Framework](https://img.shields.io/badge/framework-Yew-green)](https://yew.rs/)

---

## Présentation

**GitHub CI/CD Checker** est une application web qui analyse la qualité de la pipeline CI/CD d'un dépôt GitHub et attribue un **score de qualité** selon le pourcentage de checks réussis, réparti en 6 domaines fonctionnels. L'interface est inspirée de [Google PageSpeed Insights](https://pagespeed.web.dev/) : un score circulaire, un code couleur (vert/orange/rouge), et des recommandations actionables pour chaque check.

**Tout tourne dans le navigateur** — aucun backend requis. L'application est compilée en WebAssembly (Rust → WASM) et appelle directement l'API GitHub depuis le browser.

<p align="center">
  <strong>Entrez une URL → Obtenez un rapport détaillé en quelques secondes.</strong>
</p>

---

## Fonctionnalités

- **30 checks automatisés** couvrant pipelines CI, tests, sécurité, conteneurisation, déploiement et bonnes pratiques
- **Score visuel** avec jauge circulaire à la PageSpeed Insights
- **Détail par catégorie** avec suggestions d'amélioration pour chaque check échoué
- **Fonctionne sans token** (les repos publics) — token optionnel pour les checks avancés (branch protection)
- **Zero backend** — 100% client-side, déployable sur GitHub Pages
- **Rapide** — compilé en Rust/WASM pour des performances natives dans le browser

---

## Grille de Scoring (30 checks)

### 🔄 Pipeline CI (7 checks)

| Check | Description |
|-------|-------------|
| Pipeline CI existe | Workflow YAML dans `.github/workflows/` |
| Pipeline vert sur main | Dernier run sur `main` est en succès |
| Pipeline rapide (< 5 min) | Durée moyenne des runs < 5 minutes |
| Cache CI optimisé | actions/cache ou Docker layer cache |
| Tests en matrice | Stratégie matrix pour multi-version |
| Workflows réutilisables | workflow_call défini ou appelé |
| Notifications CI | Discord/Slack webhooks configurés |

### 🧪 Qualité & Tests (5 checks)

| Check | Description |
|-------|-------------|
| Tests présents | Tests détectés et exécutés dans la CI |
| Tests passent dans CI | Pipeline vert + tests exécutés |
| Lint dans la CI | Step de lint/formatage configuré |
| Coverage configurée | Couverture de code instrumentée |
| Quality gate | SonarCloud / CodeClimate / Codacy intégré |

### 🔒 Sécurité (4 checks)

| Check | Description |
|-------|-------------|
| Pas de secrets dans le code | Aucun secret hardcodé détecté |
| Scan de sécurité | Trivy / Snyk / Bandit / CodeQL |
| Dependabot / Renovate | Mise à jour auto des dépendances |
| Protection de branche | `main` protégée avec PR obligatoire |

### 🐳 Conteneurisation (3 checks)

| Check | Description |
|-------|-------------|
| Dockerfile présent | Dockerfile à la racine du projet |
| Docker build dans CI | Étape de build Docker dans le pipeline |
| Image publiée sur GHCR | docker/build-push-action vers ghcr.io |

### 🚀 Déploiement (4 checks)

| Check | Description |
|-------|-------------|
| Déploiement automatique | Deploy auto sur push/merge main |
| Multi-environnements | staging + production configurés |
| Tests smoke / e2e post-déploiement | Vérification post-déploiement |
| Stratégie de rollback | Mécanisme de rollback ou recovery |

### 📋 Bonnes Pratiques (6 checks)

| Check | Description |
|-------|-------------|
| README présent | Fichier README.md à la racine |
| .gitignore présent | Fichier .gitignore configuré |
| CODEOWNERS présent | Propriétaires du code définis |
| Commits conventionnels (≥ 80%) | Conventional Commits respectés |
| Changelog automatisé | release-please / semantic-release |
| Releases / Tags GitHub | Au moins une release ou un tag |

**Scoring** : Pourcentage de checks réussis sur l'ensemble des checks évalués. Les checks `Skipped` sont exclus du total. Un check en état `Warning` (⚠️ passage partiel) compte comme réussi.

---

## Stack Technique

| Technologie | Rôle |
|-------------|------|
| **Rust** | Langage principal |
| **Yew 0.21** | Framework UI (comme React, en Rust) |
| **wasm-bindgen** | Interop Rust ↔ JavaScript |
| **gloo-net** | Appels HTTP depuis WASM |
| **Trunk** | Build toolchain WASM |
| **GitHub Pages** | Hébergement statique |
| **GitHub REST API** | Source des données d'analyse |

---

## Démarrage Rapide

### Prérequis

- [Rust](https://rustup.rs/) (stable)
- Target WASM : `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/) : `cargo install trunk`

### Lancer en local

```bash
# Cloner le projet
git clone https://github.com/<your-username>/GithubCICDChecker.git
cd GithubCICDChecker

# Lancer le serveur de développement
trunk serve --open
```

L'application sera accessible sur `http://127.0.0.1:8080`.

### Build de production

```bash
trunk build --release
```

Les fichiers statiques sont générés dans `dist/`.

---

## Architecture du Projet

```
src/
├── main.rs                  # Point d'entrée WASM
├── lib.rs                   # Exports publics des modules
├── components/              # Composants UI Yew
│   ├── app.rs               # Composant racine + state machine
│   ├── header.rs            # Barre de navigation
│   ├── footer.rs            # Pied de page
│   ├── search_bar.rs        # Barre de recherche + token
│   ├── score_gauge.rs       # Jauge circulaire SVG
│   └── results.rs           # Affichage résultats + catégories
├── checks/                  # Moteur d'analyse
│   ├── definitions.rs       # Définitions des 30 checks
│   ├── runner.rs            # Logique d'évaluation par check
│   └── engine.rs            # Orchestrateur + scoring
├── models/                  # Modèles de données
│   ├── check.rs             # Check, CheckResult, CheckStatus
│   └── score.rs             # ScoreReport, CategoryScore
└── services/                # Couche d'accès externe
    ├── client.rs            # Client GitHub REST API
    └── types.rs             # Types de réponse API
```

### Principes d'architecture

- **Separation of Concerns** : UI (`components/`) ↔ logique métier (`checks/`) ↔ accès données (`services/`)
- **Composition over inheritance** : composants Yew fonctionnels composables
- **Single Responsibility** : un fichier = un composant ou un module cohérent
- **Testabilité** : le client API et le moteur de checks sont indépendants de l'UI

---

## Déploiement GitHub Pages

Le workflow CI/CD (`.github/workflows/deploy.yml`) est déjà configuré :

1. **Activer GitHub Pages** dans les settings du repo : Source → GitHub Actions
2. Chaque push sur `main` déclenche un build + déploiement automatique
3. L'application est accessible à `https://<username>.github.io/GithubCICDChecker/`

---

## Roadmap

### Phase 1 — Moteur de Checks + UI ✅

- [x] 30 checks automatisés via l'API GitHub (18 initiaux + 12 avancés)
- [x] Interface PageSpeed Insights (score circulaire, 6 catégories par domaine, détails)
- [x] Suggestions d'amélioration pour chaque check échoué
- [x] Support du GitHub PAT (fine-grained) pour les checks avancés et l'IA
- [x] Build WASM + déploiement GitHub Pages

### Phase 2 — AI Review

- [x] Intégration GitHub Models API (GPT-4.1-mini) pour analyse IA des workflows YAML
- [x] Recommandations IA affichées dans un panneau dédié
- [x] Suggestions contextuelles basées sur les checks échoués

### Phase 3 — UX & Fonctionnalités Avancées

- [ ] Historique des analyses (stockage localStorage)
- [ ] Export du rapport en PDF / Markdown
- [ ] Mode comparaison entre deux repos
- [ ] Thème sombre
- [ ] i18n (français / anglais)

### Phase 4 — Infrastructure (si besoin)

- [ ] Backend AWS Lambda (Rust) si les CORS ou rate limits deviennent bloquants
- [ ] Cache des résultats d'analyse
- [ ] Authentification OAuth GitHub pour une meilleure expérience

---

## Contribuer

Les contributions sont les bienvenues ! Merci de suivre les principes du projet :

- **Clean Code** : nommage sémantique, fonctions courtes, pas de magic numbers
- **SOLID** : chaque module a une responsabilité unique
- **Boy Scout Rule** : laissez le code un peu plus propre qu'à votre arrivée

```bash
# Vérifier la compilation
cargo build --target wasm32-unknown-unknown

# Lancer les tests
cargo test

# Build complet avec Trunk
trunk build
```

---

## Licence

MIT © 2026
