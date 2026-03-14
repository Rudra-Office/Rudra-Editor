// Toolbar event handler wiring
import { state, $ } from './state.js';
import { toggleFormat, applyFormat, updateToolbarState, updateUndoRedo } from './toolbar.js';
import { doUndo, doRedo, closeSlashMenu } from './input.js';
import { renderDocument, renderNodeById, syncParagraphText, syncAllText } from './render.js';
import { getSelectionInfo, setCursorAtOffset, setSelectionRange, getActiveNodeId } from './selection.js';
import { insertImage } from './images.js';
import { updatePageBreaks } from './pagination.js';
import { renderRuler } from './ruler.js';
import { getVersions, restoreVersion, saveVersion } from './file.js';
import { showShareDialog, broadcastOp } from './collab.js';

export function initToolbar() {
  // Format toggles
  $('btnBold').addEventListener('click', () => toggleFormat('bold'));
  $('btnItalic').addEventListener('click', () => toggleFormat('italic'));
  $('btnUnderline').addEventListener('click', () => toggleFormat('underline'));
  $('btnStrike').addEventListener('click', () => toggleFormat('strikethrough'));
  $('btnSuperscript').addEventListener('click', () => toggleFormat('superscript'));
  $('btnSubscript').addEventListener('click', () => toggleFormat('subscript'));

  // Clear formatting
  $('btnClearFormat').addEventListener('click', () => {
    if (!state.doc) return;
    const info = getSelectionInfo();
    if (!info) return;
    syncAllText();
    try {
      const keys = ['bold', 'italic', 'underline', 'strikethrough', 'superscript', 'subscript', 'color', 'highlight', 'fontFamily', 'fontSize'];
      let sn, so, en, eo;
      if (info.collapsed) {
        const el = $('docPage').querySelector(`[data-node-id="${info.startNodeId}"]`);
        const textLen = el ? Array.from(el.textContent || '').length : 0;
        sn = info.startNodeId; so = 0; en = info.startNodeId; eo = textLen;
      } else {
        sn = info.startNodeId; so = info.startOffset; en = info.endNodeId; eo = info.endOffset;
      }
      keys.forEach(k => {
        try {
          if (eo > 0 || sn !== en) state.doc.format_selection(sn, so, en, eo, k, 'false');
        } catch (_) {}
      });
      keys.forEach(k => {
        broadcastOp({ action: 'formatSelection', startNode: sn, startOffset: so, endNode: en, endOffset: eo, key: k, value: 'false' });
      });
      renderDocument();
      updateToolbarState();
      updateUndoRedo();
    } catch (e) { console.error('clear format:', e); }
  });

  // Undo/Redo
  $('btnUndo').addEventListener('click', doUndo);
  $('btnRedo').addEventListener('click', doRedo);

  // Print
  $('btnPrint').addEventListener('click', () => {
    window.print();
  });

  // Font family
  $('fontFamily').addEventListener('change', e => {
    if (e.target.value) applyFormat('fontFamily', e.target.value);
  });

  // Font size
  $('fontSize').addEventListener('change', e => {
    const v = parseInt(e.target.value);
    if (v >= 6 && v <= 96) applyFormat('fontSize', String(v));
  });

  // Style gallery dropdown
  initStyleGallery();

  // Text color
  $('colorPicker').addEventListener('input', e => {
    const hex = e.target.value.replace('#', '');
    $('colorSwatch').style.background = '#' + hex;
    applyFormat('color', hex);
  });

  // Highlight color
  $('highlightPicker').addEventListener('input', e => {
    const hex = e.target.value.replace('#', '');
    applyFormat('highlight', hex);
  });

  // Line spacing
  $('lineSpacing').addEventListener('change', e => {
    if (!state.doc) return;
    const info = getSelectionInfo();
    if (!info) return;
    syncAllText();
    try {
      state.doc.set_line_spacing(info.startNodeId, e.target.value);
      broadcastOp({ action: 'setLineSpacing', nodeId: info.startNodeId, value: e.target.value });
      renderNodeById(info.startNodeId);
      state.pagesRendered = false;
      updatePageBreaks();
      updateUndoRedo();
    } catch (err) { console.error('line spacing:', err); }
  });

  // Indent / Outdent
  $('btnIndent').addEventListener('click', () => applyIndent(36));   // +0.5in (36pt)
  $('btnOutdent').addEventListener('click', () => applyIndent(-36)); // -0.5in

  // Alignment
  $('btnAlignL').addEventListener('click', () => applyAlignment('left'));
  $('btnAlignC').addEventListener('click', () => applyAlignment('center'));
  $('btnAlignR').addEventListener('click', () => applyAlignment('right'));
  $('btnAlignJ').addEventListener('click', () => applyAlignment('justify'));

  // Lists
  $('btnBulletList').addEventListener('click', () => toggleList('bullet'));
  $('btnNumberList').addEventListener('click', () => toggleList('decimal'));

  // Insert menu
  $('btnInsertMenu').addEventListener('click', e => {
    e.stopPropagation();
    const menu = $('insertMenu');
    menu.classList.toggle('show');
    $('btnInsertMenu').setAttribute('aria-expanded', menu.classList.contains('show') ? 'true' : 'false');
  });

  // Insert table
  $('miTable').addEventListener('click', () => {
    $('insertMenu').classList.remove('show');
    $('tableModal').classList.add('show');
    $('tableRows').focus();
  });
  $('tableCancelBtn').addEventListener('click', () => {
    $('tableModal').classList.remove('show');
    $('docPage').focus();
  });
  $('tableInsertBtn').addEventListener('click', () => {
    const rows = parseInt($('tableRows').value) || 3;
    const cols = parseInt($('tableCols').value) || 3;
    if (rows < 1 || rows > 100 || cols < 1 || cols > 50) {
      alert('Rows must be 1-100, columns must be 1-50.');
      return;
    }
    $('tableModal').classList.remove('show');
    if (!state.doc) return;
    const nodeId = getActiveNodeId();
    if (!nodeId) return;
    syncAllText();
    try {
      state.doc.insert_table(nodeId, rows, cols);
      broadcastOp({ action: 'insertTable', afterNodeId: nodeId, rows, cols });
      renderDocument();
      updateUndoRedo();
    } catch (e) { console.error('insert table:', e); }
  });
  // Modal backdrop click to close
  $('tableModal').addEventListener('click', e => {
    if (e.target === $('tableModal')) {
      $('tableModal').classList.remove('show');
      $('docPage').focus();
    }
  });

  // Insert image
  $('miImage').addEventListener('click', () => {
    $('insertMenu').classList.remove('show');
    $('imageInput').click();
  });
  $('imageInput').addEventListener('change', e => {
    const f = e.target.files[0];
    if (f) insertImage(f);
    e.target.value = '';
  });

  // Insert hyperlink — modal
  $('miLink').addEventListener('click', () => {
    $('insertMenu').classList.remove('show');
    if (!state.doc) return;
    const info = getSelectionInfo();
    if (!info) return;
    state._linkSelInfo = info; // stash selection for after modal
    $('linkUrl').value = '';
    $('linkModal').classList.add('show');
    $('linkUrl').focus();
  });
  $('linkCancelBtn').addEventListener('click', () => {
    $('linkModal').classList.remove('show');
    $('docPage').focus();
  });
  $('linkInsertBtn').addEventListener('click', () => {
    let url = $('linkUrl').value.trim();
    if (!url) { $('linkModal').classList.remove('show'); return; }
    if (!/^https?:\/\//i.test(url) && !url.startsWith('#')) url = 'https://' + url;
    try { new URL(url); } catch (_) {
      $('linkUrl').style.borderColor = 'var(--danger)';
      setTimeout(() => { $('linkUrl').style.borderColor = ''; }, 1500);
      return;
    }
    $('linkModal').classList.remove('show');
    try { applyFormat('hyperlinkUrl', url); }
    catch (e) { console.error('hyperlink:', e); }
  });
  $('linkModal').addEventListener('click', e => {
    if (e.target === $('linkModal')) { $('linkModal').classList.remove('show'); $('docPage').focus(); }
  });
  $('linkUrl').addEventListener('keydown', e => {
    if (e.key === 'Enter') { e.preventDefault(); $('linkInsertBtn').click(); }
    if (e.key === 'Escape') { $('linkModal').classList.remove('show'); $('docPage').focus(); }
  });

  // Insert horizontal rule
  $('miHR').addEventListener('click', () => {
    $('insertMenu').classList.remove('show');
    if (!state.doc) return;
    const nodeId = getActiveNodeId();
    if (!nodeId) return;
    syncAllText();
    try {
      state.doc.insert_horizontal_rule(nodeId);
      broadcastOp({ action: 'insertHR', afterNodeId: nodeId });
      renderDocument();
      updateUndoRedo();
    } catch (e) { console.error('insert HR:', e); }
  });

  // Insert page break
  $('miPageBreak').addEventListener('click', () => {
    $('insertMenu').classList.remove('show');
    if (!state.doc) return;
    const nodeId = getActiveNodeId();
    if (!nodeId) return;
    syncAllText();
    try {
      state.doc.insert_page_break(nodeId);
      broadcastOp({ action: 'insertPageBreak', afterNodeId: nodeId });
      renderDocument();
      updateUndoRedo();
    } catch (e) { console.error('insert page break:', e); }
  });

  // Insert comment — modal
  $('miComment').addEventListener('click', () => {
    $('insertMenu').classList.remove('show');
    if (!state.doc) return;
    const info = getSelectionInfo();
    if (!info) return;
    state._commentSelInfo = info;
    $('commentText').value = '';
    $('commentAuthor').value = 'User';
    $('commentModal').classList.add('show');
    $('commentText').focus();
  });
  $('commentCancelBtn').addEventListener('click', () => {
    $('commentModal').classList.remove('show');
    $('docPage').focus();
  });
  $('commentInsertBtn').addEventListener('click', () => {
    const text = $('commentText').value.trim();
    if (!text) { $('commentModal').classList.remove('show'); return; }
    const author = $('commentAuthor').value.trim() || 'User';
    $('commentModal').classList.remove('show');
    const info = state._commentSelInfo;
    if (!info || !state.doc) return;
    try {
      state.doc.insert_comment(info.startNodeId, info.endNodeId, author, text);
      renderDocument();
      updateUndoRedo();
      refreshComments();
    } catch (e) { console.error('insert comment:', e); }
  });
  $('commentModal').addEventListener('click', e => {
    if (e.target === $('commentModal')) { $('commentModal').classList.remove('show'); $('docPage').focus(); }
  });
  $('commentText').addEventListener('keydown', e => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) { e.preventDefault(); $('commentInsertBtn').click(); }
    if (e.key === 'Escape') { $('commentModal').classList.remove('show'); $('docPage').focus(); }
  });

  // Comments panel toggle
  $('btnComments').addEventListener('click', () => {
    $('commentsPanel').classList.toggle('show');
    if ($('commentsPanel').classList.contains('show')) refreshComments();
  });
  $('commentsClose').addEventListener('click', () => {
    $('commentsPanel').classList.remove('show');
  });

  // Find toolbar button
  $('btnFind').addEventListener('click', () => {
    $('findBar').classList.add('show');
    $('findInput').focus();
  });

  // Spell check toggle
  $('btnSpellCheck').addEventListener('click', () => {
    const page = $('docPage');
    const enabled = page.getAttribute('spellcheck') === 'true';
    page.setAttribute('spellcheck', enabled ? 'false' : 'true');
    const btn = $('btnSpellCheck');
    btn.classList.toggle('active', !enabled);
    btn.setAttribute('aria-pressed', String(!enabled));
  });

  // Version history panel
  $('btnHistory').addEventListener('click', () => {
    const panel = $('historyPanel');
    panel.classList.toggle('show');
    if (panel.classList.contains('show')) refreshHistory();
  });
  $('historyClose').addEventListener('click', () => {
    $('historyPanel').classList.remove('show');
  });

  // Share / Collaboration
  $('btnShare').addEventListener('click', showShareDialog);

  // Zoom controls
  $('zoomIn').addEventListener('click', () => adjustZoom(10));
  $('zoomOut').addEventListener('click', () => adjustZoom(-10));

  // Table context menu
  initTableContextMenu();

  // Close menus on outside click
  document.addEventListener('click', e => {
    if (!e.target.closest('.insert-dropdown')) {
      $('insertMenu').classList.remove('show');
      $('btnInsertMenu').setAttribute('aria-expanded', 'false');
    }
    if (!e.target.closest('.style-gallery')) {
      $('styleGalleryPanel').classList.remove('show');
      $('styleGalleryBtn').setAttribute('aria-expanded', 'false');
    }
    $('tableContextMenu').style.display = 'none';
    // Close slash menu on outside click
    if (!e.target.closest('.slash-menu') && !e.target.closest('.doc-page')) {
      closeSlashMenu();
    }
  });
}

function applyAlignment(align) {
  if (!state.doc) return;
  const info = getSelectionInfo();
  if (!info) return;
  const el = $('docPage').querySelector(`[data-node-id="${info.startNodeId}"]`);
  if (el) syncParagraphText(el);
  try {
    state.doc.set_alignment(info.startNodeId, align);
    broadcastOp({ action: 'setAlignment', nodeId: info.startNodeId, alignment: align });
    const updated = renderNodeById(info.startNodeId);
    if (updated) setCursorAtOffset(updated, info.startOffset);
    state.pagesRendered = false;
    updatePageBreaks();
    updateToolbarState();
    updateUndoRedo();
  } catch (e) { console.error('alignment:', e); }
}

function toggleList(format) {
  if (!state.doc) return;
  const info = getSelectionInfo();
  if (!info) return;
  syncAllText();
  try {
    state.doc.set_list_format(info.startNodeId, format, 0);
    broadcastOp({ action: 'setListFormat', nodeId: info.startNodeId, format, level: 0 });
    renderDocument();
    updateToolbarState();
    updateUndoRedo();
  } catch (e) { console.error('list:', e); }
}

function applyIndent(delta) {
  if (!state.doc) return;
  const info = getSelectionInfo();
  if (!info) return;
  syncAllText();
  try {
    // Get current indent, add delta, clamp to 0
    const fmt = JSON.parse(state.doc.get_formatting_json(info.startNodeId));
    const current = parseFloat(fmt.indentLeft || '0');
    const newVal = Math.max(0, current + delta);
    state.doc.set_indent(info.startNodeId, 'left', newVal);
    broadcastOp({ action: 'setIndent', nodeId: info.startNodeId, side: 'left', value: newVal });
    renderNodeById(info.startNodeId);
    state.pagesRendered = false;
    updatePageBreaks();
    updateUndoRedo();
  } catch (e) { console.error('indent:', e); }
}

function adjustZoom(delta) {
  state.zoomLevel = (state.zoomLevel || 100) + delta;
  state.zoomLevel = Math.max(50, Math.min(200, state.zoomLevel));
  $('zoomValue').textContent = state.zoomLevel + '%';
  const page = $('docPage');
  if (page) {
    page.style.transform = `scale(${state.zoomLevel / 100})`;
    page.style.transformOrigin = 'top center';
  }
  renderRuler();
}

// ─── Comment Replies (in-memory store) ────────────
// Replies stored in-memory keyed by parent comment ID.
// Each reply: { id, parentId, author, text, timestamp }
if (!state.commentReplies) state.commentReplies = [];
let _replyCounter = 0;

function refreshComments() {
  const list = $('commentsList');
  if (!list || !state.doc) return;
  try {
    const comments = JSON.parse(state.doc.get_comments_json());
    const replies = state.commentReplies || [];

    if ((!comments || comments.length === 0) && replies.length === 0) {
      list.innerHTML = '<div class="comments-empty">No comments in this document.</div>';
      return;
    }

    // Build reply map: parentId -> [reply, ...]
    const replyMap = {};
    replies.forEach(r => {
      if (!replyMap[r.parentId]) replyMap[r.parentId] = [];
      replyMap[r.parentId].push(r);
    });

    let html = '';
    (comments || []).forEach(c => {
      const cid = c.id || '';
      html += renderCommentCard(c);

      // Render replies for this comment
      const threadReplies = replyMap[cid] || [];
      threadReplies.sort((a, b) => a.timestamp - b.timestamp);
      threadReplies.forEach(r => {
        html += renderReplyCard(r);
      });

      // Reply form placeholder
      html += `<div class="comment-reply-area" data-parent-id="${escapeAttr(cid)}"></div>`;
    });

    list.innerHTML = html;

    // Wire up delete buttons for WASM comments
    list.querySelectorAll('.comment-delete').forEach(btn => {
      btn.addEventListener('click', () => {
        const id = btn.dataset.id;
        if (!id || !state.doc) return;
        try {
          state.doc.delete_comment(id);
          // Also remove any replies to this comment
          state.commentReplies = (state.commentReplies || []).filter(r => r.parentId !== id);
          renderDocument();
          updateUndoRedo();
          refreshComments();
        } catch (e) { console.error('delete comment:', e); }
      });
    });

    // Wire up reply buttons
    list.querySelectorAll('.comment-reply-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const parentId = btn.dataset.parentId;
        showReplyForm(parentId);
      });
    });

    // Wire up delete buttons for replies
    list.querySelectorAll('.reply-delete').forEach(btn => {
      btn.addEventListener('click', () => {
        const replyId = btn.dataset.replyId;
        state.commentReplies = (state.commentReplies || []).filter(r => r.id !== replyId);
        refreshComments();
      });
    });
  } catch (e) {
    list.innerHTML = '<div class="comments-empty">Unable to load comments.</div>';
  }
}

function renderCommentCard(c) {
  const cid = c.id || '';
  return `
    <div class="comment-card" data-comment-id="${escapeAttr(cid)}">
      <div class="comment-author">${escapeHtml(c.author || 'Unknown')}</div>
      ${c.date ? `<div class="comment-date">${escapeHtml(c.date)}</div>` : ''}
      <div class="comment-text">${escapeHtml(c.text || c.body || '')}</div>
      <div class="comment-actions">
        <button class="comment-reply-btn" data-parent-id="${escapeAttr(cid)}">Reply</button>
        <button class="comment-delete" data-id="${escapeAttr(cid)}">Delete</button>
      </div>
    </div>`;
}

function renderReplyCard(r) {
  return `
    <div class="comment-card comment-reply" data-reply-id="${escapeAttr(r.id)}">
      <div class="comment-author">${escapeHtml(r.author || 'Unknown')}</div>
      <div class="comment-text">${escapeHtml(r.text)}</div>
      <div class="comment-actions">
        <button class="reply-delete" data-reply-id="${escapeAttr(r.id)}">Delete</button>
      </div>
    </div>`;
}

function showReplyForm(parentId) {
  const area = $('commentsList').querySelector(`.comment-reply-area[data-parent-id="${parentId}"]`);
  if (!area) return;
  // If already showing a form, remove it
  if (area.querySelector('.comment-reply-form')) {
    area.innerHTML = '';
    return;
  }
  area.innerHTML = `
    <div class="comment-reply-form">
      <input class="comment-reply-input" type="text" placeholder="Write a reply..." autocomplete="off">
      <div class="comment-reply-form-actions">
        <button class="comment-reply-submit">Post</button>
        <button class="comment-reply-cancel">Cancel</button>
      </div>
    </div>`;

  const input = area.querySelector('.comment-reply-input');
  input.focus();

  // Submit on Enter
  input.addEventListener('keydown', e => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submitReply(parentId, input.value);
    }
    if (e.key === 'Escape') {
      area.innerHTML = '';
    }
  });

  area.querySelector('.comment-reply-submit').addEventListener('click', () => {
    submitReply(parentId, input.value);
  });
  area.querySelector('.comment-reply-cancel').addEventListener('click', () => {
    area.innerHTML = '';
  });
}

function submitReply(parentId, text) {
  if (!text || !text.trim()) return;
  const author = 'User';
  const reply = {
    id: 'reply-' + (++_replyCounter) + '-' + Date.now(),
    parentId,
    author,
    text: text.trim(),
    timestamp: Date.now(),
  };
  if (!state.commentReplies) state.commentReplies = [];
  state.commentReplies.push(reply);
  refreshComments();
}

function escapeHtml(s) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
function escapeAttr(s) {
  return s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

// ── Version History ──────────────────────────────
function formatVersionDate(ts) {
  const d = new Date(ts);
  const months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
  const month = months[d.getMonth()];
  const day = d.getDate();
  let hours = d.getHours();
  const mins = d.getMinutes().toString().padStart(2, '0');
  const ampm = hours >= 12 ? 'PM' : 'AM';
  hours = hours % 12 || 12;
  return `${month} ${day}, ${hours}:${mins} ${ampm}`;
}

function refreshHistory() {
  const list = $('historyList');
  if (!list) return;
  list.innerHTML = '<div class="history-loading">Loading versions...</div>';
  getVersions().then(versions => {
    if (!versions || versions.length === 0) {
      list.innerHTML = '<div class="history-empty">No saved versions yet. Versions are saved automatically every 5 minutes and on manual save (Ctrl+S).</div>';
      return;
    }
    list.innerHTML = versions.map((v, i) => `
      <div class="version-card" data-version-id="${v.id}">
        <div class="version-info">
          <div class="version-date">${escapeHtml(formatVersionDate(v.timestamp))}</div>
          <div class="version-meta">${v.wordCount.toLocaleString()} word${v.wordCount !== 1 ? 's' : ''}${v.label ? ' &middot; ' + escapeHtml(v.label) : ''}</div>
          ${i === 0 ? '<span class="version-badge">Current version</span>' : ''}
        </div>
        ${i > 0 ? '<div class="version-actions"><button class="version-restore" data-id="' + v.id + '">Restore</button></div>' : ''}
      </div>
    `).join('');
    list.querySelectorAll('.version-restore').forEach(btn => {
      btn.addEventListener('click', () => {
        const id = parseInt(btn.dataset.id);
        if (!id || !state.engine) return;
        if (!confirm('Restore this version? Current unsaved changes will be lost.')) return;
        restoreVersion(id).then(() => {
          refreshHistory();
        }).catch(e => {
          alert('Failed to restore version: ' + e.message);
          console.error('restore version:', e);
        });
      });
    });
  });
}

// ── Style Gallery ─────────────────────────────────
// Style definitions: heading level + font/size/color for each style
const STYLE_DEFS = {
  normal:   { heading: 0, fontSize: null, fontFamily: null, color: null, italic: false },
  title:    { heading: 0, fontSize: '26', fontFamily: null, color: null, italic: false },
  subtitle: { heading: 0, fontSize: '15', fontFamily: null, color: '666666', italic: false },
  heading1: { heading: 1, fontSize: null, fontFamily: null, color: null, italic: false },
  heading2: { heading: 2, fontSize: null, fontFamily: null, color: null, italic: false },
  heading3: { heading: 3, fontSize: null, fontFamily: null, color: null, italic: false },
  heading4: { heading: 4, fontSize: null, fontFamily: null, color: null, italic: false },
  quote:    { heading: 0, fontSize: null, fontFamily: null, color: '666666', italic: true },
  code:     { heading: 0, fontSize: '11', fontFamily: 'Courier New', color: null, italic: false },
};

function initStyleGallery() {
  const btn = $('styleGalleryBtn');
  const panel = $('styleGalleryPanel');

  // Toggle panel on button click
  btn.addEventListener('click', e => {
    e.stopPropagation();
    panel.classList.toggle('show');
    btn.setAttribute('aria-expanded', panel.classList.contains('show') ? 'true' : 'false');
  });

  // Handle style item clicks
  panel.querySelectorAll('.style-gallery-item').forEach(item => {
    item.addEventListener('click', () => {
      const styleName = item.dataset.style;
      if (!styleName || !state.doc) { panel.classList.remove('show'); return; }
      const info = getSelectionInfo();
      if (!info) { panel.classList.remove('show'); return; }

      const el = $('docPage').querySelector(`[data-node-id="${info.startNodeId}"]`);
      if (el) syncParagraphText(el);

      const def = STYLE_DEFS[styleName];
      if (!def) { panel.classList.remove('show'); return; }

      try {
        // Set heading level
        state.doc.set_heading_level(info.startNodeId, def.heading);
        broadcastOp({ action: 'setHeading', nodeId: info.startNodeId, level: def.heading });

        // Apply font size (whole paragraph)
        const textLen = el ? Array.from(el.textContent || '').length : 0;
        if (textLen > 0) {
          if (def.fontSize) {
            state.doc.format_selection(info.startNodeId, 0, info.startNodeId, textLen, 'fontSize', def.fontSize);
          }
          if (def.fontFamily) {
            state.doc.format_selection(info.startNodeId, 0, info.startNodeId, textLen, 'fontFamily', def.fontFamily);
          }
          if (def.color) {
            state.doc.format_selection(info.startNodeId, 0, info.startNodeId, textLen, 'color', def.color);
          }
          if (def.italic) {
            state.doc.format_selection(info.startNodeId, 0, info.startNodeId, textLen, 'italic', 'true');
          } else {
            // Clear italic if switching away from quote
            try { state.doc.format_selection(info.startNodeId, 0, info.startNodeId, textLen, 'italic', 'false'); } catch(_) {}
          }
        }

        renderDocument();
        updateToolbarState();
        updateUndoRedo();
      } catch (err) { console.error('style gallery:', err); }

      panel.classList.remove('show');
      btn.setAttribute('aria-expanded', 'false');
    });
  });
}

function initTableContextMenu() {
  $('docPage').addEventListener('contextmenu', e => {
    const cell = e.target.closest('td, th');
    if (!cell || !state.doc) return;
    e.preventDefault();

    const table = cell.closest('table');
    const tableEl = table?.closest('[data-node-id]');
    if (!tableEl) return;

    const row = cell.parentElement;
    const rowIndex = Array.from(row.parentElement.children).indexOf(row);
    const colIndex = Array.from(row.children).indexOf(cell);

    state.ctxTable = tableEl.dataset.nodeId;
    state.ctxCell = cell.closest('[data-node-id]')?.dataset.nodeId;
    state.ctxRow = rowIndex;
    state.ctxCol = colIndex;

    const menu = $('tableContextMenu');
    menu.style.display = 'block';
    // Position with viewport boundary check
    const menuW = 200, menuH = 280;
    const x = Math.min(e.clientX, window.innerWidth - menuW);
    const y = Math.min(e.clientY, window.innerHeight - menuH);
    menu.style.left = Math.max(0, x) + 'px';
    menu.style.top = Math.max(0, y) + 'px';
  });

  const cmAction = (id, fn) => {
    $(id).addEventListener('click', () => {
      $('tableContextMenu').style.display = 'none';
      if (!state.doc || !state.ctxTable) return;
      syncAllText();
      try { fn(); renderDocument(); updateUndoRedo(); }
      catch (e) { console.error('table op:', e); }
    });
  };

  cmAction('cmInsertRowAbove', () => { state.doc.insert_table_row(state.ctxTable, state.ctxRow); broadcastOp({ action: 'insertTableRow', tableId: state.ctxTable, index: state.ctxRow }); });
  cmAction('cmInsertRowBelow', () => { state.doc.insert_table_row(state.ctxTable, state.ctxRow + 1); broadcastOp({ action: 'insertTableRow', tableId: state.ctxTable, index: state.ctxRow + 1 }); });
  cmAction('cmDeleteRow', () => { state.doc.delete_table_row(state.ctxTable, state.ctxRow); broadcastOp({ action: 'deleteTableRow', tableId: state.ctxTable, index: state.ctxRow }); });
  cmAction('cmInsertColLeft', () => { state.doc.insert_table_column(state.ctxTable, state.ctxCol); broadcastOp({ action: 'insertTableColumn', tableId: state.ctxTable, index: state.ctxCol }); });
  cmAction('cmInsertColRight', () => { state.doc.insert_table_column(state.ctxTable, state.ctxCol + 1); broadcastOp({ action: 'insertTableColumn', tableId: state.ctxTable, index: state.ctxCol + 1 }); });
  cmAction('cmDeleteCol', () => { state.doc.delete_table_column(state.ctxTable, state.ctxCol); broadcastOp({ action: 'deleteTableColumn', tableId: state.ctxTable, index: state.ctxCol }); });

  // Cell background — color picker instead of prompt
  $('cmCellBg').addEventListener('click', e => {
    e.preventDefault();
    e.stopPropagation();
    // Trigger the hidden color picker
    const picker = $('cmCellBgPicker');
    picker.style.pointerEvents = 'auto';
    picker.click();
  });
  $('cmCellBgPicker').addEventListener('input', e => {
    $('tableContextMenu').style.display = 'none';
    if (!state.doc || !state.ctxCell) return;
    const hex = e.target.value.replace('#', '');
    try {
      state.doc.set_cell_background(state.ctxCell, hex);
      broadcastOp({ action: 'setCellBackground', cellId: state.ctxCell, color: hex });
      renderDocument();
      updateUndoRedo();
    } catch (err) { console.error('cell bg:', err); }
  });
  $('cmCellBgPicker').addEventListener('change', () => {
    $('cmCellBgPicker').style.pointerEvents = 'none';
  });
}
