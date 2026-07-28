# Design QA — browser drag radial menu

- Source visual truth: `/workspace/scratch/b4cbe8668986/upload/2fd22971-021e-4e7a-846c-acffde75eaef.png`
- Implementation evidence: Cloud Browser full-page comparison capture emitted on 2026-07-26; the right pane rendered the current `lap-extension/content.css` and the exact radial-menu DOM from `lap-extension/content.js`.
- Viewport: Cloud Browser desktop, 1365 × 936 CSS px, device scale 1.
- Source pixels: 758 × 654.
- Implementation comparison pixels: 1365 × 936, with source and implementation shown together in equal-width panes.
- State: radial menu visible after the 500 ms intent threshold; pending-area center target hovered; semi-transparent drag preview visible.

## Full-view comparison

The first comparison showed the new nine-character center label wrapping onto two
lines inside the old 110 px target. This reduced scanability and made the primary
drop target feel weaker than the reference.

Fix applied:

- increased the center target from 110 px to 146 px;
- forced the primary label to remain on one line;
- retained the existing radial spacing, directory chips, shadow, opacity, and
  pointer-following preview treatment.

The second browser-rendered comparison confirmed that the center label is now
single-line, visually dominant, and does not overlap the surrounding folder
targets.

## Focused-region comparison

The center target and its nearest three folder targets were large enough to judge
in the full-page side-by-side capture, so a separate crop was not required.

## Required fidelity surfaces

- Typography: PASS — the primary label has sufficient weight and no wrapping;
  secondary AI copy remains subordinate.
- Spacing/layout rhythm: PASS — the enlarged center preserves visible separation
  from the 78 px outer targets.
- Colors/tokens: PASS — the dark center, translucent radial field, white folder
  targets, and violet active state remain consistent with the existing extension.
- Image quality: PASS — the drag preview uses a real image with reduced opacity;
  no placeholder or synthetic asset replaces it.
- Copy/content: PASS — the center says “保存到待整理区域” and explains that AI
  classification happens after release.

## Primary interactions tested

- 500 ms threshold is present in source and covered by the DOM regression.
- Center target accepts drag enter/over/drop and click.
- Center selection creates a logical pending-area destination.
- Confirmation precedes capture.
- Capture payload leaves `folderPath` empty for server-side pending-folder
  resolution and includes `organizationQueue: "inbox"`.

## Console check

The comparison page produced no page-owned warnings or errors. Earlier errors in
the same browser log came from opening the Tauri application directly without a
native Tauri runtime and are unrelated to the isolated extension comparison.

## Findings

No actionable P0, P1, or P2 visual differences remain for the requested radial
interaction. Real Chromium extension behavior and the native AI workbench remain
part of the Windows manual-validation gate.

## Comparison history

1. P2: center label wrapped and weakened the primary action.
2. Fix: enlarged target and prevented label wrapping.
3. Post-fix evidence: second Cloud Browser side-by-side capture; no overlap or
   wrapping remained.

final result: passed
