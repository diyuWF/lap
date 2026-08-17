# Browser radial design QA

## Comparison target

- Source visual truth path:
  `/workspace/scratch/b4cbe8668986/upload/6c811a23-c0c4-4b7a-8889-32ac42bd3db2.png`
- Source pixels: 715 × 687 at the uploaded image's native density.
- Rendered implementation: production `lap-extension/content.js` and
  `lap-extension/content.css` loaded by the local radial QA page.
- Implementation screenshot evidence: inline Cloud Browser capture produced in
  this QA run at 1363 × 936. A durable screenshot path could not be written
  because the documented browser shared mount returned `EROFS`; the same bytes
  were emitted and inspected in the browser result before this report was saved.
- Implementation pixels/CSS viewport: 1363 × 936 at device scale 1.
- State: desktop, configured AI provider, two root folders, one root with nested
  descendants, populated-folder covers, dark post-threshold dim/blur treatment.
- Normalization: the source image and live implementation were rendered together
  in one 1363 × 936 browser viewport. The 715 × 687 source was proportionally
  reduced to a 332 px-wide reference card; the live 840 px dial was reduced to
  76% only for the side-by-side composition. Both retained native aspect ratio.

## Full-view comparison evidence

- The browser capture placed the selected precision-instrument reference at the
  left and the live production radial at the right in the same image.
- The live dial preserves the reference hierarchy: dense outer tick ring, one
  thin inner orbit, one restrained bronze partial arc, a compact center action,
  and secondary circular controls distributed around the orbit.
- The previous opaque black disc is absent. The web page remains visible through
  a warm charcoal blur/dim layer, so the radial reads as an instrument overlay
  rather than a modal panel.
- Root state visibly contained only `风景摄影`, `lap资源`, and `创建目录`; child `1`
  did not appear beside its parent.
- No P0, P1, or P2 full-view mismatch remained after the hierarchy and center
  asset correction.

## Focused-region comparison evidence

- Center: the browser capture showed the original transparent Lap AI mark at the
  visual axis of the dial with only the label `AI 分类`. The asset remains sharp,
  has no baked background, and uses the reference's ivory/bronze/dark palette.
- Folder targets: populated folders use their first image as a circular cover;
  empty folders use the bundled folder icon. Labels remain inside the circular
  target and do not collide with the orbit or ticks.
- Child state: activating `lap资源` replaced the root controls with exactly
  `返回上级`, child `1`, and `创建目录`. `lap资源` and `风景摄影` were both absent.
- Deeper state: activating `1` showed only `三渲二`, `写实`, navigation, and the
  create action. The create form preselected `D:/Lap/lap资源/1`.

## Required fidelity surfaces

- Fonts and typography: compact system sans-serif weights and ten-to-twelve-pixel
  circular labels match the reference's restrained instrument annotations while
  remaining readable at the tested viewport. No wrapping or clipping observed.
- Spacing and layout rhythm: the dial is fixed to viewport center in production;
  orbit targets use equal-angle placement and scale with viewport size. Center,
  target, orbit, and outer tick proportions are visually balanced.
- Colors and visual tokens: ivory, warm bronze, fine white lines, and translucent
  charcoal reproduce the source palette without the previous purple-heavy disc.
  Hover/target rings reuse the bronze accent and preserve contrast.
- Image quality and asset fidelity: the gauge is the existing production raster;
  the AI center is a purpose-built generated PNG with real alpha; folder/back/
  create controls use bundled icon assets rather than CSS or text approximations.
- Copy and content: `AI 分类`, `创建目录`, `返回上级`, and real folder names are the
  only visible action copy. Tutorial, timer, progress, subtitle, and confirmation
  copy are absent from the radial state.

## Primary interaction evidence

- Cumulative drag travel still activates at one third of browser width and does
  not use displacement from the starting point.
- Initial candidates: `风景摄影`, `lap资源`; descendants absent.
- Parent activation: direct child `1` present; parent and unrelated root absent.
- Next parent activation: direct children `三渲二` and `写实` present.
- Back action restores the immediately preceding level.
- AI center drop sent `workflowStatus: "inbox"` and `autoClassify: true` and
  rendered `AI 正在后台生成标签和目录建议` from the successful response.
- Create action in the child level preselected the active parent folder.
- No application console error from `terminal.local` or Lap code was observed;
  unrelated browser-instrumentation extension messages were excluded.

## Comparison history

1. Earlier candidate — P1: the center used a generic folder icon and did not
   start automatic analysis; parent and descendant folders appeared together.
2. Fix — added the original `ai-classify.png`, queued online analysis from the
   capture service, and changed radial candidates to root/direct-child levels
   with parent activation and back navigation.
3. Post-fix evidence — the same-view comparison and focused child-state capture
   above show the corrected instrument hierarchy, dedicated AI center, and strict
   one-level folder navigation. No actionable P0/P1/P2 finding remains.

## Residual manual test gap

- Real Pinterest/Behance/ArtStation native drag cadence, Windows capture-service
  provider execution, and packaged first-image cover retrieval remain part of
  the next manual validation package.

## Current follow-up QA — 2026-08-12

- Source visual truth path:
  `/workspace/scratch/b4cbe8668986/upload/6c811a23-c0c4-4b7a-8889-32ac42bd3db2.png`
- User defect evidence path:
  `/workspace/scratch/b4cbe8668986/upload/3d53786f-7f4b-4de8-a957-56782c61bf5a.png`
- Intended implementation viewport/state: 1363 × 936, desktop Chromium,
  distance threshold reached, AI service unavailable, `lap资源` root with child
  `1`.
- Browser-rendered implementation screenshot path: unavailable in this follow-up.
  The supervised preview reported running, but the selected cloud Chrome could
  not open it and the preview bridge returned an upstream error.
- Console check: blocked because the implementation page could not be opened.
- Full-view comparison: blocked; no current browser-rendered pixels exist to put
  beside the source without reusing stale evidence.
- Focused-region comparison: blocked for the same reason.
- Functional evidence: `npm test --prefix scripts/extension-tests` passed all
  four tests for the always-visible AI center and payload, document-level nested
  parent dwell, first-install-only setup opening, and Token field reveal/focus.
- Previous baseline evidence remains valid for unchanged gauge, typography,
  spacing, color, image assets, and copy. The current logic changes do not alter
  those tokens, but this is not a substitute for a fresh visual capture.

### Current findings

- [P1] Fresh visual and real native-drag verification unavailable
  - Location: browser radial center and `lap资源` parent hover state.
  - Evidence: deterministic DOM interaction tests pass, but the selected cloud
    browser could not load the current preview.
  - Impact: packaged Chromium may still expose browser-specific drag cadence or
    stacking differences that a DOM emulator cannot detect.
  - Fix/gate: build a fresh extension package, then manually verify center
    visibility, 360 ms parent dwell through cover/icon/label, and first-install
    setup in Chromium.

### Comparison history addendum

4. Package 24 manual feedback — P1: AI center absent; parent dwell did not open
   child `1`; first install required the user to find setup manually.
5. Current fix — AI center is unconditional, dwell is driven by document-level
   hit testing, and first install opens options. Four regression tests pass.
6. Current visual result — blocked by cloud preview access; no claim of visual
   pass is made for this iteration.

## Package 26 radial follow-up — 2026-08-16

- Source visual truth paths:
  - `/workspace/scratch/b4cbe8668986/upload/7f7a08d3-0d2a-4616-86c5-a7344f77ea32.png`
    (1291 × 1141): user evidence for the intersecting orbit controls and rejected
    center mark.
  - `/workspace/scratch/b4cbe8668986/upload/00c214f3-ee6e-4a8b-8220-676d530a5e74.png`
    (682 × 681): particle-ring purple AI wordmark direction; inspiration only,
    not a geometry to copy.
- Intended implementation state: desktop Chromium after the one-third-width drag
  threshold, root radial visible, `lap资源` containing child folder `1`.
- Intended viewport/density: 1200 × 900 CSS px at device scale 1; compact geometry
  also exercised at 360 × 640 CSS px.
- Browser-rendered implementation screenshot path: unavailable. This environment
  exposes neither the cloud browser control nor a local Chromium/Chrome binary,
  and direct Playwright use was not authorized.
- Full-view comparison evidence: blocked because no current browser-rendered
  pixels can be placed beside the source.
- Focused-region evidence: the generated production assets
  `lap-extension/ai-wordmark.png` (384 × 384 RGBA) and
  `lap-extension/ai-orbit.png` (512 × 512 RGBA) were opened at native resolution.
  Both have real alpha, retain crisp purple/violet edges, and are visibly distinct
  from the reference; this asset inspection is not substituted for browser QA.
- Primary interactions tested: geometry fallback opens `lap资源 → 1 →
  三渲二/写实` when `elementsFromPoint()` returns no target; the stationary
  pointer lock prevents immediate return; current-parent save remains direct;
  crowded target sets retain `更多` and do not intersect at both tested viewports.
- Console errors checked: unavailable without a browser-rendered implementation.

### Current findings

- [P1] Packaged native-drag and rendered-motion evidence remains unavailable
  - Location: parent dwell, radial target geometry, center AI animation.
  - Evidence: six deterministic extension tests, JavaScript parsing, Manifest
    parsing, PNG alpha checks, strict i18n audit, and Vite production build pass;
    no browser screenshot or console session is available in this environment.
  - Impact: Chromium drag cadence, final compositing, motion smoothness, and the
    perceived stroke gap still require the actual extension package.
  - Fix/gate: generate the fresh Windows/extension package and manually verify
    it on a real Chromium image page at desktop scale.

### Required fidelity surfaces

- Fonts and typography: unchanged system-sans folder labels; `AI 分类` remains a
  twelve-pixel optical label. Browser antialiasing remains a manual check.
- Spacing and layout rhythm: computed geometry keeps every target inside the
  inner orbit with at least seven pixels of stroke clearance and seven pixels of
  target-to-target clearance; browser pixels remain unverified.
- Colors and visual tokens: the gauge keeps the ivory/bronze instrument system;
  purple/violet is isolated to the AI action as requested.
- Image quality and asset fidelity: both custom visuals are generated raster
  assets with alpha, not CSS/HTML drawings. CSS only transforms the real orbit
  asset for drift and counter-rotation.
- Copy and content: root and child states retain only folder names, `保存到 …`,
  `返回上级`, `创建目录`, `更多`, and `AI 分类`.

### Comparison history addendum

7. Package 26 manual feedback — P1: native drag still did not reveal children,
   controls intersected the orbit stroke, and the bronze center mark was rejected.
8. Current fix — event-target/document/geometry hybrid dwell with miss grace,
   capacity-aware inner placement, and original purple AI wordmark plus two
   independently animated particle layers. Six regression tests pass.
9. Current visual result — blocked until the fresh package is rendered and
   exercised in the user's real Chromium environment.

final result: blocked
