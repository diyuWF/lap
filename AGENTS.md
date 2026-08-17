# Lap Codex Instructions

## Mission

Lap is being refactored from a personal photo manager into a local-first digital asset management system for designers, 3D artists, motion designers, and game-development workflows.

Primary repository: `diyuWF/lap`

## Current development baseline

- Pull request: `#2` — `feat: Phase 4–5 preview system and online AI automation`
- Working branch: `feat/phase4-ai-preview-pipeline`
- Base branch: `main`
- Feature implementation baseline before handoff documentation: `99fbf45576a8f74eddc26469e8a3593fbd1777a7`
- PR status: Draft; keep it Draft until manual validation is complete and the user explicitly approves merging.

Do not restart the work from `main`.
Do not use `feat/phase-4-online-ai-workflow` as the primary development branch.
Always inspect the current branch HEAD before making changes because documentation-only commits may be newer than the feature baseline above.

## Completed scope

### Phase 1–3

- Simplified Chinese localization and visible-text audit foundation
- DAM taxonomy, source metadata, workflow state, and AI suggestion storage
- Authenticated localhost capture service
- Chromium browser extension and browser capture workflow

### Phase 4

- Unified preview routing
- SVG preview
- PDF preview
- Interactive GLB, glTF, OBJ, and STL preview
- Design-asset file type indexing
- Explicit fallback UI for unsupported professional formats

### Phase 5

- Persistent online AI provider configuration
- OpenAI-compatible, Gemini, and Anthropic providers
- Structured visual metadata suggestions
- Confidence threshold and optional automatic tag application
- AI suggestion review queue
- Batch AI organization workbench

## Current gate

The project is at the manual-validation gate.

Do not start Phase 6 or broad refactoring until the current Windows validation build has been manually tested and defects have been recorded in `docs/MANUAL_VALIDATION.md` or a linked issue/PR comment.

The expected next work is defect triage and focused fixes, not new feature expansion.

## Required reading before code changes

1. `AGENTS.md`
2. `docs/HANDOFF.md`
3. `docs/MANUAL_VALIDATION.md`
4. PR `#2`, including its current checks and conversation
5. The most recent commits on `feat/phase4-ai-preview-pipeline`

## Required checks

Run the checks relevant to every changed area. A final report must state the exact commands and results.

### Frontend and localization

```bash
cd src-vite
pnpm install --frozen-lockfile
pnpm i18n:audit:strict
pnpm exec vite build
```

### Rust backend

```bash
cd src-tauri
cargo fmt --check
cargo check --locked
```

### Browser extension

```bash
node --check lap-extension/background.js
node --check lap-extension/content.js
node --check lap-extension/options.js
node --check lap-extension/popup.js
node -e "JSON.parse(require('fs').readFileSync('lap-extension/manifest.json', 'utf8')); console.log('manifest.json OK')"
```

### Repository hygiene

```bash
git diff --check
```

If the local environment cannot run a required check, report it as `BLOCKED` or `SKIPPED`; do not represent it as passing.

## Working rules

- Preserve Simplified Chinese as the primary visible UI language.
- Preserve compatibility with existing Lap libraries and user files.
- Do not introduce destructive database changes without explicit migration and rollback considerations.
- Do not commit API keys, access tokens, private endpoints, credentials, machine-specific paths, build artifacts, or user data.
- Do not log full API keys or full AI response payloads that may contain private content.
- Do not merge PR `#2`, mark it ready for review, publish a release, or modify `main` without explicit user approval.
- Do not force-push or rewrite shared history.
- Prefer focused changes and focused commits; do not mix unrelated cleanup with defect fixes.
- Keep provider-specific network behavior isolated and preserve OpenAI-compatible, Gemini, and Anthropic paths.
- Keep unsupported file formats explicit: show a useful fallback rather than silently failing.
- Update `docs/HANDOFF.md` when branch status, validation status, architecture, or blockers materially change.
- Update `docs/MANUAL_VALIDATION.md` when a manual test is performed or a defect is confirmed.

## Important integration caveat

The successful `Chat Validation Package` workflow for the current validation artifact applies a scripted sidebar-removal transformation during the build. The packaged executable may therefore contain changes not yet represented identically in the branch source.

Before making navigation or release changes, compare the workflow transformation with `src-vite/src/views/Home.vue` and reconcile the source deliberately. Do not assume the packaged UI and branch source are byte-for-byte equivalent.

## Task reporting format

Use these statuses:

- `PASS` — completed and verified
- `FAILED` — attempted but verification failed
- `SKIPPED` — intentionally not run, with reason
- `BLOCKED` — cannot proceed without environment, credentials, user decision, or manual testing

Every implementation report must include:

1. Scope completed
2. Files changed
3. Commands run and results
4. Remaining risks
5. Next required manual or automated step

## First action after handoff

Before modifying code:

1. Confirm repository, branch, PR, and current HEAD.
2. Read the required files above.
3. Inspect the existing CI and latest successful workflow runs.
4. Summarize the architecture and current validation blockers.
5. Identify any discrepancy between source and the validation-package workflow.
6. Present a plan and wait for approval before changing files, unless the user explicitly provides a concrete defect to fix.
