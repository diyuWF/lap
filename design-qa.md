# Design QA — browser drag classification menu

- Source visual truth:
  `/workspace/scratch/b4cbe8668986/upload/5ae30932703d933dcdc7dc6f9e9a59f6.png`
- Removed interaction reference:
  `/workspace/scratch/b4cbe8668986/upload/0c23e1d7-bf82-4793-9a16-f1afc3f90519.png`
- Intended implementation state: Chromium page with the 100 ms drag radial visible,
  AI center enabled, five existing directory targets, one create-directory target,
  and one more-folders target.
- Intended viewport: 1365 × 936 CSS px at density 1.
- Source pixels: 865 × 2048.
- Removed interaction reference pixels: 1230 × 945.

## Browser-rendered evidence

The extension source, isolated preview harness, and preview service all started
successfully. The configured Work Mode Chrome session rejected the preview URL
with `ERR_BLOCKED_BY_CLIENT`, including after a fresh tab was created. The
preview service continued to report a healthy running state.

Because no browser-rendered implementation screenshot could be captured, the
required combined source-and-implementation comparison could not be produced.
Static source inspection, build success, or a code-generated mock are not being
substituted for browser-rendered evidence.

## Primary interactions prepared for verification

- 100 ms drag-intent threshold.
- AI center hidden unless an enabled provider with an API key is configured.
- AI center label is “AI 分类”.
- Existing directory drop saves immediately with no confirmation dialog.
- “创建目录” is always appended to the outer radial targets.
- Create-directory drop opens an in-page parent/name form.
- Successful directory creation immediately saves the current image.
- “更多” opens the full existing-folder browser.
- Success and failure use compact dark status toasts.

## Source-level fidelity review

- Typography: dark UI uses the product's existing Inter/system stack, compact
  labels, restrained weights, and single-line truncation.
- Spacing/layout rhythm: directory targets are compact rounded cards on a
  420 px radial field; the center is a rounded glass card instead of the old
  oversized white circular control.
- Colors/tokens: charcoal surfaces, low-contrast borders, violet focus states,
  and restrained violet/orange edge shadows follow the supplied art direction.
- Image quality: the official Lap raster icon is used for the AI center and
  panel brand; the native dragged image remains semi-transparent.
- Copy/content: “保存到待整理区域” and the old confirmation copy were removed
  from the drag flow; the new actions are “AI 分类” and “创建目录”.

## Findings

- [P0] Browser-rendered visual comparison is unavailable.
  - Location: Work Mode Chrome preview.
  - Evidence: the healthy preview was rejected with `ERR_BLOCKED_BY_CLIENT`.
  - Impact: layout density, target overlap, and final visual fidelity cannot be
    truthfully signed off from rendered evidence.
  - Required follow-up: verify the packaged extension on the user's Chromium
    browser during the next manual-validation stage.

## Comparison history

1. The prior QA pass covered the superseded 500 ms light radial and confirmation
   flow.
2. This iteration replaced that state with a 100 ms dark glass radial,
   conditional AI center, direct save, and create-directory form.
3. Post-fix browser comparison was attempted twice but blocked before rendering,
   so no visual pass is claimed.

final result: blocked
