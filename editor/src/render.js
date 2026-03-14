// Document rendering — WASM → DOM
import { state, $ } from './state.js';
import { setupImages } from './images.js';
import { updatePageBreaks } from './pagination.js';
import { updateUndoRedo } from './toolbar.js';
import { markDirty, updateTrackChanges, updateStatusBar } from './file.js';
import { broadcastTextSync, broadcastOp } from './collab.js';

export function renderDocument() {
  const { doc } = state;
  if (!doc) return;
  try {
    // Tear down any existing virtual scroll before re-rendering
    teardownVirtualScroll();
    const html = doc.to_html();
    state.ignoreInput = true;
    const page = $('docPage');
    page.innerHTML = html;
    // Extract header/footer HTML from WASM output, then remove the elements.
    // The pagination system will render them per-page instead.
    const hdrEl = page.querySelector(':scope > header');
    const ftrEl = page.querySelector(':scope > footer');
    state.docHeaderHtml = hdrEl ? hdrEl.innerHTML : '';
    state.docFooterHtml = ftrEl ? ftrEl.innerHTML : '';
    if (hdrEl) hdrEl.remove();
    if (ftrEl) ftrEl.remove();
    fixEmptyBlocks();
    setupImages();
    cacheAllText();
    setupTrackChangeHandlers();
    state.ignoreInput = false;
    state.pagesRendered = false;
    updatePageBreaks();
    updateUndoRedo();
    updateTrackChanges();
    updateStatusBar();
    // E-19: Re-apply zoom level after full re-render (DOM is rebuilt)
    if (state.zoomLevel && state.zoomLevel !== 100) {
      page.style.transform = `scale(${state.zoomLevel / 100})`;
      page.style.transformOrigin = 'top center';
    }
    // Activate virtual scrolling for large documents
    maybeInitVirtualScroll();
    // E1.5: Refresh find highlights after full re-render
    state._onTextChanged?.();
  } catch (e) { console.error('Render error:', e); }
}

// ─── Per-change Track Changes popup ─────────────────
function setupTrackChangeHandlers() {
  const page = $('docPage');
  const tcElements = page.querySelectorAll('[data-tc-node-id]');
  if (tcElements.length === 0) return;

  tcElements.forEach(el => {
    el.style.cursor = 'pointer';
    el.addEventListener('click', e => {
      e.stopPropagation();
      showTcPopup(el);
    });
  });
}

function dismissTcPopup() {
  const existing = document.getElementById('tcPopup');
  if (existing) existing.remove();
}

function showTcPopup(el) {
  dismissTcPopup();

  const nodeId = el.dataset.tcNodeId;
  const tcType = el.dataset.tcType;
  if (!nodeId || !state.doc) return;

  const popup = document.createElement('div');
  popup.id = 'tcPopup';
  popup.className = 'tc-popup';

  const label = document.createElement('span');
  label.className = 'tc-popup-label';
  label.textContent = tcType === 'insert' ? 'Insertion' : tcType === 'delete' ? 'Deletion' : 'Format change';
  popup.appendChild(label);

  const acceptBtn = document.createElement('button');
  acceptBtn.className = 'tc-popup-btn tc-popup-accept';
  acceptBtn.innerHTML = '&#10003; Accept';
  acceptBtn.title = 'Accept this change';
  acceptBtn.addEventListener('click', e => {
    e.stopPropagation();
    dismissTcPopup();
    try {
      state.doc.accept_change(nodeId);
      broadcastOp({ action: 'acceptChange', nodeId });
      renderDocument();
    } catch (err) { console.error('accept change:', err); }
  });
  popup.appendChild(acceptBtn);

  const rejectBtn = document.createElement('button');
  rejectBtn.className = 'tc-popup-btn tc-popup-reject';
  rejectBtn.innerHTML = '&#10007; Reject';
  rejectBtn.title = 'Reject this change';
  rejectBtn.addEventListener('click', e => {
    e.stopPropagation();
    dismissTcPopup();
    try {
      state.doc.reject_change(nodeId);
      broadcastOp({ action: 'rejectChange', nodeId });
      renderDocument();
    } catch (err) { console.error('reject change:', err); }
  });
  popup.appendChild(rejectBtn);

  document.body.appendChild(popup);

  // Position near the element
  const rect = el.getBoundingClientRect();
  const popupW = 200;
  let left = rect.left + (rect.width / 2) - (popupW / 2);
  let top = rect.bottom + 6;

  // Keep within viewport
  if (left < 8) left = 8;
  if (left + popupW > window.innerWidth - 8) left = window.innerWidth - popupW - 8;
  if (top + 40 > window.innerHeight) top = rect.top - 44;

  popup.style.left = left + 'px';
  popup.style.top = top + 'px';

  // Dismiss on outside click
  const dismiss = (e) => {
    if (!popup.contains(e.target)) {
      dismissTcPopup();
      document.removeEventListener('click', dismiss, true);
    }
  };
  setTimeout(() => document.addEventListener('click', dismiss, true), 0);
}

export function renderNodeById(nodeIdStr) {
  const { doc } = state;
  if (!doc) return null;
  try {
    const html = doc.render_node_html(nodeIdStr);
    // E-05: Always re-query the DOM to get the current element, avoiding stale references
    const el = $('docPage').querySelector(`[data-node-id="${nodeIdStr}"]`);
    if (!el) return null;
    const temp = document.createElement('div');
    temp.innerHTML = html;
    const newEl = temp.firstElementChild;
    if (!newEl) return null;
    if (!newEl.innerHTML.trim()) newEl.innerHTML = '<br>';
    el.replaceWith(newEl);
    setupImages(newEl);
    state.syncedTextCache.set(nodeIdStr, newEl.textContent || '');
    return newEl;
  } catch (e) { console.error('renderNode error:', e); }
  return null;
}

// E-05: Batch render multiple nodes in a single pass to avoid race conditions
// where rendering one node invalidates another's DOM reference. Each node is
// re-queried from the DOM immediately before replacement, ensuring we always
// operate on the current element.
export function renderNodesById(nodeIds) {
  const results = new Map();
  for (const id of nodeIds) {
    results.set(id, renderNodeById(id));
  }
  return results;
}

export function fixEmptyBlocks() {
  $('docPage').querySelectorAll('p:empty, h1:empty, h2:empty, h3:empty, h4:empty, h5:empty, h6:empty')
    .forEach(el => { el.innerHTML = '<br>'; });
}

export function cacheAllText() {
  state.syncedTextCache.clear();
  $('docPage').querySelectorAll('[data-node-id]').forEach(el => {
    // Skip virtual-scroll placeholders
    if (el.classList.contains('vs-placeholder')) return;
    const tag = el.tagName.toLowerCase();
    if (tag === 'p' || /^h[1-6]$/.test(tag)) {
      state.syncedTextCache.set(el.dataset.nodeId, el.textContent || '');
    }
  });
}

export function syncParagraphText(el) {
  const { doc, syncedTextCache } = state;
  if (!doc || state.ignoreInput || !el) return;
  const nodeId = el.dataset?.nodeId;
  if (!nodeId) return;
  const newText = el.textContent || '';
  if (syncedTextCache.get(nodeId) === newText) return;
  try {
    doc.set_paragraph_text(nodeId, newText);
    syncedTextCache.set(nodeId, newText);
    markDirty();
    // E-09: Immediately clear stale find highlights when text changes
    clearFindHighlights();
    // Broadcast to collaboration peers
    broadcastTextSync(nodeId, newText);

    // E3.1: Track continuous typing for batch undo
    const batch = state._typingBatch;
    if (batch && batch.nodeId === nodeId) {
      batch.count++;
      clearTimeout(batch.timer);
      batch.timer = setTimeout(() => { state._typingBatch = null; }, 500);
    } else {
      // New typing session (different paragraph or first sync)
      if (batch) clearTimeout(batch.timer);
      state._typingBatch = {
        nodeId,
        count: 1,
        timer: setTimeout(() => { state._typingBatch = null; }, 500),
      };
    }
  } catch (e) { console.error('sync error:', e); }
}

export function syncAllText() {
  if (!state.doc) return;
  $('docPage').querySelectorAll('[data-node-id]').forEach(el => {
    // Skip virtual-scroll placeholders — their content is not rendered
    if (el.classList.contains('vs-placeholder')) return;
    const tag = el.tagName.toLowerCase();
    if (tag === 'p' || /^h[1-6]$/.test(tag)) syncParagraphText(el);
  });
}

// E-09: Clear stale find highlights immediately when text content changes
function clearFindHighlights() {
  const page = $('docPage');
  if (!page) return;
  const marks = page.querySelectorAll('mark.find-highlight');
  if (marks.length === 0) return;
  marks.forEach(m => {
    const parent = m.parentNode;
    while (m.firstChild) parent.insertBefore(m.firstChild, m);
    m.remove();
    parent.normalize();
  });
}

export function debouncedSync(el) {
  clearTimeout(state.syncTimer);
  state.syncTimer = setTimeout(() => {
    syncParagraphText(el);
    state.pagesRendered = false;
    updatePageBreaks();
    updateUndoRedo();
    updateStatusBar();
    // E1.5: Refresh find highlights after text edits
    state._onTextChanged?.();
  }, 200);
}

export function renderPages() {
  const { doc } = state;
  if (!doc) return;
  try {
    const html = doc.to_paginated_html();
    const container = $('pagesView');
    const pageCount = (html.match(/class="s1-page"/g) || []).length;
    container.innerHTML =
      '<div class="pages-nav">' +
        '<button id="prevPage">&#9664; Prev</button>' +
        '<span>' + pageCount + ' page' + (pageCount !== 1 ? 's' : '') + '</span>' +
        '<button id="nextPage">Next &#9654;</button>' +
      '</div>' + html;
    state.pagesRendered = true;
    const pages = container.querySelectorAll('.s1-page');
    let cur = 0;
    const prev = container.querySelector('#prevPage');
    const next = container.querySelector('#nextPage');
    if (prev) prev.onclick = () => { if (cur > 0) pages[--cur].scrollIntoView({ behavior: 'smooth', block: 'start' }); };
    if (next) next.onclick = () => { if (cur < pages.length - 1) pages[++cur].scrollIntoView({ behavior: 'smooth', block: 'start' }); };
  } catch (e) {
    $('pagesView').innerHTML = '<div style="padding:32px;color:#ff6b6b">Layout error: ' + e.message + '</div>';
  }
}

export function renderText() {
  const { doc } = state;
  if (!doc) return;
  try { $('textContent').textContent = doc.to_plain_text(); }
  catch (e) { $('textContent').textContent = 'Error: ' + e.message; }
}

// ═══════════════════════════════════════════════════
// VIRTUAL SCROLLING — for documents with 100+ blocks
// ═══════════════════════════════════════════════════

const VS_THRESHOLD = 100;  // Minimum block count to activate
const VS_BUFFER = 20;      // Extra blocks to render above/below viewport

function getBlockElements() {
  const page = $('docPage');
  if (!page) return [];
  // Collect direct children that are block-level content (skip page-break divs, etc.)
  return Array.from(page.children).filter(el => {
    const tag = el.tagName.toLowerCase();
    return tag === 'p' || /^h[1-6]$/.test(tag) || tag === 'table' ||
           tag === 'ul' || tag === 'ol' || tag === 'hr' ||
           el.dataset.nodeId || el.classList.contains('vs-placeholder');
  });
}

function maybeInitVirtualScroll() {
  const blocks = getBlockElements();
  if (blocks.length < VS_THRESHOLD) return;
  initVirtualScroll(blocks);
}

function initVirtualScroll(blocks) {
  const canvas = $('editorCanvas');
  if (!canvas) return;

  // Measure actual heights of all rendered blocks and store them
  const entries = blocks.map(el => {
    const rect = el.getBoundingClientRect();
    return {
      el,
      nodeId: el.dataset?.nodeId || null,
      height: Math.max(rect.height, 20),
      html: el.outerHTML,
      visible: true,
    };
  });

  // Create the IntersectionObserver with a generous root margin (buffer zone)
  const bufferPx = VS_BUFFER * 30; // ~30px per block estimate for buffer
  const observer = new IntersectionObserver((ioEntries) => {
    if (!state.virtualScroll) return;
    const vs = state.virtualScroll;
    let needsUpdate = false;

    for (const ioe of ioEntries) {
      const el = ioe.target;
      const idx = vs.indexMap.get(el);
      if (idx === undefined) continue;
      const entry = vs.entries[idx];
      if (!entry) continue;

      if (ioe.isIntersecting && !entry.visible) {
        // Coming into view — restore real content
        restoreBlock(entry, idx);
        needsUpdate = true;
      } else if (!ioe.isIntersecting && entry.visible) {
        // Going out of view — replace with placeholder
        collapseBlock(entry, idx, vs);
        needsUpdate = true;
      }
    }
  }, {
    root: canvas,
    rootMargin: `${bufferPx}px 0px ${bufferPx}px 0px`,
    threshold: 0,
  });

  // Build index map for fast lookups
  const indexMap = new WeakMap();
  entries.forEach((entry, i) => {
    indexMap.set(entry.el, i);
    observer.observe(entry.el);
  });

  state.virtualScroll = { entries, observer, indexMap };
}

function collapseBlock(entry, idx, vs) {
  if (!entry.visible || !entry.el || !entry.el.parentNode) return;
  // Don't collapse blocks that have focus or selection inside them
  const sel = window.getSelection();
  if (sel && sel.rangeCount > 0) {
    const range = sel.getRangeAt(0);
    if (entry.el.contains(range.startContainer) || entry.el.contains(range.endContainer)) {
      return; // Don't collapse blocks with active selection
    }
  }

  // E-07: Sync text content to WASM model before hiding, so any user edits
  // are preserved even if the block is off-screen when a sync would occur.
  if (entry.nodeId) {
    const tag = entry.el.tagName?.toLowerCase() || '';
    if (tag === 'p' || /^h[1-6]$/.test(tag)) {
      syncParagraphText(entry.el);
    }
  }

  // Save current HTML and measured height
  entry.html = entry.el.outerHTML;
  const rect = entry.el.getBoundingClientRect();
  entry.height = Math.max(rect.height, 20);

  // Create placeholder div with same height
  const placeholder = document.createElement('div');
  placeholder.className = 'vs-placeholder';
  placeholder.style.height = entry.height + 'px';
  // Preserve data-node-id for find/replace compatibility
  if (entry.nodeId) placeholder.dataset.nodeId = entry.nodeId;
  placeholder.dataset.vsIndex = String(idx);

  entry.el.replaceWith(placeholder);
  // Unobserve old element, observe placeholder
  vs.observer.unobserve(entry.el);
  entry.el = placeholder;
  vs.indexMap.set(placeholder, idx);
  vs.observer.observe(placeholder);
  entry.visible = false;
}

function restoreBlock(entry, idx) {
  if (entry.visible || !entry.el || !entry.el.parentNode) return;

  const vs = state.virtualScroll;
  let newEl = null;

  // E-07: Always re-render from WASM model when revealing a placeholder, so that
  // any edits made while the block was off-screen (e.g., via find/replace, undo,
  // or collaboration) are reflected. Fall back to cached HTML only if WASM render
  // is unavailable.
  if (entry.nodeId && state.doc) {
    try {
      const html = state.doc.render_node_html(entry.nodeId);
      const temp = document.createElement('div');
      temp.innerHTML = html;
      newEl = temp.firstElementChild;
    } catch (_) {
      // WASM render failed — fall back to cached HTML below
    }
  }

  if (!newEl) {
    // Fall back to cached HTML
    if (!entry.html) return;
    const temp = document.createElement('div');
    temp.innerHTML = entry.html;
    newEl = temp.firstElementChild;
    if (!newEl) return;
  }

  // Unobserve placeholder, replace with real element
  vs.observer.unobserve(entry.el);
  entry.el.replaceWith(newEl);
  entry.el = newEl;
  vs.indexMap.set(newEl, idx);
  vs.observer.observe(newEl);
  entry.visible = true;

  // Re-run fixup on restored element
  if (!newEl.innerHTML.trim()) newEl.innerHTML = '<br>';
  setupImages(newEl);
  // Update text cache from the freshly rendered content
  if (entry.nodeId) {
    state.syncedTextCache.set(entry.nodeId, newEl.textContent || '');
  }
}

function teardownVirtualScroll() {
  if (!state.virtualScroll) return;
  const vs = state.virtualScroll;
  vs.observer.disconnect();
  state.virtualScroll = null;
}
