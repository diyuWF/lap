# Lap 0.4.1 Manual Validation Checklist

## Validation information

Tester: User

Date: 2026-07-26

Branch: `feat/phase4-ai-preview-pipeline`

Commit: `f8b84f002dda25f0864686d411bd69864e63cb29`

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

Result: FAILED

Evidence: Dragging an image opens the Lap destination overlay, but the overlay is only partially styled and the preview is broken.

Issues:

- The drag operation has no clear image/confirmation card following the pointer.
- The capture overlay is incomplete, has unreadable low-contrast content, and shows a broken preview image.

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

Result: FAILED

Evidence: An OpenAI-compatible OpenRouter provider configured with `https://openrouter.ai` returned HTTP 200 HTML beginning with `<!DOCTYPE html>` instead of JSON.

Issues:

- The OpenRouter root URL is joined to `/chat/completions` instead of the required `/api/v1/chat/completions` endpoint.
- The non-JSON response error renders a long HTML excerpt that obscures the actionable configuration problem.

## Final status

- [ ] PASS — ready for merge
- [x] FAILED — defects require fixes
- [ ] BLOCKED — waiting for environment, credentials, or decision

## Defect log

| ID | Description | Severity | Status |
|---|---|---|---|
| LAP-VAL-001 | OpenRouter root URL reaches the website HTML response instead of the OpenAI-compatible JSON API; the resulting error is excessively verbose. | High | Fix implemented; awaiting retest |
| LAP-VAL-002 | Browser drag capture lacks a pointer-following confirmation state, and the truncated overlay stylesheet causes incomplete layout, low contrast, and a broken preview presentation. | High | Fix implemented; awaiting retest |
