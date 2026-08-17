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

`4eefde94d81ddbe7a745bf82b4c672e15ed5cafb`

Automated validation:

- PASS — PR Build #82
- PASS — Chat Validation Package #27
- PASS — artifact `Lap-0.4.1-Chat-Delivery-27`

That package contains the cumulative-path browser capture, native reference-board
drag-out correction, instrument-control radial, real folder-cover data path,
the restored complete SQLite source, hierarchical AI-radial navigation,
AI-classification collection, silent background capture, first-install Token
pairing, inner-orbit target placement, and the moving purple AI mark. Manual
feedback found that the native drag preview could visibly cover a parent while
its pointer hotspot remained outside the parent's circle, so child folders still
did not open. The remaining bronze accents and visible center label were also
rejected. The focused working-tree correction requires a fresh PR build plus
Windows package.
The Windows chat-delivery workflow now runs for every opened, synchronized, or
reopened PR update so each focused modification produces a matching installer
and browser-extension package automatically.
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

`BLOCKED — DRAG-PREVIEW HIERARCHY/PURPLE RADIAL FOLLOW-UP AWAITING FRESH PACKAGE`

Reason:

`Lap-0.4.1-Chat-Delivery-27` passed its automated checks and resolved the prior
orbit intersection and AI-mark defects. Manual feedback confirmed one native
Chromium interaction gap: the visible drag preview can cover a parent folder
while the pointer hotspot stays outside the circular hit area, so the child level
does not open. The instrument accents also need to be purple and the center must
show only the moving AI logo without a visible text label.

Focused fixes are implemented in the working branch:

- browser intent accumulates every pointer-path segment, so reversing direction
  never reduces intent; the threshold is exactly one third of browser width;
- there is no tutorial rail, timer, or progress bar before activation; the
  semi-transparent native drag image is the only pre-threshold feedback;
- after activation, the page is moderately dimmed/blurred and one 820 px
  precision-instrument dial is fixed to the exact viewport center;
- the dial uses the selected source hierarchy: a full radial tick ring, one thin
  inner orbit, and one restrained purple partial arc rather than a solid disc;
- the center always uses an original transparent purple AI wordmark plus two
  independently moving, counter-rotating particle-halo layers with no visible
  label; its accessible name remains “AI 分类”. Reduced-motion mode stops those
  animations, while a drop silently enters the logical AI classification queue
  with `autoClassify: false` and never calls a model during browser capture;
- the first radial level contains top-level folders only; descendants and recent
  paths can no longer leak into that level;
- dropping on, clicking, or dwelling 220 ms with the dragged preview over a
  parent opens only that parent's direct children; dwell detection combines the
  drag event target, document hit testing, viewport-centered radial geometry,
  and dragged-image rectangle/circle intersection with a 140 ms miss grace
  period, so an offset native hotspot cannot lose the visibly covered parent;
  a post-navigation pointer lock prevents stationary-cursor bounce;
- the custom semi-transparent drag preview is staged inside the viewport before
  `setDragImage`, uses a centered hotspot, and is then moved off-screen after the
  browser snapshots it; this prevents Chromium from falling back to an oversized
  source-page preview;
- every child level contains a dedicated “保存到 {current folder}” target in
  addition to its direct children, “返回上级”, and “创建目录”;
- nested levels never render their parent or unrelated root siblings alongside
  the children; “创建目录” remains available and preselects the current parent;
- visible folders plus “创建目录” are circular controls distributed evenly and
  completely inside the inner orbit; their size/capacity adapts to the viewport,
  keeps a visible stroke gap, prevents target intersections, and retains the
  complete folder browser behind “更多” when a level exceeds the safe capacity;
- empty destination folders show the bundled folder icon; populated folders ask
  the authenticated local service for the first image and show its thumbnail as
  a circular cover, falling back safely to the icon if decoding fails;
- the new Token-protected `POST /folder-covers` endpoint returns only requested
  folder IDs and 256 px data-URL thumbnails; the extension background worker
  performs that request so the pairing Token is never exposed to page scripts;
- each cover image is rendered inside a closed Shadow DOM owned by the isolated
  content script, preventing the host page from reading the local image data URL;
- dropping on an existing directory saves immediately without the removed
  confirmation modal and marks the asset workflow as `selected`;
- AI-center captures retain the logical `inbox` workflow and set
  `autoClassify: false`; collected assets remain visible until the user opens the
  desktop “AI 分类” workspace and starts one-click plan generation;
- the old “智能相册” navigation slot is now the AI classification collection;
  it shows queued count, candidate thumbnails, configured provider/scope, and
  the existing review-before-move plan executor directly in the main stage;
- candidate assets are preselected when the workspace opens, so its one-click
  action analyzes the current queue while folder moves still require explicit
  confirmation;
- automatic classification never silently moves the asset: folder output remains
  a reviewable suggestion and existing provider confidence/auto-tag rules stay in
  force;
- every radial menu appends “创建目录”; dropping there opens an in-page parent
  directory/name form, creates one validated child directory, and immediately
  saves the current image;
- the authenticated localhost API now supports `POST /folders`; parent directories
  must already exist in Lap and invalid or Windows-reserved names are rejected;
- ordinary folder and AI-center drops remove the overlay immediately and hand
  the request to the extension service worker; no “正在保存” or success toast is
  shown in the page. Failures are logged without interrupting the webpage.
- the local capture service reuses a pooled HTTP client and performs the final
  file write asynchronously, reducing repeated-connection and blocking-I/O cost.
- a first-time extension install immediately opens the existing setup page so
  local Lap detection and Token pairing are the first visible task; updates do
  not reopen it.
- a committed six-test Happy DOM regression suite covers the always-visible AI
  center and generated assets, inbox/auto-classify payload, geometry-fallback
  root-to-child dwell, stationary-pointer lock, desktop/compact no-overlap orbit
  layouts, direct current-folder save, and first-install setup behavior; both PR
  and package jobs run this suite.
- the default day/night pair is now `lap-light` and `lap-dark`;
- title bars, sidebars, content surfaces, settings cards, popovers, buttons,
  fields, and toggles share the same restrained glass hierarchy;
- browser preview safely stubs Tauri-only window/event calls so the real
  Settings screen can be visually checked without changing desktop behavior.
- the first image drag beyond Lap asks whether to create a reference board;
- after that choice, Lap starts a native OS file drag instead of intercepting
  the gesture, so Photoshop and other applications remain valid destinations;
- an open reference board receives an image only from a real native drop inside
  its own window; no edge-crossing event adds assets automatically;
- native drops use the real pointer location as their placement anchor and
  natural image sizing preserves that anchor instead of recentering the item;
- closing the reference board clears its items, camera, and persisted layout so
  a later newly created board starts empty;
- the reference board supports blank-space panning, pointer-centered wheel
  zoom from 5% to 3200%, Fit All, 100%, image arrangement/removal, and additional
  native file drops while the window remains open.

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
5. Test the browser extension distance-based radial capture.
6. Configure a real AI provider.
7. Test single-file AI analysis.
8. Test both AI folder-planning scopes.
9. Review and execute AI folder plans.
10. Test AI review queue acceptance/rejection.
11. Drag images out of Lap into the reference board and test pan, zoom, arrange,
    additional drops, persistence, and always-on-top behavior.
12. Switch between the default Lap light and dark themes and inspect Home,
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
native semi-transparent drag image follows the pointer with no tutorial or
progress surface. Moving back toward the start must keep accumulating path
length, and the circular UI must appear only after total travel reaches roughly
one third of browser width. Verify the page is dimmed/blurred only after that
threshold and the precision dial remains fixed at the viewport center. Its tick
ring, inner orbit, purple arc, and circular directory controls must remain crisp.
Every directory/create target must sit wholly inside the inner orbit with a
visible gap and must not intersect another target. The AI center must show only
the original purple AI wordmark and a slowly drifting/counter-rotating particle
halo; “AI 分类” remains its accessible and desktop-workspace name but is not
visible inside the radial.
It must silently collect with `workflowStatus: inbox` and `autoClassify: false`;
no provider call or page-level saving toast may occur. The first level must show
only roots; dwelling with the dragged image visibly overlapping a parent during
a real native drag must show its direct children even when the pointer hotspot
itself is outside the circle, plus an explicit
save-to-current-parent action and navigation. A stationary cursor must not make
the new level bounce back; the pointer must move before another dwell begins.
Empty
folders must show the folder icon, populated folders must show their first image
as a circular cover, and unavailable covers must fall back to the icon. Small or
lazy-loaded thumbnails must enter the same flow, existing-folder drops must save
without confirmation, and releasing early must not leave an overlay.
Also drop on “创建目录”, choose an existing parent, create a valid child directory,
and confirm the image is saved there without a second dialog.

### Desktop reference board

The reference board is a focused first slice, not full PureRef parity. Native
Windows validation must verify that the first boundary drag asks whether to
create the board, and that a later drag becomes a normal OS file drag accepted
by both Photoshop and the board. The board must add only files actually dropped
inside it and place them at the drop location. Confirm always-on-top, frameless
window controls, pan, 5%–3200% zoom, Fit All, 100%, image movement, Delete
removal, external file drops, and an empty board after close/recreate.

### AI classification collection and organization

The organizer has two explicit scopes:

- `within_folder`: a selected root folder and all of its existing descendants;
- `library`: all existing folders in the current library.

The logical AI classification queue is excluded as a destination. Folder candidates are capped
at 800 and a single batch is capped at 200 assets. Suggestions persist as folder
plans and must be confirmed before execution. The executor rejects missing assets,
deleted destinations, and stale or malformed suggestion IDs. It does not create
or delete folders.

### Browser extension setup

The capture service remains bound to `127.0.0.1:47821`. `/health` is intentionally
available without a Token so the extension can detect a running local Lap. All
folder and capture endpoints remain Token-protected; do not expose the Token
through an unauthenticated discovery endpoint.

On the browser's first `install` event, the extension opens its options page
automatically. The `update` event must never reopen that page.

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
