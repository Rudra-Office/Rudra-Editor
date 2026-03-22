// Undo/redo operations.
// Extracted from input.js to break circular dependency.
import { state, $ } from '../../../state.js';
import { getEditableText, setCursorAtOffset, setCursorAtStart } from '../../../selection.js';
import { renderDocument, syncAllText } from '../../../render.js';
import { updateToolbarState, updateUndoRedo, renderUndoHistory } from '../../../toolbar.js';
import { broadcastOp, isApplyingRemote, flushDeferredRemoteOps as _flushDeferredRemoteOps } from '../../../collab.js';
import { exitFormatPainter } from '../toolbar/format-painter.js';

/** Restore cursor to the best available position after undo/redo re-render */
function restoreCursorAfterUndoRedo(savedSel) {
  const page = $('pageContainer');
  if (!page) return;
  // Try saved position first
  if (savedSel && savedSel.startNodeId) {
    const el = page.querySelector(`[data-node-id="${savedSel.startNodeId}"]`);
    if (el) {
      const content = el.closest('.page-content');
      if (content) content.focus();
      const maxLen = Array.from(getEditableText(el)).length;
      const offset = Math.min(savedSel.startOffset || 0, maxLen);
      setCursorAtOffset(el, offset);
      return;
    }
  }
  // Fallback: first paragraph
  const firstEl = page.querySelector('[data-node-id]');
  if (firstEl) {
    const content = firstEl.closest('.page-content');
    if (content) content.focus();
    setCursorAtStart(firstEl);
  }
}

export function doUndo() {
  if (!state.doc) return;
  // Guard: don't undo while applying remote collaborative ops
  if (isApplyingRemote()) return;
  // X18: Prevent re-entry during undo execution
  if (state._applyingUndo) return;
  // ED2-21: Reset format painter on undo to prevent stale painter state
  if (state.formatPainterMode) {
    exitFormatPainter();
  }
  state._applyingUndo = true;
  clearTimeout(state.syncTimer);
  syncAllText();
  // Save cursor position before undo for restoration
  const savedSel = state.lastSelInfo ? { ...state.lastSelInfo } : null;
  try {
    // E3.1: Batch undo — if we're in a typing session, undo all typing steps at once
    const batch = state._typingBatch;
    // FS-37: Also check formatting batch — multiple rapid format ops undo together
    const fmtBatch = state._formatBatch;
    if (batch && batch.count > 1) {
      const steps = batch.count;
      state._typingBatch = null;
      for (let i = 0; i < steps; i++) {
        if (!state.doc.can_undo()) break;
        state.doc.undo();
      }
    } else if (fmtBatch && fmtBatch.count > 1) {
      // FS-37: Undo all formatting operations in the batch at once
      const steps = fmtBatch.count;
      state._formatBatch = null;
      clearTimeout(fmtBatch.timer);
      for (let i = 0; i < steps; i++) {
        if (!state.doc.can_undo()) break;
        state.doc.undo();
      }
    } else {
      state._typingBatch = null;
      if (fmtBatch) { clearTimeout(fmtBatch.timer); state._formatBatch = null; }
      state.doc.undo();
    }
    // E3.2: Advance undo history position
    state.undoHistoryPos = Math.min(state.undoHistoryPos + 1, state.undoHistory.length);
    renderDocument();
    // Restore cursor position after undo
    restoreCursorAfterUndoRedo(savedSel);
    updateToolbarState();
    renderUndoHistory();
    // Broadcast undo result with inline doc state so peers can apply
    // directly without a round-trip requestFullSync
    try {
      const bytes = state.doc.export('docx');
      const base64 = btoa(String.fromCharCode(...new Uint8Array(bytes)));
      broadcastOp({ action: 'fullDocSync', docBase64: base64 });
    } catch (_) {
      broadcastOp({ action: 'fullDocSync' });
    }
  } catch (e) { console.error('undo:', e); }
  finally { state._applyingUndo = false; }
  // X18: Apply any remote ops that were deferred during undo
  _flushDeferredRemoteOps();
  updateUndoRedo();
}

export function doRedo() {
  if (!state.doc) return;
  // Guard: don't redo while applying remote collaborative ops
  if (isApplyingRemote()) return;
  // X18: Prevent re-entry during redo execution
  if (state._applyingUndo) return;
  // ED2-21: Reset format painter on redo to prevent stale painter state
  if (state.formatPainterMode) {
    exitFormatPainter();
  }
  state._applyingUndo = true;
  clearTimeout(state.syncTimer);
  syncAllText();
  const savedSel = state.lastSelInfo ? { ...state.lastSelInfo } : null;
  try {
    state.doc.redo();
    // E3.2: Move undo history position back
    state.undoHistoryPos = Math.max(state.undoHistoryPos - 1, 0);
    renderDocument();
    restoreCursorAfterUndoRedo(savedSel);
    updateToolbarState();
    renderUndoHistory();
    // Broadcast redo result with inline doc state so peers can apply
    // directly without a round-trip requestFullSync
    try {
      const bytes = state.doc.export('docx');
      const base64 = btoa(String.fromCharCode(...new Uint8Array(bytes)));
      broadcastOp({ action: 'fullDocSync', docBase64: base64 });
    } catch (_) {
      broadcastOp({ action: 'fullDocSync' });
    }
  } catch (e) { console.error('redo:', e); }
  finally { state._applyingUndo = false; }
  // X18: Apply any remote ops that were deferred during redo
  _flushDeferredRemoteOps();
  updateUndoRedo();
}
