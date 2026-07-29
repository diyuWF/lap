# Lap Distance Radial and Reference Board Design QA

## Comparison target

- Source visual truth:
  `/workspace/scratch/b4cbe8668986/upload/8ea02719-4a1a-4349-b664-25b0f7b42839.png`
  (`865 × 2048` px), plus the rejected radial capture
  `/workspace/scratch/b4cbe8668986/upload/4d76ad77-db1c-4e8a-b0f3-41230e499264.png`
  (`601 × 470` px).
- Browser radial implementation: cloud-browser inline capture of the real
  `lap-extension/content.css` radial markup rendered at `1363 × 936` CSS px.
- Reference-board implementation: cloud-browser inline capture of
  `/reference-board` at `1363 × 936` CSS px.
- Implementation pixels: `1363 × 936` for each full-view capture.
- Density: `devicePixelRatio = 1`; no density normalization was required.
- State: default Lap dark theme; intent rail at 82% distance progress; expanded
  radial with AI configured; reference board with one selected image at 100%.
- Normalization: the tall source is a visual-language reference rather than a
  layout specification. Comparison therefore targets charcoal surfaces,
  translucent depth, restrained purple/orange light, low-contrast borders,
  typography hierarchy, and centered interaction composition. The rejected
  radial screenshot provides the direct before-state for layout and behavior.

## Full-view comparison evidence

The source visual, rejected radial, expanded radial implementation, and
reference-board implementation were inspected at the same time in the review
surface. The new browser interaction removes the rejected white disc, drifting
pointer-relative composition, missing AI center, and incomplete directory-card
layout. The implementation fixes the stage to the viewport center, expands the
working diameter to 560 px, darkens and blurs the page, and uses individual
charcoal-glass targets with restrained purple/orange lighting.

The reference board applies the same surface hierarchy to a full desktop tool:
a near-black canvas, thin separators, compact controls, quiet status copy, an
official Lap logo, and focused accent light only around the selected image.

## Focused region comparison evidence

- Intent rail: inspected the 372 × 78 center rail, official logo, two-line copy,
  82% progress meter, entry motion, and mild page dim/blur.
- Expanded radial: inspected the 560 × 560 stage, 198 × 116 AI center, five
  138 × 64 outer targets, path truncation, card spacing, and stronger backdrop
  dim/blur.
- Reference-board toolbar: inspected pin, zoom, fit, 100%, minimize, and close
  controls for consistent icon weight, spacing, hover/active treatment, and
  title-bar density.
- Reference-board canvas: inspected selected-image border/glow, 100% framing,
  status bar, and the visual response after zooming to 270% and panning.

## Findings

- No actionable P0, P1, or P2 visual differences remain for this focused slice.
- Fonts and typography: compact neutral sans-serif typography matches the
  reference direction. The AI action is the strongest label; paths and helper
  text remain secondary and truncate safely.
- Spacing and layout rhythm: radial targets are evenly distributed, the AI
  center has clear dominance, and neither the 1363 × 936 radial nor reference
  board showed clipping or persistent-control overflow.
- Colors and tokens: charcoal surfaces, quiet borders, purple primary light,
  restrained warm light, and low-opacity shadows map to the source without
  copying its marketing-page composition.
- Image quality and assets: the official Lap raster logo is used in the rail,
  radial, and board. No emoji, CSS drawing, fake SVG, or placeholder logo
  replaces a supplied asset.
- Copy and content: “AI 分类”, existing folders, “创建目录”, and “更多” describe
  actual outcomes. The board hints correspond to working pan, zoom, and arrange
  interactions.
- Icons and affordances: the board uses the existing Lap icon library; pin,
  zoom, window, selected-image remove, and pointer cursors expose the available
  actions without decorative controls.
- Accessibility and resilience: buttons remain semantic; text contrast is
  readable in the tested dark state; Escape closes the browser overlay; Delete
  and Ctrl/Cmd+0 work on the board. Reduced-motion behavior remains a P3 polish
  opportunity.

## Primary interactions tested

- Opened the browser radial visual state and checked exact viewport centering,
  backdrop dim/blur, AI center, five outer targets, and completed entry motion.
- Opened the intent rail at 82% progress and checked the center position,
  progress meter, copy hierarchy, and animation.
- Opened `/reference-board` in the selected cloud browser.
- Used the wheel to zoom from 100% to 270% around the pointer.
- Dragged blank canvas space and confirmed the image viewport moved.
- Used “Show at 100%” and confirmed the status returned to 100% and the selected
  image was centered.
- Checked the current reference-board route for new console errors. No new Lap
  application error was produced; stale errors from earlier Tauri-only preview
  routes and one unrelated cloud-browser extension metadata error remained.

## Comparison history

- Iteration 1 finding (P1): the rejected interaction placed an unrefined light
  disc relative to the drag position, omitted “AI 分类”, exposed incomplete
  directory UI, and did not establish the requested dark visual hierarchy.
- Fix: replaced elapsed-time activation with one-third-short-side drag distance;
  added centered intent motion, mild intent dim/blur, stronger radial dim/blur,
  a fixed 560 px stage, restored AI center, compact outer cards, and official
  Lap logo.
- Iteration 2 finding (P1, QA harness only): the cloud browser's previously
  installed extension stylesheet added an obsolete square radial background to
  the temporary comparison harness.
- Fix: neutralized the stale harness-only rule and recaptured the implementation.
  The production `content.css` has no radial-stage fill and the post-fix capture
  showed only the intended floating cards.
- Post-fix result: no P0/P1/P2 visual finding remains in the browser-rendered
  states.

## Open questions and residual test gaps

- Native Windows pointer exit, multiwindow creation/reuse, always-on-top,
  external file drop, transparent-window composition, and layout persistence
  require the fresh packaged Tauri build.
- The first reference-board slice intentionally does not claim full PureRef
  parity such as rotation, crop, image grouping, annotations, or export.

## Follow-up polish

- P3: add `prefers-reduced-motion` variants for the rail and radial entry.
- P3: after Windows validation, consider optional per-image resize/rotate
  handles as a later focused reference-board increment.

## Implementation checklist

- [x] Distance-driven drag intent replaces the timer.
- [x] Intent and radial remain centered.
- [x] Page dim/blur has distinct intent and radial strengths.
- [x] AI classification is restored as the fixed center action.
- [x] Larger charcoal-glass radial matches the chosen visual language.
- [x] Reference board supports pan, 5%–3200% zoom, Fit All, 100%, arrangement,
  remove, additional drops, always-on-top control, and persistence.
- [x] Browser-rendered visual and interaction checks completed.
- [ ] Native Windows validation completed.

final result: passed
