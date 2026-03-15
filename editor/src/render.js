// Document rendering — WASM → DOM
import { state, $ } from './state.js';
import { setupImages } from './images.js';
import { updatePageBreaks } from './pagination.js';
import { updateUndoRedo } from './toolbar.js';
import { markDirty, updateTrackChanges, updateStatusBar } from './file.js';
import { broadcastTextSync, broadcastOp } from './collab.js';

// ═══════════════════════════════════════════════════
// E8.4: Large-document warning threshold
// ═══════════════════════════════════════════════════
const LARGE_DOC_PARAGRAPH_THRESHOLD = 500;

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
    // E8.2: Clear the nodeIdToElement map on full re-render (DOM is rebuilt)
    state.nodeIdToElement.clear();
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
    // E8.2: Populate nodeIdToElement map from freshly rendered DOM
    populateNodeIdMap();
    setupTrackChangeHandlers();
    state.ignoreInput = false;
    state.pagesRendered = false;
    // Apply page dimensions from WASM before pagination
    applyPageDimensions();
    updatePageBreaks();
    updateUndoRedo();
    updateTrackChanges();
    updateStatusBar();
    // E8.4: Show document size warning if paragraph count exceeds threshold
    checkLargeDocumentWarning();
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

/**
 * Read page dimensions from WASM sections and apply to .doc-page.
 * Sets width, min-height, and padding (margins) from actual document properties.
 * Falls back to CSS defaults (US Letter, 1" margins) if WASM data unavailable.
 */
export function applyPageDimensions() {
  const page = $('docPage');
  if (!page || !state.doc) return;

  const ptToPx = 96 / 72; // 1pt = 1.333px

  try {
    const json = state.doc.get_sections_json();
    const sections = JSON.parse(json);
    if (sections.length > 0) {
      // Use first section for the page container dimensions
      const sec = sections[0];
      const widthPt = sec.pageWidth || 612;
      const heightPt = sec.pageHeight || 792;
      const marginTopPt = sec.marginTop || 72;
      const marginBottomPt = sec.marginBottom || 72;
      const marginLeftPt = sec.marginLeft || 72;
      const marginRightPt = sec.marginRight || 72;

      const widthPx = Math.round(widthPt * ptToPx);
      const heightPx = Math.round(heightPt * ptToPx);
      const marginTopPx = Math.round(marginTopPt * ptToPx);
      const marginBottomPx = Math.round(marginBottomPt * ptToPx);
      const marginLeftPx = Math.round(marginLeftPt * ptToPx);
      const marginRightPx = Math.round(marginRightPt * ptToPx);

      page.style.width = widthPx + 'px';
      page.style.minHeight = heightPx + 'px';
      page.style.paddingTop = marginTopPx + 'px';
      page.style.paddingBottom = marginBottomPx + 'px';
      page.style.paddingLeft = marginLeftPx + 'px';
      page.style.paddingRight = marginRightPx + 'px';

      // Store for pagination and ruler
      state.pageDims = {
        widthPt, heightPt,
        marginTopPt, marginBottomPt,
        marginLeftPt, marginRightPt,
      };
    }
  } catch (_) {
    // CSS defaults apply (US Letter, 1" margins)
  }
}

// ═══════════════════════════════════════════════════
// E8.2: nodeIdToElement map — O(1) DOM lookups
// ═══════════════════════════════════════════════════

/**
 * Populate the nodeIdToElement map by scanning all [data-node-id] elements
 * currently in the docPage. Called after full re-render.
 */
function populateNodeIdMap() {
  const map = state.nodeIdToElement;
  map.clear();
  const page = $('docPage');
  if (!page) return;
  page.querySelectorAll('[data-node-id]').forEach(el => {
    map.set(el.dataset.nodeId, el);
  });
}

/**
 * E8.2: Fast lookup of a DOM element by node ID.
 * Uses the cached map first, falls back to querySelector if not found
 * (the map entry may be stale or the element may have been added after
 * the last populateNodeIdMap call). Validates the cached element is still
 * attached to the DOM before returning it.
 */
export function lookupNodeElement(nodeIdStr) {
  const cached = state.nodeIdToElement.get(nodeIdStr);
  if (cached && cached.isConnected) return cached;
  // Fallback: querySelector and update cache
  const page = $('docPage');
  if (!page) return null;
  const el = page.querySelector(`[data-node-id="${nodeIdStr}"]`);
  if (el) state.nodeIdToElement.set(nodeIdStr, el);
  else state.nodeIdToElement.delete(nodeIdStr);
  return el;
}

// ═══════════════════════════════════════════════════
// E8.2: Incremental DOM patching in renderNodeById
// ═══════════════════════════════════════════════════

export function renderNodeById(nodeIdStr) {
  const { doc } = state;
  if (!doc) return null;
  try {
    const html = doc.render_node_html(nodeIdStr);
    // E8.2: Use lookupNodeElement for O(1) lookup, falling back to querySelector
    const el = lookupNodeElement(nodeIdStr);
    if (!el) return null;

    // E8.2: Incremental DOM patching — parse new HTML and compare before replacing
    const temp = document.createElement('div');
    temp.innerHTML = html;
    const newEl = temp.firstElementChild;
    if (!newEl) return null;
    if (!newEl.innerHTML.trim()) newEl.innerHTML = '<br>';

    // Compare outerHTML: if identical, skip the DOM replacement entirely
    if (el.outerHTML === newEl.outerHTML) {
      // Content is unchanged — no DOM mutation needed
      state.syncedTextCache.set(nodeIdStr, el.textContent || '');
      return el;
    }

    el.replaceWith(newEl);
    // E8.2: Update the nodeIdToElement map with the new element
    state.nodeIdToElement.set(nodeIdStr, newEl);
    // Also register any child node IDs (e.g., runs inside a paragraph)
    newEl.querySelectorAll('[data-node-id]').forEach(child => {
      state.nodeIdToElement.set(child.dataset.nodeId, child);
    });
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
// E8.4: Large-document warning in status bar
// ═══════════════════════════════════════════════════

function checkLargeDocumentWarning() {
  const page = $('docPage');
  if (!page) return;
  const paraCount = page.querySelectorAll(
    'p[data-node-id], h1[data-node-id], h2[data-node-id], h3[data-node-id], h4[data-node-id], h5[data-node-id], h6[data-node-id]'
  ).length;
  const info = $('statusInfo');
  if (paraCount > LARGE_DOC_PARAGRAPH_THRESHOLD && info) {
    // Append warning to status bar (will be overwritten by next updateStatusBar
    // call, so we set the _userMsg flag to keep it visible briefly)
    info._userMsg = true;
    info.textContent = `Large document: ${paraCount} paragraphs. Performance may be affected.`;
    // Clear after 5 seconds so normal status resumes
    setTimeout(() => { info._userMsg = false; updateStatusBar(); }, 5000);
  }
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

/**
 * E8.1: Check whether virtual scrolling should be suppressed.
 * Returns true if virtual scrolling must NOT run right now.
 * Guards:
 *   1. Find/replace bar is open — collapsing blocks breaks highlighting
 *   2. Selection spans across many paragraphs — collapsing would disrupt it
 */
function isVirtualScrollSuppressed() {
  // Guard 1: find/replace bar is open
  const findBar = $('findBar');
  if (findBar && findBar.classList.contains('show')) return true;

  // Guard 2: selection spans multiple paragraphs
  const sel = window.getSelection();
  if (sel && sel.rangeCount > 0 && !sel.isCollapsed) {
    const range = sel.getRangeAt(0);
    const page = $('docPage');
    if (page) {
      // Count how many block-level [data-node-id] elements the selection spans
      const startBlock = findAncestorBlock(range.startContainer, page);
      const endBlock = findAncestorBlock(range.endContainer, page);
      if (startBlock && endBlock && startBlock !== endBlock) {
        // Selection crosses at least two blocks — suppress virtual scroll
        return true;
      }
    }
  }
  return false;
}

/** Walk up to find the nearest block-level ancestor with data-node-id under page */
function findAncestorBlock(node, page) {
  let n = node;
  while (n && n !== page) {
    if (n.nodeType === 1 && n.dataset?.nodeId) {
      const tag = n.tagName.toLowerCase();
      if (tag === 'p' || /^h[1-6]$/.test(tag) || tag === 'table') return n;
    }
    n = n.parentNode;
  }
  return null;
}

function maybeInitVirtualScroll() {
  const blocks = getBlockElements();
  if (blocks.length < VS_THRESHOLD) return;
  if (isVirtualScrollSuppressed()) return;
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
    // E8.1: Re-check suppression on every observer callback
    if (isVirtualScrollSuppressed()) return;
    const vs = state.virtualScroll;

    for (const ioe of ioEntries) {
      const el = ioe.target;
      const idx = vs.indexMap.get(el);
      if (idx === undefined) continue;
      const entry = vs.entries[idx];
      if (!entry) continue;

      if (ioe.isIntersecting && !entry.visible) {
        // Coming into view — restore real content
        restoreBlock(entry, idx);
      } else if (!ioe.isIntersecting && entry.visible) {
        // Going out of view — replace with placeholder
        collapseBlock(entry, idx, vs);
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
  // E8.1: Don't collapse if virtual scroll is suppressed (find/replace, multi-para selection)
  if (isVirtualScrollSuppressed()) return;
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
  // E8.2: Update nodeIdToElement map — placeholder now holds the nodeId
  if (entry.nodeId) state.nodeIdToElement.set(entry.nodeId, placeholder);
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

  // E8.2: Update nodeIdToElement map with restored real element
  if (entry.nodeId) {
    state.nodeIdToElement.set(entry.nodeId, newEl);
    // Also register child node IDs
    newEl.querySelectorAll('[data-node-id]').forEach(child => {
      state.nodeIdToElement.set(child.dataset.nodeId, child);
    });
  }

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
