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

  // ─── Regular input (typing) ─────────────────────
  page.addEventListener('input', (e) => {
    if (state.ignoreInput) return;
    const el = getActiveElement();
    if (el) debouncedSync(el);

    // ── Slash menu: detect "/" or update filter ──
    if (state.slashMenuOpen) {
      const text = el?.textContent || '';
      const offset = getCursorOffset(el);
      // Find the "/" that triggered the menu
      const before = text.substring(0, offset);
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
        const charBefore = offset >= 2 ? text[offset - 2] : null;
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
      if (e.key === 'Backspace') {
        // If query is empty, the "/" itself will be deleted, so close menu
        if (state.slashQuery.length === 0) {
          closeSlashMenu();
          // Let backspace proceed to delete the "/"
        }
        // Otherwise let it proceed normally; the input handler will update the filter
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
        const updated = renderNodeById(nodeId);
        if (updated) setCursorAtOffset(updated, offset + 1);
        state.pagesRendered = false; updatePageBreaks(); updateUndoRedo();
      } catch (_) {
        // Fallback: insert newline character directly
        document.execCommand('insertLineBreak');
      }
      return;
    }

    // ── Enter — split paragraph ──
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
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
        renderDocument();
        // Place cursor at end of pasted content
        const lines = text.split('\n');
        const lastLine = lines[lines.length - 1];
        // Find the last paragraph — it should have the last line's text
        const allEls = Array.from(page.querySelectorAll('p[data-node-id], h1[data-node-id], h2[data-node-id], h3[data-node-id]'));
        if (allEls.length > 0) {
          const lastEl = allEls[allEls.length - 1];
          setCursorAtOffset(lastEl, Array.from(lastLine).length);
        }
        updateUndoRedo();
        markDirty();
      } catch (err) {
        console.error('paste multi-line:', err);
        // Fallback: insert as single line
        try {
          doc.insert_text_in_paragraph(info.startNodeId, info.startOffset, text.replace(/\n/g, ' '));
          renderDocument();
          updateUndoRedo();
        } catch (_) {
          insertTextAtCursor(text.replace(/\n/g, ' '));
        }
      }
    } else {
      try {
        doc.insert_text_in_paragraph(info.startNodeId, info.startOffset, text);
        const updated = renderNodeById(info.startNodeId);
        if (updated) setCursorAtOffset(updated, info.startOffset + Array.from(text).length);
        updateUndoRedo();
        markDirty();
      } catch (_) {
        insertTextAtCursor(text);
      }
    }
  });

  // ─── Selection change ──────────────────────────
  document.addEventListener('selectionchange', updateToolbarState);

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

function insertTextAtCursor(text) {
  const sel = window.getSelection();
  if (!sel?.rangeCount) return;
  const range = sel.getRangeAt(0);
  range.deleteContents();
  range.insertNode(document.createTextNode(text));
  range.collapse(false);
  sel.removeAllRanges(); sel.addRange(range);
  const el = getActiveElement();
  if (el) debouncedSync(el);
}

function doUndo() {
  if (!state.doc) return;
  clearTimeout(state.syncTimer);
  syncAllText();
  try { state.doc.undo(); renderDocument(); updateToolbarState(); }
  catch (e) { console.error('undo:', e); }
}

function doRedo() {
  if (!state.doc) return;
  try { state.doc.redo(); renderDocument(); updateToolbarState(); }
  catch (e) { console.error('redo:', e); }
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
