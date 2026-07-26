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

Automated validation has passed:

- frontend production build
- strict Simplified Chinese audit
- Rust cargo check
- browser extension validation
- Windows x64 validation package build

Current state:

`BLOCKED`

Reason:

The remaining work requires real user validation on Windows and with real AI credentials.

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

The Windows validation workflow performs some build-time source transformations. Always compare workflow scripts against source files before assuming the packaged application exactly matches repository source.

### AI providers

Real API behavior depends on:

- provider availability
- vision model capability
- API limits
- user configuration

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