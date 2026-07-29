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

Latest packaged candidate commit:

`19050567b2159b5dcaa8ea23ceef7bde1ddc94a7`

Automated validation:

- PASS — PR Build #70
- PASS — Chat Validation Package #15
- PASS — artifact `Lap-0.4.1-Chat-Delivery-15`
- Artifact ID: `8719050322`
- Artifact SHA256 digest:
  `d701c41100cbd06c0df2d07bdc5818db223c5bcd130e6c51852b77103529ed0c`
- Windows installer SHA256:
  `8adab3f96bff507575156dcf0f7ca91347be40868d2ae4e88e2800b1e9acde0e`
- Chromium extension SHA256:
  `22b704b5f69dd1097fe378ebd4d3d9c2f8f44a2a0b04864290625cbaa9399e1d`

The browser-classification candidate is packaged and ready. The current visual
theme update requires a fresh PR build plus Windows validation package before
manual testing.
Keep PR #2 in Draft until the updated checks and manual validation pass.

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

`BLOCKED — SIGNATURE THEME CANDIDATE AWAITING FRESH PACKAGE`

Reason:

The browser classification package (`Lap-0.4.1-Chat-Delivery-15`) passed its
automated checks. The next focused change applies the selected compact,
near-black glass visual language to Lap's default dark theme and maps the same
hierarchy to a warm, translucent light theme.

Focused fixes are implemented in the working branch:

- drag intent confirmation is now 100 ms;
- the radial UI uses compact dark translucent directory cards and the official
  Lap icon;
- the center action is “AI 分类” and is hidden unless at least one enabled online
  AI provider has a stored API key;
- dropping on an existing directory saves immediately without the removed
  confirmation modal and marks the asset workflow as `selected`;
- AI-center captures retain the logical `inbox` workflow for the existing
  AI-planning and review process;
- every radial menu appends “创建目录”; dropping there opens an in-page parent
  directory/name form, creates one validated child directory, and immediately
  saves the current image;
- the authenticated localhost API now supports `POST /folders`; parent directories
  must already exist in Lap and invalid or Windows-reserved names are rejected;
- toolbar fallback copy no longer exposes “保存到待整理区域”; its “AI 分类” option
  is also conditional on the configured AI state.
- the default day/night pair is now `lap-light` and `lap-dark`;
- title bars, sidebars, content surfaces, settings cards, popovers, buttons,
  fields, and toggles share the same restrained glass hierarchy;
- browser preview safely stubs Tauri-only window/event calls so the real
  Settings screen can be visually checked without changing desktop behavior.

Browser-rendered Settings visual QA passes for both day and night themes at
1363 × 936 with no layout overflow or Lap console errors. The native Home,
viewer, and data-backed surfaces require the Windows Tauri package and remain
explicit manual checks. Local Rust tools may not be available in the Work Mode
container, so the fresh GitHub PR build remains the authoritative Rust check.

Do not start future phases. Generate a fresh package from the next pushed
candidate, then return to manual validation.

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
11. Switch between the default Lap light and dark themes and inspect Home,
    Settings, image viewer, dialogs, dropdowns, and dense asset grids.

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
card, the dark radial folder menu appears after the 100 ms threshold, the
“AI 分类” center is absent before an AI model is configured and visible after
configuration, small/lazy-loaded thumbnails enter the same flow, existing-folder
drops save without confirmation, and releasing early does not leave an overlay.
Also drop on “创建目录”, choose an existing parent, create a valid child directory,
and confirm the image is saved there without a second dialog.

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
