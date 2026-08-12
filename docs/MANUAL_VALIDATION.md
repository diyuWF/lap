# Lap 0.4.1 Manual Validation Checklist

## Validation information

Tester: User

Date: 2026-07-29

Branch: `feat/phase4-ai-preview-pipeline`

Manual feedback baseline commit: `d541af4f97cea3cd069ece8139324cb18b11d809`

Manual feedback baseline package: `Lap-0.4.1-Chat-Delivery-12`

Latest packaged candidate commit: `00d13d5c0d1b42e047531ea7b554685bc9824d88`

Latest packaged candidate: `Lap-0.4.1-Chat-Delivery-23`

Next implementation commit: pending hierarchical AI-radial correction

Next candidate package: pending fresh GitHub build

Automated gate:

- PASS — PR Build #78
- PASS — Chat Validation Package #23

Environment: Windows desktop application and Chromium browser extension; Pinterest image detail page

## 1. Installation and startup

- [ ] Windows installer completes successfully
- [ ] Application launches
- [ ] Existing Lap library opens
- [ ] Existing assets remain visible
- [ ] Restart preserves settings
- [ ] No database migration errors

Result:

Evidence:

Issues:

## 2. SVG preview

- [ ] SVG is indexed
- [ ] SVG opens in preview
- [ ] Transparency is correct
- [ ] Large SVG remains responsive

Result:

Evidence:

Issues:

## 3. PDF preview

- [ ] PDF is indexed
- [ ] PDF preview opens
- [ ] Multi-page documents work
- [ ] Zoom works
- [ ] Non-English filenames work

Result:

Evidence:

Issues:

## 4. 3D preview

Formats:

- [ ] GLB
- [ ] glTF
- [ ] OBJ
- [ ] STL

Controls:

- [ ] Orbit rotation
- [ ] Zoom
- [ ] Pan
- [ ] Reset camera
- [ ] Wireframe mode
- [ ] Switching models releases previous resources
- [ ] Invalid model produces useful error

Result:

Evidence:

Issues:

## 5. Browser extension

Sites tested:

- [ ] Normal webpage
- [x] Pinterest
- [ ] Behance
- [ ] ArtStation

Functions:

- [ ] Extension loads
- [ ] Local Lap connection works
- [ ] Folder list loads
- [ ] Single image capture works
- [ ] Batch image discovery works
- [ ] Destination folder selection works
- [ ] Source URL saved
- [ ] Page title saved
- [ ] Duplicate handling works

Result: FAILED — hierarchical AI-radial correction implemented; awaiting a fresh package

Evidence:

- The extension settings page finds the capture service only after manually testing a copied address and Token.
- With an invalid Token, the page shows raw English `Invalid pairing token`.
- In Windows light mode, labels and default-rule controls have insufficient contrast.
- Dragging an image immediately opens a large destination panel and uses a pointer card. This does not provide a clear intent threshold and is not the requested image-following interaction.
- Earlier drag thresholds and the repeated tutorial made the action feel slow.
- Small/lazy-loaded thumbnails do not reliably begin drag capture.
- The extension points to an App Token location that does not exist in the current Advanced settings screen.
- The extension does not use the official Lap icon.

Issues:

- The setup flow needs a one-click local Lap detector that distinguishes “Lap not running”, “Lap found but Token missing”, and “Lap found but Token invalid”.
- Light-theme labels, placeholders, status messages, and controls need accessible contrast.
- The dragged image should become semi-transparent and follow the pointer with
  no tutorial rail, timer, or progress bar before activation.
- Total pointer-path length must accumulate even when the cursor returns toward
  the image; the threshold is one third of browser width, not current
  displacement or the shorter viewport side.
- After the threshold, one instrument-like circular menu must dim/blur the page
  and stay fixed at the exact viewport center.
- With no configured provider the AI center must be hidden. With an enabled
  provider the center contains only the dedicated Lap AI mark and “AI 分类”, and
  a drop there must immediately queue background analysis.
- Folder destinations and “创建目录” must be circular controls distributed around
  the inner orbit according to the available destination count.
- The first level must contain only root folders. A parent activation must replace
  that level with only its direct children plus “返回上级” and “创建目录”; parents,
  descendants from deeper levels, and unrelated roots must never be mixed.
- Empty folders must show a folder icon; populated folders must show the first
  image as a circular thumbnail and fall back safely if the thumbnail is missing.
- Dropping on an existing radial folder should save immediately with no
  confirmation dialog; “更多” should open the complete folder browser.
- Every radial menu should append “创建目录”; dropping there should open an
  in-page parent/name form, create a real child directory, and immediately save
  the current image.
- Small images, `draggable=false` thumbnails, lazy-loaded image sources, and images under a covering element need the same explicit drag flow.
- `Settings → Advanced → Browser capture` must expose the local address and pairing Token that the extension asks users to copy.
- The extension Manifest, popup, options page, and in-page save panel should use the official Lap icon.

## 5.1 Localization

Functions:

- [ ] Language names are written in their own language
- [ ] Changing language updates the Settings window immediately
- [ ] Changing language updates the main window immediately
- [ ] Online AI, review, batch, preview, and 3D surfaces do not remain Chinese under a non-Chinese locale

Result: FAILED — localization fixes implemented; awaiting a fresh package

Evidence:

- The language selector displayed every language name in Chinese.
- The Settings window changed its local i18n state, but the main window event only updated persisted configuration and did not update the global i18n locale.
- Newly added AI analysis, review, batch, preview, 3D, settings, and manager surfaces contained hard-coded Chinese strings.

Issues:

- Use autonyms such as `English`, `Deutsch`, `Español`, `日本語`, and `한국어`.
- Synchronize the global i18n locale when the persisted language changes.
- Move newly added AI and preview text into the locale catalogs; non-Chinese locales must not fall back to hard-coded Chinese.

## 6. Online AI

Providers tested:

- [x] OpenAI-compatible
- [ ] Gemini
- [ ] Anthropic

Functions:

- [x] Provider saves
- [ ] API key is protected
- [ ] Connection test works
- [ ] Single-file analysis works
- [ ] Batch analysis works
- [ ] Suggestions enter review queue
- [ ] Accept applies tags
- [ ] Reject does not apply tags
- [ ] Restart preserves configuration
- [ ] Pending assets can generate folder plans
- [ ] Scope can be limited to one selected root and its descendants
- [ ] Scope can include the whole existing folder hierarchy
- [ ] Folder plans require confirmation before files move
- [ ] Confirmed plans move files with conflict-safe naming
- [ ] Missing/deleted destinations are rejected without data loss

Result: BLOCKED — automated package passed; awaiting manual provider retest

Evidence:

- OpenRouter configured with `https://openrouter.ai` now requests `https://openrouter.ai/api/v1/chat/completions`.
- The provider returned JSON `HTTP 429` from Google AI Studio for `google/gemma-4-31b-it:free`, confirming that the request reached the correct API.
- Lap displayed the complete upstream JSON payload, including an internal `user_id`, instead of a concise, safe explanation.

Issues:

- Upstream rate limits need a provider-aware Chinese message with retry, own-key, and model-switch guidance.
- Raw provider payloads and internal identifiers must not be shown in the UI.

## Final status

- [ ] PASS — ready for merge
- [ ] FAILED — defects require fixes
- [x] BLOCKED — automated gate passed; waiting for manual validation

## Defect log

| ID | Description | Severity | Status |
|---|---|---|---|
| LAP-VAL-001 | OpenRouter root URL reached website HTML instead of the OpenAI-compatible JSON API. | High | Retest PASS — correct endpoint returned provider JSON |
| LAP-VAL-002 | Browser drag capture lacked feedback and the overlay stylesheet was truncated. | High | Partial retest PASS; interaction design superseded by LAP-VAL-005 |
| LAP-VAL-003 | HTTP 429 shows raw upstream JSON and an internal user identifier instead of a safe, actionable Chinese rate-limit message. | High | Candidate 13 built successfully; awaiting manual retest |
| LAP-VAL-004 | Extension setup lacks local auto-detection, exposes a raw English Token error, and has insufficient light-theme contrast. | High | Candidate 13 built successfully; awaiting manual retest |
| LAP-VAL-005 | Drag capture opens the destination UI without a clear intent threshold; expected behavior is a semi-transparent pointer-following image followed by a delayed radial folder menu. | High | Superseded by the 100 ms candidate in LAP-VAL-011 |
| LAP-VAL-006 | The 1.3-second tutorial appears on every drag, feels slow, and small/lazy-loaded thumbnails do not reliably enter drag capture. | High | Candidate 14 built successfully; 100 ms follow-up awaiting package |
| LAP-VAL-007 | Language names are shown in Chinese, the main window does not switch locale immediately, and new AI/preview surfaces stay hard-coded in Chinese. | High | Candidate 13 built successfully; awaiting manual retest |
| LAP-VAL-008 | Extension setup points to a missing App Token screen, and the extension does not use the official Lap icon. | High | Candidate 13 built successfully; awaiting manual retest |
| LAP-VAL-009 | The one-second drag threshold still feels slow and the center action uses the ambiguous “保存到 Lap” label. | High | Superseded by the conditional “AI 分类” flow in LAP-VAL-011 |
| LAP-VAL-010 | Pending assets cannot be safely planned into existing nested folders and executed after user confirmation. | High | Candidate 13 built successfully with scoped plans and the validating executor; awaiting manual retest |
| LAP-VAL-011 | The 500 ms radial still feels slow; the light circular visual, unconditional pending center, and post-drop confirmation do not match the requested direct-save/conditional-AI flow. | High | Candidate 15 built successfully; awaiting manual retest |
| LAP-VAL-012 | The desktop application's default day/night themes do not yet use the selected compact charcoal-glass visual language and its warm light counterpart. | Medium | Implemented in the working branch; awaiting fresh package and manual validation |
| LAP-VAL-013 | The 100 ms timer does not reflect drag intent; the interaction needs a centered animated rail, distance activation, page dim/blur, a larger fixed-center radial, and a restored AI center. | High | Implemented in the working branch; awaiting fresh package and manual validation |
| LAP-VAL-014 | Dragging images outside Lap cannot yet create a persistent, always-on-top, pan/zoom reference board. | High | First authorized reference-board slice implemented; awaiting Windows validation |
| LAP-VAL-015 | Browser intent used displacement from the starting point, so returning toward the image reduced progress; the progress rail was also visually intrusive. | High | Replaced by cumulative path length, a width/3 threshold, and no pre-threshold UI; awaiting fresh package |
| LAP-VAL-016 | Lap intercepted every boundary drag into the reference board, blocking normal drag-out to Photoshop; drops were recentered and close/recreate restored stale layout. | High | Replaced by first-use choice, native OS drag, real drop-point placement, and close reset; awaiting Windows package |
| LAP-VAL-017 | The centered browser radial rendered as an oversized opaque black disc with undersized folder targets, leaving a large visual gap from the selected circular-control reference. | High | Replaced by a transparent 820 px positioning ring, 118 px count-aware folder targets, and a 164 px conditional AI center; browser visual QA passed, awaiting packaged manual validation |
| LAP-VAL-018 | The transparent-ring candidate still did not follow the selected precision-instrument reference and could not visually distinguish populated folders from empty folders. | High | Replaced by an instrument tick/orbit/bronze-arc dial, full-orbit circular controls, and authenticated first-image folder covers; cloud-browser visual QA passed, awaiting packaged manual validation |
| LAP-VAL-019 | The AI center lacked a dedicated mark and did not start classification directly; root and descendant folders were rendered together instead of opening one hierarchy level at a time. | High | Replaced by an original AI center asset, background AI queueing, and root/direct-child radial navigation; cloud-browser interaction and visual QA passed, awaiting packaged manual validation |

## 5.2 Browser radial classification follow-up

Drag intent:

- [ ] Native semi-transparent image follows the pointer
- [ ] No tutorial, timer, progress rail, or other overlay appears before threshold
- [ ] Moving forward and then backward keeps adding to total path length
- [ ] Total travel below one third of browser width does not open the radial
- [ ] Total travel at one third of browser width opens the radial
- [ ] Page dim/blur begins only when the radial appears
- [ ] The 820 px precision dial remains centered while the pointer moves
- [ ] The dial shows one crisp outer tick ring, one thin inner orbit, and one bronze partial arc
- [ ] Circular destination controls are distributed evenly around the inner orbit
- [ ] Empty folders show the bundled folder icon
- [ ] Populated folders show the first image as a circular thumbnail
- [ ] A missing or unreadable cover falls back to the folder icon
- [ ] Folder labels remain readable and circular targets do not overlap
- [ ] The first level contains top-level folders only
- [ ] A child folder is absent until its parent is activated
- [ ] Dropping on or dwelling over a parent opens only its direct children
- [ ] Parent and unrelated root siblings are absent from a child level
- [ ] “返回上级” restores the immediately preceding level
- [ ] Deeper descendants appear only after activating their direct parent

Without an AI provider configured:

- [ ] Center “AI 分类” is absent
- [ ] Existing directory targets remain usable
- [ ] “创建目录” is present

With an enabled AI provider and stored API key:

- [ ] Center “AI 分类” card appears
- [ ] Center contains the dedicated Lap AI mark and “AI 分类”
- [ ] Dropping on “AI 分类” saves with workflow status `inbox`
- [ ] The capture request sets `autoClassify: true`
- [ ] Lap immediately starts background analysis with the enabled keyed provider
- [ ] AI produces tag metadata and, when destinations exist, an existing-folder suggestion
- [ ] The AI center does not move the file without the existing review/confirmation flow
- [ ] Dropping on an existing directory saves immediately with status `selected`
- [ ] No old confirmation modal appears after either drop

Create-directory path:

- [ ] Drop on “创建目录”
- [ ] In-page form lists existing Lap directories as parents
- [ ] Invalid names and duplicate child names show a safe error
- [ ] A valid child directory is created in the selected parent
- [ ] Current image is immediately saved into the new child directory
- [ ] No second save confirmation appears

Result:

Evidence: Cloud-browser QA passed at 1363 × 936 using the production extension
code. The initial state contained only `lap资源`, `风景摄影`, and “创建目录”.
Activating `lap资源` replaced both roots with only “返回上级”, child `1`, and
“创建目录”; activating `1` then showed only its direct children `三渲二` and
`写实`. The AI center remained visible with the original AI mark. A real center
drop sent `workflowStatus: inbox` plus `autoClassify: true` and returned the
background-classification success state. Packaged Chromium validation is still
required.

Issues:

## 5.3 Desktop reference board

Window creation:

- [ ] First image drag to a Lap boundary asks whether to create a reference board
- [ ] Choosing create opens one empty separate reference board
- [ ] Choosing another app does not create or populate a board
- [ ] With a board open, dragging to Photoshop creates a normal Photoshop drop
- [ ] With a board open, only dropping inside the board adds the image
- [ ] Multiple selected images can be dropped into the board together
- [ ] The drag-out does not also move/copy files inside the Lap library
- [ ] Non-image files do not open the reference board

Board behavior:

- [ ] Reference board is frameless and stays above normal windows by default
- [ ] Pin control disables and re-enables always-on-top
- [ ] Dragging empty space pans the canvas
- [ ] Mouse wheel zooms around the pointer
- [ ] Zoom reaches both very small and very large scales without breaking
- [ ] Fit All frames every image
- [ ] 100% centers the selected image at actual board size
- [ ] Images can be rearranged independently
- [ ] A newly dropped image appears at the actual drop location, not the center
- [ ] Delete/Backspace removes only the selected board item
- [ ] Native file drop adds more images to the open board
- [ ] Closing and recreating starts with an empty board and reset camera
- [ ] Minimize and close controls work

Result: BLOCKED — fresh Windows package required

Evidence:

- Browser-rendered board passed its existing pan/zoom surface at 1363 × 936.
- Native system drag-out, the first-use dialog, Photoshop coexistence, physical
  drop coordinates, and close reset require the packaged Windows application.

Issues:

## 7. AI pending-area organization

Test setup:

- [ ] Create `游戏原画`
- [ ] Under it create `三渲二`, `写实`, `中国风`, and `欧美风`
- [ ] Create a separate top-level folder `风景摄影`
- [ ] Capture several mixed images into the pending area

Selected-root mode:

- [ ] Select `游戏原画`
- [ ] Generate plans
- [ ] Every proposed destination is `游戏原画` or one of its descendants
- [ ] `风景摄影` is never proposed in this mode

Whole-library mode:

- [ ] Generate plans without a selected root
- [ ] AI can select suitable existing folders under either top-level folder
- [ ] AI cannot invent a folder that does not exist
- [ ] Inbox/待整理 is not offered as a destination

Review and execution:

- [ ] Plans show destination, confidence, and reason
- [ ] No file moves before explicit confirmation
- [ ] Confirmed plans move to the selected existing folder
- [ ] Name conflicts keep both files
- [ ] Delete a proposed destination before execution and confirm the item fails safely
- [ ] Restart preserves unexecuted plans

Result:

Evidence:

Issues:

## 8. Lap signature day/night themes

Dark theme:

- [ ] Default Dark + Default Theme resolves to the new Lap dark visual system
- [ ] Main canvas is near-black without crushing image or text contrast
- [ ] Title bar, sidebar, toolbar, cards, popovers, fields, buttons, and toggles
  use consistent charcoal/glass surfaces and subtle borders
- [ ] Purple/amber accent glow stays restrained and does not distract from assets

Light theme:

- [ ] Default Light + Default Theme resolves to the new Lap light visual system
- [ ] Canvas is warm off-white/gray rather than harsh pure white
- [ ] Translucent white cards remain distinct without washed-out labels
- [ ] Primary actions and selected navigation retain the shared purple accent

Coverage:

- [ ] Home library view
- [ ] Settings General and Advanced
- [ ] Image viewer
- [ ] Dialogs and dropdown menus
- [ ] Empty, loading, disabled, selected, hover, and focus states
- [ ] Dense image grids at 100% and 125% Windows scaling
- [ ] Theme selection persists after restart

Result: BLOCKED — fresh packaged candidate required

Evidence:

- Browser-rendered Settings passed at 1363 × 936 in both `lap-dark` and
  `lap-light`, with no overflow and no Lap application console errors.
- Native Home/viewer/data-backed screens require the Windows Tauri package.

Issues:
