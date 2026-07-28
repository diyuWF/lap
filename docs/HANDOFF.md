# Lap Project Handoff

## Product direction

Lap is moving from a photo-oriented organizer toward a local-first Digital Asset Management (DAM) tool for creators.

Target users:

- UI and graphic designers
- 3D artists
- motion designers
- game artists
- technical artists
- creators managing large reference libraries

The long-term direction combines:

- local asset management
- browser capture
- AI-assisted organization
- professional preview workflows
- future Blender, Houdini, and Unreal Engine workflows

## Architecture

Desktop application:

- Tauri
- Rust backend
- Vue + Vite frontend
- SQLite storage

Browser integration:

- Chromium Manifest V3 extension
- Local authenticated capture service

Preview stack:

- native image/video viewers
- SVG preview
- PDF preview
- Three.js based 3D preview path

AI stack:

- OpenAI-compatible providers
- Gemini providers
- Anthropic providers
- structured AI metadata suggestions
- AI review workflow

## Current branch and PR

Repository:

`diyuWF/lap`

Pull request:

`#2 feat: Phase 4–5 preview system and online AI automation`

Branch:

`feat/phase4-ai-preview-pipeline`

The PR is intentionally kept Draft until manual validation is complete.

## Current validation candidate

Remote candidate commit:

`8e4c41d559297c2864403f807d45cda68ac29362`

Automated validation:

- PASS — PR Build #68, run `30366546582`
- PASS — Chat Validation Package #13, run `30366545915`
- PASS — artifact `Lap-0.4.1-Chat-Delivery-13`
- Artifact ID: `8692268324`
- Artifact SHA256 digest:
  `ea12989aaaa9c4094cf9a5652fd01a346883a0224fd8cbfc9a3a3731f7288904`
- Artifact expires: 2026-08-27

The automated gate is complete. Stop feature development and perform the
manual checks in `docs/MANUAL_VALIDATION.md`. Keep PR #2 in Draft until those
checks pass.

## Completed milestones

### Phase 1–3

Completed:

- Simplified Chinese localization foundation
- DAM taxonomy
- source metadata
- workflow states
- AI suggestion storage
- browser capture service
- browser extension workflow

### Phase 4

Completed:

- unified preview routing
- SVG preview
- PDF preview
- GLB/glTF/OBJ/STL preview support
- design asset indexing
- unsupported-format fallback UI

### Phase 5

Completed:

- persistent online AI provider configuration
- OpenAI-compatible provider support
- Gemini provider support
- Anthropic provider support
- structured AI analysis
- confidence thresholds
- automatic tag application option
- AI suggestion review queue
- batch AI organization workflow

## Validation status

Automated validation has passed on the Phase 4–5 feature baseline:

- frontend production build
- strict Simplified Chinese audit
- Rust cargo check
- browser extension validation
- Windows x64 validation package build

Every source-alignment change must receive a fresh successful PR build and
Windows validation-package build before that package is used for manual testing.

Current state:

`FAILED — FOURTH FOCUSED VALIDATION FIX IMPLEMENTED`

Reason:

The third focused package (`Lap-0.4.1-Chat-Delivery-12`) passed its automated
checks. Manual retest then identified that the one-second drag threshold still
feels slow and that browser captures need a clear Inbox-to-AI-organization flow.

Focused fixes are implemented in the working branch:

- drag intent confirmation is now 500 ms;
- the radial center action is named “保存到待整理区域”;
- captured assets keep the existing `dam_file_workflow.status = inbox` workflow;
- AI can plan against either one selected root and its descendants or the whole
  existing folder hierarchy;
- AI output is limited to existing database folder IDs and relative display paths;
- folder plans are persisted and reviewed before execution;
- an internal executor revalidates the current asset and destination folder, then
  moves with the existing `keep_both` conflict policy;
- AI cannot invent folders, receive absolute paths, or move files directly.

Frontend, localization, extension syntax/resources, drag DOM regressions, and
repository hygiene pass locally. GitHub PR Build #68 also passed the Rust backend
check, and Chat Validation Package #13 produced the Windows/browser validation
artifact.

Do not start future phases. Return to manual validation with
`Lap-0.4.1-Chat-Delivery-13`.

## Manual validation priority

1. Install Windows build.
2. Confirm existing Lap libraries open correctly.
3. Test SVG/PDF preview.
4. Test GLB/glTF/OBJ/STL model loading.
5. Test browser extension capture into the pending area.
6. Configure a real AI provider.
7. Test single-file AI analysis.
8. Test both AI folder-planning scopes.
9. Review and execute AI folder plans.
10. Test AI review queue acceptance/rejection.

Record all results in:

`docs/MANUAL_VALIDATION.md`

## Known risks

### Validation package vs source

The sidebar cleanup is now represented directly in `src-vite/src/views/Home.vue`.
The Windows validation workflow no longer rewrites that source file during the
build, so the checked-in sidebar implementation and packaged implementation use
the same code.

### AI providers

Real API behavior depends on:

- provider availability
- vision model capability
- API limits
- user configuration

OpenRouter compatibility must accept both its root URL and `/api/v1` base URL,
and route both to `/api/v1/chat/completions`.

Free OpenRouter models may be temporarily limited by their upstream provider.
HTTP error messages must summarize safe provider/model details only; never render
the complete provider payload or internal user identifiers.

### Browser extension drag capture

The next package must be manually checked on a real Chromium page. Verify the
native semi-transparent drag image follows the pointer with no repeated tutorial
card, the radial folder menu appears after the 500 ms threshold, the center reads
“保存到待整理区域”, small/lazy-loaded thumbnails enter the same flow, dropping
on the center opens confirmation, and releasing early does not leave an overlay
behind.

### AI pending-area organization

The organizer has two explicit scopes:

- `within_folder`: a selected root folder and all of its existing descendants;
- `library`: all existing folders in the current library.

Inbox/待整理 folders are excluded as destinations. Folder candidates are capped
at 800 and a single batch is capped at 200 assets. Suggestions persist as folder
plans and must be confirmed before execution. The executor rejects missing assets,
deleted destinations, and stale or malformed suggestion IDs. It does not create
or delete folders.

### Browser extension setup

The capture service remains bound to `127.0.0.1:47821`. `/health` is intentionally
available without a Token so the extension can detect a running local Lap. All
folder and capture endpoints remain Token-protected; do not expose the Token
through an unauthenticated discovery endpoint.

The App now exposes the local address and a masked pairing Token under
`Settings → Advanced → Browser capture`, with explicit copy buttons. The
extension setup instructions must continue to point to that exact location.

### Localization

The language selector uses language autonyms. Locale changes must update both the
settings window and the main window immediately. New AI analysis, batch, review,
preview, 3D, and settings surfaces use the i18n catalog; locales without new
feature translations fall back to English rather than rendering hard-coded
Chinese.

### Professional formats

Some professional files may initially provide metadata/fallback behavior instead of full native rendering.

## Future backlog (not approved)

Possible future phases:

- Blender asset browser integration
- Houdini workflow integration
- Unreal Engine asset workflow
- PSD/AI/AE proxy previews
- material library management
- PureRef-style infinite canvas
- AI project classification

These are backlog items only.
Do not implement until the current validation gate is complete.

## Development philosophy

Prioritize:

1. stability
2. user data safety
3. backward compatibility
4. focused fixes
5. verified workflows

Avoid feature expansion before validation.
