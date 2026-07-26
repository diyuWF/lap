# Lap 0.4.1 Manual Validation Checklist

## Validation information

Tester:

Date:

Branch:

Commit:

Environment:

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
- [ ] Pinterest
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

Result:

Evidence:

Issues:

## 6. Online AI

Providers tested:

- [ ] OpenAI-compatible
- [ ] Gemini
- [ ] Anthropic

Functions:

- [ ] Provider saves
- [ ] API key is protected
- [ ] Connection test works
- [ ] Single-file analysis works
- [ ] Batch analysis works
- [ ] Suggestions enter review queue
- [ ] Accept applies tags
- [ ] Reject does not apply tags
- [ ] Restart preserves configuration

Result:

Evidence:

Issues:

## Final status

- [ ] PASS — ready for merge
- [ ] FAILED — defects require fixes
- [ ] BLOCKED — waiting for environment, credentials, or decision

## Defect log

| ID | Description | Severity | Status |
|---|---|---|---|
| | | | |
