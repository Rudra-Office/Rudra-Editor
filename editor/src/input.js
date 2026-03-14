// Keyboard, input, paste, clipboard handling
import { state, $ } from './state.js';
import {
  getSelectionInfo, getActiveElement, getCursorOffset,
  setCursorAtOffset, setCursorAtStart, isCursorAtStart, isCursorAtEnd,
} from './selection.js';
import { renderDocument, renderNodeById, syncParagraphText, syncAllText, debouncedSync } from './render.js';
import { toggleFormat, applyFormat, updateToolbarState, updateUndoRedo } from './toolbar.js';
import { deleteSelectedImage, setupImages } from './images.js';
import { updatePageBreaks } from './pagination.js';
import { markDirty, saveVersion, updateDirtyIndicator, updateStatusBar } from './file.js';
import { broadcastOp } from './collab.js';

export function initInput() {
  const page = $('docPage');

  // ─── E-01 fix: Capture cursor offset before text insertion for pending formats ───
  page.addEventListener('beforeinput', (e) => {
    if (state.ignoreInput) return;
    if (e.inputType === 'insertText' && e.data) {
      const pending = state.pendingFormats;
      if (pending && Object.keys(pending).length > 0) {
        const el = getActiveElement();
        if (el) {
          state._pendingFormatInsert = {
            nodeId: el.dataset.nodeId,
            offset: getCursorOffset(el),
            charCount: Array.from(e.data).length,
          };
        }
      }
    }
  });

  // ─── Regular input (typing) ─────────────────────
  page.addEventListener('input', (e) => {
    if (state.ignoreInput) return;
    const el = getActiveElement();
    if (el) debouncedSync(el);

    // ── E-01 fix: Apply pending formats to newly inserted character(s) ──
    if (state._pendingFormatInsert && e.inputType === 'insertText') {
      const pfi = state._pendingFormatInsert;
      state._pendingFormatInsert = null;
      const pending = state.pendingFormats;
      if (pending && Object.keys(pending).length > 0 && state.doc) {
        // Sync the paragraph text immediately so the WASM model has the new character
        if (el) {
          clearTimeout(state.syncTimer);
          syncParagraphText(el);
        }
        try {
          const startOff = pfi.offset;
          const endOff = startOff + pfi.charCount;
          const nodeId = pfi.nodeId;
          for (const [key, value] of Object.entries(pending)) {
            state.doc.format_selection(nodeId, startOff, nodeId, endOff, key, value);
            broadcastOp({ action: 'formatSelection', startNode: nodeId, startOffset: startOff, endNode: nodeId, endOffset: endOff, key, value });
          }
          // Re-render the node to show the formatting, then restore cursor
          const updated = renderNodeById(nodeId);
          if (updated) setCursorAtOffset(updated, endOff);
          // Update the tracked cursor position so selectionchange doesn't
          // clear pending formats due to the cursor advancing by one character
          state._pendingFormatCursorPos = { nodeId, offset: endOff };
          updateUndoRedo();
        } catch (err) { console.error('pending format apply:', err); }
        // Keep pending formats active so subsequent characters also get formatted
        // They will be cleared when the user clicks somewhere else or changes selection
      }
    }

    // ── Slash menu: detect "/" or update filter ──
    if (state.slashMenuOpen) {
      const text = el?.textContent || '';
      const offset = getCursorOffset(el);
      // Find the "/" that triggered the menu
      // offset is in codepoints, so convert text to codepoint array for slicing
      const codepoints = [...text];
      const before = codepoints.slice(0, offset).join('');
      const slashPos = before.lastIndexOf('/');
      if (slashPos >= 0) {
        const query = before.substring(slashPos + 1);
        updateSlashFilter(query);
      } else {
        closeSlashMenu();
      }
    } else if (e.inputType === 'insertText' && e.data === '/') {
      // Open menu if "/" is at start of paragraph or after whitespace
      if (el) {
        const offset = getCursorOffset(el);
        const text = el.textContent || '';
        // offset is in codepoints, so index into codepoint array
        const codepoints = [...text];
        const charBefore = offset >= 2 ? codepoints[offset - 2] : null;
        if (offset === 1 || (charBefore && /\s/.test(charBefore))) {
          openSlashMenu();
        }
      }
    }
  });

  // ─── Copy — write both plain text and HTML to clipboard ───
  page.addEventListener('copy', e => {
    if (!state.doc) return;
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed) return;

    e.preventDefault();
    const text = sel.toString();
    const html = getSelectionHtml();

    // Store internal clipboard for rich paste within editor
    syncAllText();
    storeInternalClipboard();

    e.clipboardData.setData('text/plain', text);
    e.clipboardData.setData('text/html', html);
  });

  // ─── Keydown ────────────────────────────────────
  page.addEventListener('keydown', e => {
    if (!state.doc) return;
    const doc = state.doc;

    // ── Slash menu navigation ──
    if (state.slashMenuOpen) {
      const commands = filterSlashCommands(state.slashQuery);
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        state.slashMenuIndex = Math.min(state.slashMenuIndex + 1, commands.length - 1);
        renderSlashMenu(commands);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        state.slashMenuIndex = Math.max(state.slashMenuIndex - 1, 0);
        renderSlashMenu(commands);
        return;
      }
      if (e.key === 'Enter') {
        e.preventDefault();
        if (commands.length > 0 && state.slashMenuIndex < commands.length) {
          executeSlashCommand(commands[state.slashMenuIndex].id);
        } else {
          closeSlashMenu();
        }
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        closeSlashMenu();
        return;
      }
      if (e.key === 'Backspace' || e.key === 'Delete') {
        // If query is empty, the "/" itself will be deleted, so close menu
        if (state.slashQuery.length === 0) {
          closeSlashMenu();
          // Let backspace/delete proceed to remove the "/"
        } else {
          // Let it proceed normally; after the DOM updates, verify the slash
          // trigger character is still present. Use setTimeout(0) so the check
          // runs after the browser applies the deletion to the DOM.
          setTimeout(() => {
            if (!state.slashMenuOpen) return;
            const activeEl = getActiveElement();
            const text = activeEl?.textContent || '';
            const cursorOff = activeEl ? getCursorOffset(activeEl) : 0;
            // cursorOff is in codepoints, so slice codepoint array
            const before = [...text].slice(0, cursorOff).join('');
            if (before.lastIndexOf('/') < 0) {
              closeSlashMenu();
            }
          }, 0);
        }
      }
    }

    // Delete selected image
    if (state.selectedImg && (e.key === 'Delete' || e.key === 'Backspace')) {
      e.preventDefault(); deleteSelectedImage(); return;
    }

    const info = getSelectionInfo();

    // ── Ctrl/Cmd shortcuts ──
    if (e.ctrlKey || e.metaKey) {
      switch (e.key.toLowerCase()) {
        case 'b': e.preventDefault(); toggleFormat('bold'); return;
        case 'i': e.preventDefault(); toggleFormat('italic'); return;
        case 'u': e.preventDefault(); toggleFormat('underline'); return;
        case 'z': e.preventDefault(); e.shiftKey ? doRedo() : doUndo(); return;
        case 'y': e.preventDefault(); doRedo(); return;
        case 'x': e.preventDefault(); doCut(e); return;
        case 'c': /* handled by copy event above */ return;
        case 'v': /* handled by paste event */ return;
        case 'a': /* let browser handle select all */ return;
        case 's': e.preventDefault(); saveToLocal(); return;
        case 'f': e.preventDefault(); $('findBar').classList.add('show'); $('findInput').focus(); return;
        case 'h': e.preventDefault(); $('findBar').classList.add('show'); $('replaceInput')?.focus(); return;
        case 'p': e.preventDefault(); window.print(); return;
        case '=':
        case '+': e.preventDefault(); adjustEditorZoom(10); return;
        case '-': e.preventDefault(); adjustEditorZoom(-10); return;
        case '0': e.preventDefault(); adjustEditorZoom(0); return;
      }
    }

    // ── Delete/Backspace with selection ──
    if ((e.key === 'Delete' || e.key === 'Backspace') && info && !info.collapsed) {
      e.preventDefault();
      clearTimeout(state.syncTimer);
      syncAllText();
      try {
        doc.delete_selection(info.startNodeId, info.startOffset, info.endNodeId, info.endOffset);
        renderDocument();
        // Try to place cursor at the start of the deletion point
        let el = page.querySelector(`[data-node-id="${info.startNodeId}"]`);
        if (el) {
          setCursorAtOffset(el, info.startOffset);
        } else {
          // The start node was deleted — find any remaining paragraph
          el = page.querySelector('p[data-node-id], h1[data-node-id], h2[data-node-id], h3[data-node-id]');
          if (el) {
            setCursorAtStart(el);
          } else {
            // Document is completely empty — create a new paragraph
            try { doc.append_paragraph(''); } catch (_) {}
            renderDocument();
            const n = page.querySelector('[data-node-id]');
            if (n) setCursorAtStart(n);
          }
        }
        updateUndoRedo();
        broadcastOp({ action: 'deleteSelection', startNode: info.startNodeId, startOffset: info.startOffset, endNode: info.endNodeId, endOffset: info.endOffset });
      } catch (err) { console.error('delete selection:', err); }
      return;
    }

    const el = getActiveElement();

    // ── Tab — table navigation ──
    if (e.key === 'Tab') {
      const cell = el?.closest?.('td, th');
      if (cell) {
        e.preventDefault();
        const row = cell.parentElement;
        const table = row?.closest('table');
        if (!table) return;
        const cells = Array.from(table.querySelectorAll('td, th'));
        const idx = cells.indexOf(cell);
        const next = e.shiftKey ? cells[idx - 1] : cells[idx + 1];
        if (next) {
          const textNode = next.querySelector('[data-node-id]');
          if (textNode) { setCursorAtStart(textNode); }
          else { next.focus(); }
        }
        return;
      }
    }

    // ── Shift+Enter — insert line break ──
    if (e.key === 'Enter' && e.shiftKey) {
      e.preventDefault();
      if (!el) return;
      const nodeId = el.dataset.nodeId;
      const offset = getCursorOffset(el);
      clearTimeout(state.syncTimer); syncParagraphText(el);
      try {
        doc.insert_line_break(nodeId, offset);
        broadcastOp({ action: 'insertLineBreak', nodeId, offset });
        const updated = renderNodeById(nodeId);
        if (updated) setCursorAtOffset(updated, offset + 1);
        state.pagesRendered = false; updatePageBreaks(); updateUndoRedo();
      } catch (err) {
        console.error('insert line break:', err);
      }
      return;
    }

    // ── Enter — split paragraph ──
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      state._typingBatch = null; // E3.1: End typing session on Enter
      if (!el) return;
      const nodeId = el.dataset.nodeId;
      const offset = getCursorOffset(el);
      clearTimeout(state.syncTimer); syncParagraphText(el);
      try {
        const newId = doc.split_paragraph(nodeId, offset);
        renderNodeById(nodeId);
        const newHtml = doc.render_node_html(newId);
        const tmp = document.createElement('div'); tmp.innerHTML = newHtml;
        const newEl = tmp.firstElementChild;
        if (newEl) {
          if (!newEl.innerHTML.trim()) newEl.innerHTML = '<br>';
          const orig = page.querySelector(`[data-node-id="${nodeId}"]`);
          if (orig) orig.after(newEl);
          setupImages(newEl);
          setCursorAtStart(newEl);
        }
        state.pagesRendered = false; updatePageBreaks(); updateUndoRedo(); markDirty();
        broadcastOp({ action: 'splitParagraph', nodeId, offset });
      } catch (err) { console.error('split:', err); }
      return;
    }

    // ── E1.6: Prevent deletion of page-break / HR divs ──
    if ((e.key === 'Delete' || e.key === 'Backspace') && !el) {
      // Cursor might be on a non-editable element (page-break, HR)
      const sel = window.getSelection();
      if (sel && sel.anchorNode) {
        const anchor = sel.anchorNode.nodeType === 1 ? sel.anchorNode : sel.anchorNode.parentElement;
        if (anchor && (anchor.classList?.contains('page-break') || anchor.tagName === 'HR' ||
            anchor.closest?.('.page-break') || anchor.closest?.('.editor-header') || anchor.closest?.('.editor-footer'))) {
          e.preventDefault();
          return;
        }
      }
    }

    // ── Backspace at start — merge prev ──
    if (e.key === 'Backspace' && el && isCursorAtStart(el)) {
      let prev = el.previousElementSibling;
      while (prev && (prev.classList.contains('page-break') || prev.classList.contains('editor-footer') || prev.classList.contains('editor-header'))) prev = prev.previousElementSibling;
      if (prev?.dataset?.nodeId) {
        e.preventDefault();
        clearTimeout(state.syncTimer); syncParagraphText(el); syncParagraphText(prev);
        const cursorPos = Array.from(prev.textContent || '').length;
        const nodeId1 = prev.dataset.nodeId;
        const nodeId2 = el.dataset.nodeId;
        try {
          doc.merge_paragraphs(nodeId1, nodeId2);
          const updated = renderNodeById(nodeId1);
          el.remove();
          if (updated) setCursorAtOffset(updated, cursorPos);
          state.pagesRendered = false; updatePageBreaks(); updateUndoRedo(); markDirty();
          broadcastOp({ action: 'mergeParagraphs', nodeId1, nodeId2 });
        } catch (err) { console.error('merge:', err); }
      }
      return;
    }

    // ── Delete at end — merge next ──
    if (e.key === 'Delete' && el && isCursorAtEnd(el)) {
      let next = el.nextElementSibling;
      while (next && (next.classList.contains('page-break') || next.classList.contains('editor-footer') || next.classList.contains('editor-header'))) next = next.nextElementSibling;
      if (next?.dataset?.nodeId) {
        e.preventDefault();
        clearTimeout(state.syncTimer); syncParagraphText(el); syncParagraphText(next);
        const cursorPos = Array.from(el.textContent || '').length;
        const nodeId1 = el.dataset.nodeId;
        const nodeId2 = next.dataset.nodeId;
        try {
          doc.merge_paragraphs(nodeId1, nodeId2);
          const updated = renderNodeById(nodeId1);
          next.remove();
          if (updated) setCursorAtOffset(updated, cursorPos);
          state.pagesRendered = false; updatePageBreaks(); updateUndoRedo(); markDirty();
          broadcastOp({ action: 'mergeParagraphs', nodeId1, nodeId2 });
        } catch (err) { console.error('merge:', err); }
      }
      return;
    }
  });

  // ─── Paste ──────────────────────────────────────
  page.addEventListener('paste', e => {
    e.preventDefault();
    if (!state.doc) return;
    const doc = state.doc;

    let info = getSelectionInfo();

    // Delete selection first if not collapsed
    if (info && !info.collapsed) {
      syncAllText();
      try {
        doc.delete_selection(info.startNodeId, info.startOffset, info.endNodeId, info.endOffset);
        renderDocument();
      } catch (_) {}
      // After delete + re-render, DOM is rebuilt. Clear stale selection info.
      state.lastSelInfo = null;
      info = null;
    } else if (info) {
      syncParagraphText(info.startEl);
    }

    // Ensure we have a valid target paragraph
    const ensureTarget = () => {
      // Try to get fresh selection after re-render
      let firstEl = page.querySelector('p[data-node-id], h1[data-node-id], h2[data-node-id], h3[data-node-id], h4[data-node-id], h5[data-node-id], h6[data-node-id]');
      if (!firstEl) {
        // Document is completely empty — create a paragraph
        try { doc.append_paragraph(''); renderDocument(); } catch (_) {}
        firstEl = page.querySelector('[data-node-id]');
      }
      if (firstEl) {
        setCursorAtStart(firstEl);
        return { startNodeId: firstEl.dataset.nodeId, startOffset: 0, startEl: firstEl };
      }
      return null;
    };

    if (!info || !info.startNodeId) {
      info = ensureTarget();
      if (!info) return;
    } else {
      // Verify the node still exists in the DOM (might have been deleted)
      const existing = page.querySelector(`[data-node-id="${info.startNodeId}"]`);
      if (!existing) {
        info = ensureTarget();
        if (!info) return;
      }
    }

    const text = e.clipboardData.getData('text/plain');
    if (!text) return;

    if (text.includes('\n')) {
      try {
        doc.paste_plain_text(info.startNodeId, info.startOffset, text);
        broadcastOp({ action: 'pasteText', nodeId: info.startNodeId, offset: info.startOffset, text });
        renderDocument();
        // E-11: Place cursor at end of pasted content — find the paragraph
        // whose text ends with the last pasted line, searching from the bottom
        const lines = text.split('\n');
        const lastLine = lines[lines.length - 1];
        const allEls = Array.from(page.querySelectorAll('p[data-node-id], h1[data-node-id], h2[data-node-id], h3[data-node-id], h4[data-node-id], h5[data-node-id], h6[data-node-id]'));
        let targetEl = null;
        for (let i = allEls.length - 1; i >= 0; i--) {
          if ((allEls[i].textContent || '').endsWith(lastLine)) {
            targetEl = allEls[i];
            break;
          }
        }
        if (!targetEl && allEls.length > 0) targetEl = allEls[allEls.length - 1];
        if (targetEl) setCursorAtOffset(targetEl, [...(targetEl.textContent || '')].length);
        updateUndoRedo();
        markDirty();
      } catch (err) {
        console.error('paste multi-line:', err);
        // Fallback: insert as single line via WASM
        try {
          const flatText = text.replace(/\n/g, ' ');
          doc.insert_text_in_paragraph(info.startNodeId, info.startOffset, flatText);
          broadcastOp({ action: 'insertText', nodeId: info.startNodeId, offset: info.startOffset, text: flatText });
          renderDocument();
          updateUndoRedo();
        } catch (e2) { console.error('paste fallback:', e2); }
      }
    } else {
      try {
        doc.insert_text_in_paragraph(info.startNodeId, info.startOffset, text);
        broadcastOp({ action: 'insertText', nodeId: info.startNodeId, offset: info.startOffset, text });
        const updated = renderNodeById(info.startNodeId);
        if (updated) setCursorAtOffset(updated, info.startOffset + Array.from(text).length);
        updateUndoRedo();
        markDirty();
      } catch (err) {
        console.error('paste single-line:', err);
      }
    }
  });

  // ─── Selection change ──────────────────────────
  // E-01 fix: Clear pending formats when cursor moves or selection changes
  document.addEventListener('selectionchange', () => {
    if (state.pendingFormats && Object.keys(state.pendingFormats).length > 0) {
      // Only clear if the selection actually moved to a different position
      // (toolbar mousedown prevention means formatting buttons won't trigger this)
      const sel = window.getSelection();
      if (sel && sel.rangeCount > 0) {
        const info = getSelectionInfo();
        const prev = state._pendingFormatCursorPos;
        if (info && prev) {
          const moved = info.startNodeId !== prev.nodeId || info.startOffset !== prev.offset;
          // If the cursor moved and we're not in the middle of a pending format insert,
          // clear the pending formats
          if (moved && !state._pendingFormatInsert) {
            state.pendingFormats = {};
          }
        }
        // Track current cursor position for comparison
        if (info) {
          state._pendingFormatCursorPos = { nodeId: info.startNodeId, offset: info.startOffset };
        }
      }
    }
    updateToolbarState();
  });

  // ─── Prevent toolbar from stealing focus ───────
  $('toolbar').addEventListener('mousedown', e => {
    const tag = e.target.tagName.toLowerCase();
    if (tag !== 'select' && tag !== 'input') e.preventDefault();
  });

  // ─── Global Escape handler — close modals/menus ──
  document.addEventListener('keydown', e => {
    if (e.key !== 'Escape') return;
    // Close slash menu
    if (state.slashMenuOpen) {
      closeSlashMenu();
      return;
    }
    // Close find bar
    if ($('findBar').classList.contains('show')) {
      $('findBar').classList.remove('show');
      const docPage = $('docPage');
      if (docPage) docPage.focus();
      return;
    }
    // Close table modal
    if ($('tableModal').classList.contains('show')) {
      $('tableModal').classList.remove('show');
      return;
    }
    // Close comment modal
    if ($('commentModal').classList.contains('show')) {
      $('commentModal').classList.remove('show');
      return;
    }
    // Close link modal
    if ($('linkModal').classList.contains('show')) {
      $('linkModal').classList.remove('show');
      return;
    }
    // Close alt text modal
    if ($('altTextModal').classList.contains('show')) {
      $('altTextModal').classList.remove('show');
      return;
    }
    // Close menus
    $('exportMenu').classList.remove('show');
    $('insertMenu').classList.remove('show');
    $('tableContextMenu').style.display = 'none';
    // Close comments panel
    if ($('commentsPanel').classList.contains('show')) {
      $('commentsPanel').classList.remove('show');
      return;
    }
    // Close history panel
    if ($('historyPanel').classList.contains('show')) {
      $('historyPanel').classList.remove('show');
      return;
    }
  });
}

// ─── Internal Clipboard System ─────────────────────
// Stores the full document state before cut, so paste restores everything

function storeInternalClipboard() {
  // Internal clipboard disabled — use standard paste flow
  // The old approach replaced the entire document on paste, causing data loss
  state.internalClipboard = null;
}

function restoreFromInternalClipboard() {
  // Disabled — no-op. Standard paste flow handles all cases.
  state.internalClipboard = null;
  throw new Error('Internal clipboard disabled');
}

function getSelectionHtml() {
  const sel = window.getSelection();
  if (!sel || !sel.rangeCount) return '';
  const range = sel.getRangeAt(0);
  const div = document.createElement('div');
  div.appendChild(range.cloneContents());
  return div.innerHTML;
}

// insertTextAtCursor removed — all text insertion must go through WASM to maintain model consistency

function doUndo() {
  if (!state.doc) return;
  clearTimeout(state.syncTimer);
  syncAllText();
  try {
    // E3.1: Batch undo — if we're in a typing session, undo all typing steps at once
    const batch = state._typingBatch;
    if (batch && batch.count > 1) {
      const steps = batch.count;
      state._typingBatch = null;
      for (let i = 0; i < steps; i++) {
        if (!state.doc.can_undo()) break;
        state.doc.undo();
      }
    } else {
      state._typingBatch = null;
      state.doc.undo();
    }
    renderDocument();
    updateToolbarState();
    // Broadcast full document sync so peers see the undo result
    broadcastOp({ action: 'fullDocSync' });
  } catch (e) { console.error('undo:', e); }
}

function doRedo() {
  if (!state.doc) return;
  try {
    state.doc.redo();
    renderDocument();
    updateToolbarState();
    // Broadcast full document sync so peers see the redo result
    broadcastOp({ action: 'fullDocSync' });
  } catch (e) { console.error('redo:', e); }
}

function doCut() {
  const info = getSelectionInfo();
  if (!info || info.collapsed || !state.doc) return;

  // Store document state for rich paste
  syncAllText();
  storeInternalClipboard();

  // Copy HTML + plain text to system clipboard
  const sel = window.getSelection();
  if (sel) {
    const text = sel.toString();
    const html = getSelectionHtml();
    // Use clipboard API with both formats
    try {
      const blob = new Blob([html], { type: 'text/html' });
      const textBlob = new Blob([text], { type: 'text/plain' });
      navigator.clipboard.write([
        new ClipboardItem({ 'text/html': blob, 'text/plain': textBlob })
      ]).catch(() => {
        navigator.clipboard.writeText(text).catch(() => {});
      });
    } catch (_) {
      navigator.clipboard.writeText(text).catch(() => {});
    }
  }

  // Delete the selection
  try {
    state.doc.delete_selection(info.startNodeId, info.startOffset, info.endNodeId, info.endOffset);
    broadcastOp({ action: 'deleteSelection', startNode: info.startNodeId, startOffset: info.startOffset, endNode: info.endNodeId, endOffset: info.endOffset });
    renderDocument();
    const el = $('docPage').querySelector(`[data-node-id="${info.startNodeId}"]`);
    if (el) setCursorAtOffset(el, info.startOffset);
    else {
      const first = $('docPage').querySelector('[data-node-id]');
      if (first) setCursorAtStart(first);
      else { state.doc.append_paragraph(''); renderDocument(); }
    }
    updateUndoRedo();
  } catch (e) { console.error('cut:', e); }
}

function saveToLocal() {
  if (!state.doc) return;
  try {
    syncAllText();
    const bytes = state.doc.export('docx');
    const name = $('docName').value || 'Untitled Document';
    const req = indexedDB.open('FolioAutosave', 2);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains('documents')) {
        db.createObjectStore('documents', { keyPath: 'id' });
      }
      if (!db.objectStoreNames.contains('versions')) {
        db.createObjectStore('versions', { keyPath: 'id', autoIncrement: true });
      }
    };
    req.onsuccess = () => {
      const db = req.result;
      const tx = db.transaction('documents', 'readwrite');
      tx.objectStore('documents').put({ id: 'current', name, bytes, timestamp: Date.now() });
      state.dirty = false;
      updateDirtyIndicator();
      const info = $('statusInfo');
      info._userMsg = true;
      info.textContent = 'Saved';
      setTimeout(() => { info._userMsg = false; updateStatusBar(); }, 1500);
    };
    // Also save a version snapshot on manual save
    saveVersion('Manual save');
  } catch (e) { console.error('save:', e); }
}

// ─── Slash Command Menu ─────────────────────────────
const SLASH_COMMANDS = [
  { id: 'heading1',   label: 'Heading 1',       icon: 'H1', keywords: 'heading h1 title' },
  { id: 'heading2',   label: 'Heading 2',       icon: 'H2', keywords: 'heading h2' },
  { id: 'heading3',   label: 'Heading 3',       icon: 'H3', keywords: 'heading h3' },
  { id: 'bullet',     label: 'Bullet List',     icon: '\u2022',  keywords: 'bullet list unordered ul' },
  { id: 'numbered',   label: 'Numbered List',   icon: '1.',  keywords: 'numbered list ordered ol' },
  { id: 'table',      label: 'Table',           icon: '\u2637',  keywords: 'table grid' },
  { id: 'image',      label: 'Image',           icon: '\uD83D\uDDBC',  keywords: 'image picture photo' },
  { id: 'hr',         label: 'Horizontal Rule', icon: '\u2014',  keywords: 'horizontal rule divider line separator hr' },
  { id: 'pagebreak',  label: 'Page Break',      icon: '\u23CE',  keywords: 'page break new page' },
  { id: 'quote',      label: 'Quote',           icon: '\u201C',  keywords: 'quote blockquote' },
  { id: 'code',       label: 'Code Block',      icon: '</>',keywords: 'code block monospace' },
];

function filterSlashCommands(query) {
  if (!query) return SLASH_COMMANDS;
  const q = query.toLowerCase();
  return SLASH_COMMANDS.filter(cmd =>
    cmd.label.toLowerCase().includes(q) || cmd.keywords.includes(q)
  );
}

function renderSlashMenu(commands) {
  const menu = $('slashMenu');
  if (commands.length === 0) {
    menu.style.display = 'none';
    state.slashMenuOpen = false;
    return;
  }
  menu.innerHTML = commands.map((cmd, i) =>
    `<div class="slash-menu-item${i === state.slashMenuIndex ? ' active' : ''}" data-cmd="${cmd.id}" role="option" aria-selected="${i === state.slashMenuIndex}">` +
      `<span class="slash-menu-icon">${cmd.icon}</span>` +
      `<span class="slash-menu-label">${cmd.label}</span>` +
    `</div>`
  ).join('');
  menu.style.display = 'block';

  // Scroll active item into view
  const activeItem = menu.querySelector('.slash-menu-item.active');
  if (activeItem) activeItem.scrollIntoView({ block: 'nearest' });

  // Click handler for each item
  menu.querySelectorAll('.slash-menu-item').forEach(item => {
    item.addEventListener('mousedown', e => {
      e.preventDefault();
      executeSlashCommand(item.dataset.cmd);
    });
  });
}

function positionSlashMenu() {
  const sel = window.getSelection();
  if (!sel || !sel.rangeCount) return;
  const range = sel.getRangeAt(0);
  const rect = range.getBoundingClientRect();
  const menu = $('slashMenu');
  const canvas = $('editorCanvas');
  const canvasRect = canvas.getBoundingClientRect();

  let top = rect.bottom - canvasRect.top + canvas.scrollTop + 4;
  let left = rect.left - canvasRect.left;

  // Clamp within canvas bounds
  const menuW = 240;
  if (left + menuW > canvasRect.width) left = canvasRect.width - menuW - 8;
  if (left < 8) left = 8;

  menu.style.top = top + 'px';
  menu.style.left = left + 'px';
}

function openSlashMenu() {
  state.slashMenuOpen = true;
  state.slashMenuIndex = 0;
  state.slashQuery = '';
  const commands = filterSlashCommands('');
  renderSlashMenu(commands);
  positionSlashMenu();
}

function closeSlashMenu() {
  state.slashMenuOpen = false;
  state.slashQuery = '';
  state.slashMenuIndex = 0;
  $('slashMenu').style.display = 'none';
}

function updateSlashFilter(query) {
  state.slashQuery = query;
  state.slashMenuIndex = 0;
  const commands = filterSlashCommands(query);
  renderSlashMenu(commands);
  if (commands.length === 0) closeSlashMenu();
}

function deleteSlashText() {
  // Delete the "/" and any typed query text from the paragraph
  const el = getActiveElement();
  if (!el) return;
  const offset = getCursorOffset(el);
  const slashLen = 1 + state.slashQuery.length; // "/" + query
  const deleteFrom = Math.max(0, offset - slashLen);

  // Remove text by manipulating textContent
  const text = el.textContent || '';
  const chars = Array.from(text);
  chars.splice(deleteFrom, slashLen);
  el.textContent = chars.join('') || '';
  if (!el.textContent) el.innerHTML = '<br>';

  // Sync and restore cursor
  syncParagraphText(el);
  if (el.textContent && deleteFrom > 0) setCursorAtOffset(el, deleteFrom);
  else setCursorAtStart(el);
}

function executeSlashCommand(cmdId) {
  const doc = state.doc;
  if (!doc) { closeSlashMenu(); return; }

  const el = getActiveElement();
  const nodeId = el?.dataset?.nodeId;
  if (!nodeId) { closeSlashMenu(); return; }

  // Delete the slash text first
  deleteSlashText();
  closeSlashMenu();

  syncAllText();

  try {
    switch (cmdId) {
      case 'heading1': doc.set_heading_level(nodeId, 1); renderDocument(); break;
      case 'heading2': doc.set_heading_level(nodeId, 2); renderDocument(); break;
      case 'heading3': doc.set_heading_level(nodeId, 3); renderDocument(); break;
      case 'bullet':   doc.set_list_format(nodeId, 'bullet', 0); renderDocument(); break;
      case 'numbered': doc.set_list_format(nodeId, 'decimal', 0); renderDocument(); break;
      case 'table':
        doc.insert_table(nodeId, 3, 3);
        renderDocument();
        break;
      case 'image':
        $('imageInput').click();
        break;
      case 'hr':
        doc.insert_horizontal_rule(nodeId);
        renderDocument();
        break;
      case 'pagebreak':
        doc.insert_page_break(nodeId);
        renderDocument();
        break;
      case 'quote': {
        doc.set_heading_level(nodeId, 0);
        const textLen = el ? Array.from(el.textContent || '').length : 0;
        if (textLen > 0) {
          doc.format_selection(nodeId, 0, nodeId, textLen, 'italic', 'true');
          doc.format_selection(nodeId, 0, nodeId, textLen, 'color', '666666');
        }
        renderDocument();
        break;
      }
      case 'code': {
        doc.set_heading_level(nodeId, 0);
        const codeLen = el ? Array.from(el.textContent || '').length : 0;
        if (codeLen > 0) {
          doc.format_selection(nodeId, 0, nodeId, codeLen, 'fontFamily', 'Courier New');
          doc.format_selection(nodeId, 0, nodeId, codeLen, 'fontSize', '11');
        }
        renderDocument();
        break;
      }
    }
    updateToolbarState();
    updateUndoRedo();
  } catch (e) { console.error('slash command:', e); }
}

export { closeSlashMenu };

// Expose for toolbar buttons
export { doUndo, doRedo };

// E10.2: Zoom via keyboard (Ctrl+=/Ctrl+-/Ctrl+0)
function adjustEditorZoom(delta) {
  if (delta === 0) {
    state.zoomLevel = 100;
  } else {
    state.zoomLevel = Math.max(50, Math.min(200, (state.zoomLevel || 100) + delta));
  }
  const zoomEl = $('zoomValue');
  if (zoomEl) zoomEl.textContent = state.zoomLevel + '%';
  const page = $('docPage');
  if (page) {
    page.style.transform = `scale(${state.zoomLevel / 100})`;
    page.style.transformOrigin = 'top center';
  }
}
