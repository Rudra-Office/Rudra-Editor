// Zoom level management.
// Extracted from toolbar-handlers.js to break circular dependency.
import { state, $ } from '../../../state.js';
import { markLayoutDirty } from '../../../render.js';
import { renderRuler } from '../../../ruler.js';

// ── E10.2: Unified zoom — set, persist, update UI ──
export function setZoomLevel(level) {
  level = Math.max(50, Math.min(200, Math.round(level)));
  const changed = state.zoomLevel !== level;
  state.zoomLevel = level;
  // Persist zoom across sessions
  try { localStorage.setItem('s1_zoom', String(level)); } catch (_) {}
  const label = level + '%';
  if ($('zoomValue')) $('zoomValue').textContent = label;
  if ($('tbZoomValue')) $('tbZoomValue').textContent = label;
  // Apply CSS zoom to the page container (not transform:scale, which offsets coordinates)
  const container = $('pageContainer');
  if (container) {
    if (level === 100) {
      container.style.zoom = '';
    } else {
      container.style.zoom = (level / 100);
    }
  }
  // Update active state in zoom dropdowns (status bar + toolbar)
  [$('zoomDropdown'), $('tbZoomDropdown')].forEach(dd => {
    if (dd) {
      dd.querySelectorAll('.zoom-preset').forEach(btn => {
        const v = btn.dataset.zoom;
        btn.classList.toggle('active', v === String(level));
      });
    }
  });
  // Invalidate layout cache when zoom changes so repagination uses fresh dimensions
  if (changed) markLayoutDirty();
  try { localStorage.setItem('s1-zoom', String(level)); } catch (_) {}
  renderRuler();
}
