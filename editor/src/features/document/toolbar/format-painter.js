// Format painter helpers.
// Extracted from toolbar-handlers.js to break circular dependency.
import { state, $ } from '../../../state.js';
import { renderDocument, syncAllText } from '../../../render.js';
import { getSelectionInfo } from '../../../selection.js';
import { updateToolbarState, updateUndoRedo, recordUndoAction } from '../../../toolbar.js';
import { broadcastOp } from '../../../collab.js';
import { markDirty } from '../../../file.js';
import { announce } from './toast-announce.js';

export function exitFormatPainter() {
  state.formatPainterMode = null;
  state.copiedFormat = null;
  const btn = $('btnFormatPainter');
  if (btn) {
    btn.classList.remove('format-painter-active');
    btn.classList.remove('active');
    btn.setAttribute('aria-pressed', 'false');
  }
  const page = $('pageContainer');
  if (page) page.classList.remove('format-painter-cursor');
  // D21: Remove body class for global cursor override
  document.body.classList.remove('format-painter-active');
}

/**
 * Apply the previously copied format to the current text selection.
 * Called on mouseup in the document while format painter is active.
 * Returns true if format was applied, false otherwise.
 */
export function applyFormatPainter() {
  if (!state.formatPainterMode) return false;
  // D10: If copiedFormat or doc is missing, the painter is in a stale state — force clear
  if (!state.copiedFormat || !state.doc) {
    exitFormatPainter();
    return false;
  }
  const info = getSelectionInfo();
  if (!info || info.collapsed) return false;

  syncAllText();
  try {
    const sn = info.startNodeId, so = info.startOffset;
    const en = info.endNodeId, eo = info.endOffset;
    const fmt = state.copiedFormat;

    // Apply each formatting property from the copied format
    const formatKeys = ['bold', 'italic', 'underline', 'strikethrough', 'superscript', 'subscript'];
    for (const key of formatKeys) {
      if (key in fmt) {
        state.doc.format_selection(sn, so, en, eo, key, fmt[key]);
        broadcastOp({ action: 'formatSelection', startNode: sn, startOffset: so, endNode: en, endOffset: eo, key, value: fmt[key] });
      }
    }
    // Apply value-based properties (fontSize, fontFamily, color, highlightColor)
    const valueKeys = ['fontSize', 'fontFamily', 'color', 'highlightColor'];
    for (const key of valueKeys) {
      if (fmt[key]) {
        state.doc.format_selection(sn, so, en, eo, key, fmt[key]);
        broadcastOp({ action: 'formatSelection', startNode: sn, startOffset: so, endNode: en, endOffset: eo, key, value: fmt[key] });
      }
    }

    renderDocument();
    updateToolbarState();
    recordUndoAction('Apply format painter');
    updateUndoRedo();
    markDirty();
    announce('Format applied');

    // In "once" mode, exit after first application
    if (state.formatPainterMode === 'once') {
      exitFormatPainter();
    }
    return true;
  } catch (e) {
    console.error('Format Painter: failed to apply formatting:', e);
    // D10: Always exit format painter on error to prevent stale active state
    exitFormatPainter();
    return false;
  }
}
