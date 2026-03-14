// Page break visualization using WASM layout engine.
// Shows page breaks, headers, and footers like a real document editor.
import { state, $ } from './state.js';
import { updateStatusBar as _updateStatus } from './file.js';

export function updatePageBreaks() {
  const page = $('docPage');
  if (!page) return;

  // Remove previous page-break indicators, editor headers/footers
  page.querySelectorAll('.page-break, .editor-header, .editor-footer').forEach(el => el.remove());

  const { doc } = state;
  if (!doc) return;

  // Header/footer HTML from the document (extracted by renderDocument)
  const headerHtml = state.docHeaderHtml || '';
  const footerHtml = state.docFooterHtml || '';

  // E-10: Sync all text to WASM before querying page map so it reflects latest edits.
  // Inline sync to avoid circular import with render.js.
  $('docPage').querySelectorAll('[data-node-id]').forEach(el => {
    if (el.classList.contains('vs-placeholder')) return;
    const tag = el.tagName.toLowerCase();
    if ((tag === 'p' || /^h[1-6]$/.test(tag)) && el.dataset.nodeId) {
      const nodeId = el.dataset.nodeId;
      const newText = el.textContent || '';
      if (state.syncedTextCache.get(nodeId) !== newText) {
        try {
          doc.set_paragraph_text(nodeId, newText);
          state.syncedTextCache.set(nodeId, newText);
        } catch (_) {}
      }
    }
  });

  // Get page map from WASM layout engine
  let pageMap = null;
  try { pageMap = JSON.parse(doc.get_page_map_json()); } catch (_) {}

  const numPages = pageMap?.pages?.length || 1;

  // ── Page 1 header (always shown at top of document) ──
  const topHdr = document.createElement('div');
  topHdr.className = 'editor-header';
  topHdr.contentEditable = 'false';
  if (headerHtml) {
    topHdr.innerHTML = headerHtml;
    substitutePageNumbers(topHdr, 1, numPages);
  }
  // Always prepend header area (even if empty, for page margin visualization)
  page.prepend(topHdr);

  // Remove previous page-bottom spacers
  page.querySelectorAll('.page-bottom-spacer').forEach(el => el.remove());

  if (pageMap && pageMap.pages && numPages > 1) {
    const pages = pageMap.pages;
    // CSS page padding (top + bottom) — must match .doc-page padding
    const pagePaddingPx = 96 * 2;
    // DPI ratio: layout uses 72pt/inch, screen is 96px/inch
    const ptToPx = 96 / 72;

    for (let i = 0; i < numPages - 1; i++) {
      const currentPage = pages[i];
      const nextPage = pages[i + 1];
      if (!nextPage.nodeIds?.length) continue;

      // Find first DOM element of next page
      const firstNextEl = page.querySelector(`[data-node-id="${nextPage.nodeIds[0]}"]`);
      if (!firstNextEl) continue;

      // E1.7: Calculate remaining space to fill the page height.
      // Measure content height between the previous page break (or top) and this break point.
      const pageHeightPx = (currentPage.height || 792) * ptToPx;
      const contentHeightPx = pageHeightPx - pagePaddingPx;

      // Measure actual content height for this page section
      let sectionContentHeight = 0;
      const currentNodeIds = currentPage.nodeIds || [];
      for (const nid of currentNodeIds) {
        const el = page.querySelector(`[data-node-id="${nid}"]`);
        if (el) sectionContentHeight += el.getBoundingClientRect().height;
      }

      // Add spacer to fill remaining page height
      const remaining = contentHeightPx - sectionContentHeight;
      if (remaining > 20) {
        const spacer = document.createElement('div');
        spacer.className = 'page-bottom-spacer';
        spacer.style.height = remaining + 'px';
        spacer.contentEditable = 'false';
        firstNextEl.before(spacer);
      }

      // Build page break with footer of current page and header of next page
      const brk = document.createElement('div');
      brk.className = 'page-break';
      brk.contentEditable = 'false';

      // Footer for page i
      const ftrDiv = document.createElement('div');
      ftrDiv.className = 'pb-footer';
      if (footerHtml) {
        ftrDiv.innerHTML = footerHtml;
        substitutePageNumbers(ftrDiv, i + 1, numPages);
      } else {
        ftrDiv.textContent = `Page ${i + 1}`;
      }
      brk.appendChild(ftrDiv);

      // Gap between pages
      const gap = document.createElement('div');
      gap.className = 'pb-gap';
      brk.appendChild(gap);

      // Header for page i+1
      const hdrDiv = document.createElement('div');
      hdrDiv.className = 'pb-header';
      if (headerHtml) {
        hdrDiv.innerHTML = headerHtml;
        substitutePageNumbers(hdrDiv, i + 2, numPages);
      }
      brk.appendChild(hdrDiv);

      // Page badge
      const badge = document.createElement('span');
      badge.className = 'pb-badge';
      badge.textContent = `Page ${i + 2}`;
      brk.appendChild(badge);

      firstNextEl.before(brk);
    }
  }

  // ── Footer (always shown at bottom of document) ──
  const btmFtr = document.createElement('div');
  btmFtr.className = 'editor-footer';
  btmFtr.contentEditable = 'false';
  if (footerHtml) {
    btmFtr.innerHTML = footerHtml;
    substitutePageNumbers(btmFtr, numPages, numPages);
  } else {
    // Default page number footer
    btmFtr.innerHTML = `<span style="display:block;text-align:center;color:#5f6368;font-size:9pt">${numPages}</span>`;
  }
  page.appendChild(btmFtr);

  _updateStatus();
}

/**
 * Replace page number / page count field placeholders in header/footer HTML.
 */
function substitutePageNumbers(container, pageNum, totalPages) {
  // Handle <span data-field="PageNumber"> elements from WASM
  container.querySelectorAll('[data-field]').forEach(el => {
    const field = el.dataset.field;
    if (field === 'PageNumber' || field === 'PAGE') {
      el.textContent = String(pageNum);
    } else if (field === 'PageCount' || field === 'NUMPAGES') {
      el.textContent = String(totalPages);
    }
  });
  // Handle plain text patterns
  const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT, null);
  let node;
  while ((node = walker.nextNode())) {
    const t = node.textContent;
    if (t.includes('PAGE') || t.includes('NUMPAGES')) {
      node.textContent = t
        .replace(/\bNUMPAGES\b/g, String(totalPages))
        .replace(/\bPAGE\b/g, String(pageNum));
    }
  }
}

function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
