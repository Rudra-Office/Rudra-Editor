// Footnote and endnote insertion helpers.
// Extracted from input.js to break circular dependency.
import { state } from '../../../state.js';
import { getSelectionInfo } from '../../../selection.js';

export function insertFootnoteAtCursor() {
  if (!state.doc) return;
  const info = getSelectionInfo();
  if (!info) return;
  const nodeId = info.startNodeId;
  if (!nodeId) return;
  import('../../../render.js').then(({ syncAllText: syncAll, renderDocument: renderDoc }) => {
    syncAll();
    try {
      if (typeof state.doc.insert_footnote === 'function') {
        state.doc.insert_footnote(nodeId, '');
        import('../../../collab.js').then(({ broadcastOp: bcast }) => {
          bcast({ action: 'insertFootnote', nodeId });
        });
        renderDoc();
        import('../../../toolbar.js').then(({ updateUndoRedo: uur, recordUndoAction: rua }) => {
          rua('Insert footnote');
          uur();
        });
        import('../toolbar/toast-announce.js').then(({ announce: ann }) => { ann('Footnote inserted'); });
      } else {
        import('../toolbar/toast-announce.js').then(({ showToast: st }) => {
          st('Footnote insertion not available in this build.', 'info');
        });
      }
    } catch (e) { console.error('insert footnote:', e); }
  });
}

export function insertEndnoteAtCursor() {
  if (!state.doc) return;
  const info = getSelectionInfo();
  if (!info) return;
  const nodeId = info.startNodeId;
  if (!nodeId) return;
  import('../../../render.js').then(({ syncAllText: syncAll, renderDocument: renderDoc }) => {
    syncAll();
    try {
      if (typeof state.doc.insert_endnote === 'function') {
        state.doc.insert_endnote(nodeId, '');
        import('../../../collab.js').then(({ broadcastOp: bcast }) => {
          bcast({ action: 'insertEndnote', nodeId });
        });
        renderDoc();
        import('../../../toolbar.js').then(({ updateUndoRedo: uur, recordUndoAction: rua }) => {
          rua('Insert endnote');
          uur();
        });
        import('../toolbar/toast-announce.js').then(({ announce: ann }) => { ann('Endnote inserted'); });
      } else {
        import('../toolbar/toast-announce.js').then(({ showToast: st }) => {
          st('Endnote insertion not available in this build.', 'info');
        });
      }
    } catch (e) { console.error('insert endnote:', e); }
  });
}
