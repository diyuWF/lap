# Browser radial design QA

## Source visual truth

- Selected reference: `/workspace/scratch/b4cbe8668986/upload/6c811a23-c0c4-4b7a-8889-32ac42bd3db2.png`.
- Source dimensions: 715 × 687 px.
- Required hierarchy: a precision-instrument outer tick ring, one thin inner
  circular orbit, one bronze partial arc, a simple center control, and secondary
  circular controls arranged around the circle.
- Product-specific adaptation: empty folders use Lap's bundled folder icon;
  populated folders use their first image as a circular thumbnail.

## Rendered implementation

- Production files exercised by the QA harness: `lap-extension/content.js`,
  `lap-extension/content.css`, `lap-extension/folder.svg`,
  `lap-extension/radial-gauge.png`, `lap-extension/plus.svg`, and
  `lap-extension/more.svg`.
- Browser viewport: 1363 × 936 CSS px.
- The source reference and current browser render were normalized and inspected
  together in one 1600 × 720 comparison image.
- The final rendered dial is fixed to the exact viewport center and preserves the
  requested post-threshold dim/blur state.

## Visual comparison findings

- The outer tick density, thin inner orbit, bronze partial arc, and circular
  control vocabulary visibly match the reference hierarchy.
- Folder controls are distributed evenly around the orbit rather than gathered
  into the previous gravity rows.
- Three populated test folders rendered real first-image circular covers; three
  empty test folders retained the bundled folder icon.
- The center contains only the folder icon and `AI 分类` when AI is configured.
- Ivory empty-folder controls and the bronze create control remain legible over
  the dimmed web page; covered folders use a solid dark label band and white text.
- No opaque black disc, progress rail, timer, tutorial, or confirmation dialog is
  present in the radial interaction.
- No actionable P0, P1, or P2 visual mismatch remains in the tested desktop state.

## Primary interaction evidence

- A short 200 px cumulative path produced zero overlays.
- A reversing 500 px cumulative path crossed the one-third-width threshold.
- AI configured: center target visible; AI not configured: center target hidden.
- Existing directory: direct activation returned `已保存到目录` with zero dialogs.
- Create directory: activation opened the parent/name form and its primary action.
- Folder cover response failure remains non-blocking and preserves folder icons.
- Cover bytes render inside a closed Shadow DOM; the host page can observe only
  the circular cover host, not the local image data URL.

## Remaining packaged checks

- Native drag cadence and drop hit-testing on real Pinterest, Behance, and
  ArtStation pages.
- Windows capture-service access to real folder thumbnails and fallback behavior
  for corrupt, unsupported, or not-yet-indexed first images.

final result: passed
