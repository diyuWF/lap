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

`FAILED — THIRD FOCUSED VALIDATION FIX IMPLEMENTED`

Reason:

The second focused package (`Lap-0.4.1-Chat-Delivery-11`) passed its automated
checks. Manual retest then identified four usability and source-alignment defects:

- The 1.3-second drag tutorial is repeated for every capture and makes an ordinary
  action feel slower. The only pre-menu feedback should be the semi-transparent
  image, with a one-second threshold.
- Small or lazy-loaded thumbnails may use `draggable=false`, a covering element,
  a data URI, or a deferred source attribute and therefore never enter the capture
  flow.
- The extension tells users to copy the pairing Token from
  `Settings → Advanced → Browser capture`, but the App does not expose that UI.
- Language names are shown in Chinese, the main window does not update its locale
  immediately, and the new AI/preview settings and dialogs are hard-coded in
  Chinese. The extension also lacks the official Lap icon.

Focused fixes are implemented in the working branch. Frontend, localization,
extension syntax/resources, drag/setup DOM regressions, and repository hygiene
pass locally. Local Rust checks are blocked because this workstation has no
Cargo/rustfmt; a fresh GitHub PR build and Windows validation package are still
required.

Do not start future phases. Produce a fresh Windows validation package and
return to manual validation.

## Manual validation priority

1. Install Windows build.
2. Confirm existing Lap libraries open correctly.
3. Test SVG/PDF preview.
4. Test GLB/glTF/OBJ/STL model loading.
5. Test browser extension capture workflow.
6. Configure a real AI provider.
7. Test single-file AI analysis.
8. Test batch AI analysis.
9. Test AI review queue acceptance/rejection.

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
card, the radial folder menu does not appear before the one-second threshold,
small/lazy-loaded thumbnails enter the same flow, folder drop opens confirmation,
and releasing early does not leave an overlay behind.

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
