# Lap 0.4.1 Manual Validation Checklist

## Validation information

Tester: User

Date: 2026-07-26

Branch: `feat/phase4-ai-preview-pipeline`

Manual feedback baseline commit: `a1ab547aafa9845432d2384e5f0e0e38a3722ca0`

Manual feedback baseline package: `Lap-0.4.1-Chat-Delivery-10`

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

Result: FAILED — third focused rework implemented; awaiting a fresh package

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
- The dragged image should become semi-transparent and follow the pointer. After a one-second hold, a radial folder menu should appear without a repeated tutorial card.
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

Result: FAILED — endpoint retest passed; rate-limit UX rework implemented; awaiting a fresh package

Evidence:

- OpenRouter configured with `https://openrouter.ai` now requests `https://openrouter.ai/api/v1/chat/completions`.
- The provider returned JSON `HTTP 429` from Google AI Studio for `google/gemma-4-31b-it:free`, confirming that the request reached the correct API.
- Lap displayed the complete upstream JSON payload, including an internal `user_id`, instead of a concise, safe explanation.

Issues:

- Upstream rate limits need a provider-aware Chinese message with retry, own-key, and model-switch guidance.
- Raw provider payloads and internal identifiers must not be shown in the UI.

## Final status

- [ ] PASS — ready for merge
- [x] FAILED — defects require fixes
- [ ] BLOCKED — waiting for environment, credentials, or decision

## Defect log

| ID | Description | Severity | Status |
|---|---|---|---|
| LAP-VAL-001 | OpenRouter root URL reached website HTML instead of the OpenAI-compatible JSON API. | High | Retest PASS — correct endpoint returned provider JSON |
| LAP-VAL-002 | Browser drag capture lacked feedback and the overlay stylesheet was truncated. | High | Partial retest PASS; interaction design superseded by LAP-VAL-005 |
| LAP-VAL-003 | HTTP 429 shows raw upstream JSON and an internal user identifier instead of a safe, actionable Chinese rate-limit message. | High | Fix implemented; awaiting automated and manual retest |
| LAP-VAL-004 | Extension setup lacks local auto-detection, exposes a raw English Token error, and has insufficient light-theme contrast. | High | Fix implemented; awaiting automated and manual retest |
| LAP-VAL-005 | Drag capture opens the destination UI without a clear intent threshold; expected behavior is a semi-transparent pointer-following image followed by a delayed radial folder menu. | High | Fix implemented; awaiting automated and manual retest |
| LAP-VAL-006 | The 1.3-second tutorial appears on every drag, feels slow, and small/lazy-loaded thumbnails do not reliably enter drag capture. | High | Fix implemented with a silent one-second threshold and thumbnail source/drag recovery; awaiting automated and manual retest |
| LAP-VAL-007 | Language names are shown in Chinese, the main window does not switch locale immediately, and new AI/preview surfaces stay hard-coded in Chinese. | High | Fix implemented; awaiting automated and manual retest |
| LAP-VAL-008 | Extension setup points to a missing App Token screen, and the extension does not use the official Lap icon. | High | Fix implemented; awaiting automated and manual retest |
