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

Result: FAILED — focused rework implemented; awaiting a fresh package

Evidence:

- The extension settings page finds the capture service only after manually testing a copied address and Token.
- With an invalid Token, the page shows raw English `Invalid pairing token`.
- In Windows light mode, labels and default-rule controls have insufficient contrast.
- Dragging an image immediately opens a large destination panel and uses a pointer card. This does not provide a clear intent threshold and is not the requested image-following interaction.

Issues:

- The setup flow needs a one-click local Lap detector that distinguishes “Lap not running”, “Lap found but Token missing”, and “Lap found but Token invalid”.
- Light-theme labels, placeholders, status messages, and controls need accessible contrast.
- The dragged image should become semi-transparent and follow the pointer. Only after a 1–2 second hold should a radial folder menu appear.
- Dropping on a radial folder should lead to explicit save confirmation; “More folders” should open the complete folder browser.

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
