# Canvas-First Rendering Specification — M13.1

**Reference**: OnlyOffice `Graphics.js`, `DrawingDocument.js`, FreeType WASM
**Status**: Specification
**Current State**: Canvas mode exists (815 lines, production features), but is OPTIONAL. DOM is default.

---

## Current Canvas Capabilities (canvas-render.js)

Already implemented:
- Full document rendering (paragraphs, tables, images, shapes, textboxes, headers/footers, footnotes)
- Hit testing (click → nodeId + char offset)
- Caret rendering with blink
- Selection highlight rendering
- Page caching with hash-based dirty detection
- DPR scaling for retina displays
- Scene API integration (batch + per-page)
- Font-aware rendering via WasmFontDatabase

## What Needs to Change

### Phase 1: Make Canvas Default

1. **Toggle default**: In `main.js` initialization, call `setCanvasMode(true)` instead of relying on user toggle
2. **DOM fallback**: Keep DOM rendering as `a11y` mode — hidden DOM layer for screen readers
3. **Graceful degradation**: If canvas context creation fails, fall back to DOM

### Phase 2: Font-Aware Rendering (rustybuzz WASM)

Current: Canvas uses `ctx.font = "16px Arial"` and `ctx.fillText()` — browser-dependent.

Target: Use s1-text (rustybuzz) via WASM for glyph positioning, then draw individual glyphs.

```
Document Model → s1-layout (with rustybuzz shaping) → LayoutDocument
  → GlyphRun { glyphs: [{glyph_id, x_advance, y_advance, x_offset, y_offset}], font_id }
  → Canvas renders positioned glyphs using font outlines
```

**Option A: WASM font rasterization (like OnlyOffice FreeType)**
- Compile `ab_glyph` or `fontdue` to WASM
- Render glyph bitmaps in WASM, transfer to canvas via ImageData
- Pro: pixel-perfect cross-platform
- Con: Complex, large WASM binary

**Option B: Canvas font rendering with layout-engine metrics (pragmatic)**
- Use `ctx.fillText()` for actual rendering (browser font engine)
- Use s1-layout glyph positions for placement (rustybuzz metrics)
- Measure discrepancy and apply correction factors
- Pro: Simpler, smaller binary
- Con: Not pixel-perfect across platforms (but close)

**Recommendation: Option B first, Option A as future upgrade.**

### Phase 3: Overlay Canvas

Two-canvas architecture per page:
1. **Content canvas**: Document text, images, shapes — expensive to render, cached
2. **Overlay canvas**: Cursor, selection highlights, spell-check underlines, track change marks — cheap to render, redrawn on every frame

Benefits:
- Cursor blink doesn't require full page redraw
- Selection changes are instant
- Spell check underlines render independently

### Phase 4: Glyph Cache

```javascript
class GlyphCache {
  // Key: `${fontFamily}|${fontSize}|${bold}|${italic}|${char}`
  // Value: { width, height, bitmap? } or just measured metrics
  cache: Map<string, GlyphMetrics>
  maxSize: 10000  // LRU eviction
}
```

### Phase 5: Hidden DOM for Accessibility

Maintain invisible DOM that mirrors canvas content:
- `aria-hidden="false"` on hidden DOM container
- `aria-hidden="true"` on canvas container
- Screen readers read the DOM, visual users see canvas
- Selection events from DOM translated to canvas coordinates

---

## Migration Plan

1. Set `canvasMode = true` as default in state initialization
2. Ensure all editing operations work in canvas mode (text input, formatting, etc.)
3. Add overlay canvas for cursor/selection
4. Add spell-check underline rendering to overlay
5. Add track-change marks rendering to overlay
6. Performance test: ensure 60fps during typing
7. Remove DOM rendering from critical path (keep as a11y layer only)
