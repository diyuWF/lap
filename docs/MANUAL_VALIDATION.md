# Lap 0.4.1 Manual Validation Checklist

## Validation information

Tester: User

Date: 2026-07-26

Branch: `feat/phase4-ai-preview-pipeline`

Manual feedback baseline commit: `d541af4f97cea3cd069ece8139324cb18b11d809`

Manual feedback baseline package: `Lap-0.4.1-Chat-Delivery-12`

Current candidate commit: `8e4c41d559297c2864403f807d45cda68ac29362`

Current candidate package: `Lap-0.4.1-Chat-Delivery-13`

Automated gate:

- PASS — PR Build #68 (`30366546582`)
- PASS — Chat Validation Package #13 (`30366545915`)
- Artifact ID: `8692268324`
- Artifact SHA256:
  `ea12989aaaa9c4094cf9a5652fd01a346883a0224fd8cbfc9a3a3731f7288904`

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

Result: FAILED — fourth focused rework implemented; awaiting a fresh package

Evidence:

- The extension settings page finds the capture service only after manually testing a copied address and Token.
- With an invalid Token, the page shows raw English `Invalid pairing token`.
- In Windows light mode, labels and default-rule controls have insufficient contrast.
- Dragging an image immediately opens a large destination panel and uses a pointer card. This does not provide a clear intent threshold and is not the requested image-following interaction.
- The 1.3-second “continue dragging” tutorial appears on every drag and makes the action feel slow.
- Small/lazy-loaded thumbnails do not reliably begin drag capture.
- The extension points to an App Token location that does not exist in the current Advanced settings screen.
- The extension does not use the official Lap icon.

Issues:

- The setup flow needs a one-click local Lap detector that distinguishes “Lap not running”, “Lap found but Token missing”, and “Lap found but Token invalid”.
- Light-theme labels, placeholders, status messages, and controls need accessible contrast.
- The dragged image should become semi-transparent and follow the pointer. After a 0.5-second hold, a radial folder menu should appear without a repeated tutorial card.
- The radial center action should read “保存到待整理区域” and save into the logical pending workflow.
- Dropping on a radial folder should lead to explicit save confirmation; “More folders” should open the complete folder browser.
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
| LAP-VAL-005 | Drag capture opens the destination UI without a clear intent threshold; expected behavior is a semi-transparent pointer-following image followed by a delayed radial folder menu. | High | Candidate 13 built successfully; awaiting manual retest |
| LAP-VAL-006 | The 1.3-second tutorial appears on every drag, feels slow, and small/lazy-loaded thumbnails do not reliably enter drag capture. | High | Candidate 13 built successfully; awaiting manual retest |
| LAP-VAL-007 | Language names are shown in Chinese, the main window does not switch locale immediately, and new AI/preview surfaces stay hard-coded in Chinese. | High | Candidate 13 built successfully; awaiting manual retest |
| LAP-VAL-008 | Extension setup points to a missing App Token screen, and the extension does not use the official Lap icon. | High | Candidate 13 built successfully; awaiting manual retest |
| LAP-VAL-009 | The one-second drag threshold still feels slow and the center action uses the ambiguous “保存到 Lap” label. | High | Candidate 13 built successfully with the 500 ms threshold and “保存到待整理区域”; awaiting manual retest |
| LAP-VAL-010 | Pending assets cannot be safely planned into existing nested folders and executed after user confirmation. | High | Candidate 13 built successfully with scoped plans and the validating executor; awaiting manual retest |

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
