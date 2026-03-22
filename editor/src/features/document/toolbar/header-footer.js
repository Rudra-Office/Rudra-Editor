// Header/footer inline editing mode.
// Extracted from toolbar-handlers.js to break circular dependency.
import { state, $ } from '../../../state.js';
import { renderDocument } from '../../../render.js';

// Late-bound callback for opening the header/footer options modal.
// toolbar-handlers.js registers this after import to avoid circular dependency.
let _openHeaderFooterModalFn = null;

/**
 * Register the function that opens the header/footer options modal.
 * Called by toolbar-handlers.js during initialization.
 */
export function setOpenHeaderFooterModalFn(fn) {
  _openHeaderFooterModalFn = fn;
}

/**
 * Enter inline header/footer editing mode.
 * @param {'header'|'footer'} kind — which region to edit
 * @param {HTMLElement} pageEl — the .doc-page element
 */
export function enterHeaderFooterEditMode(kind, pageEl) {
  // Exit any existing edit mode first
  if (state.hfEditingMode) {
    exitHeaderFooterEditMode();
  }

  const selector = kind === 'header' ? '.page-header' : '.page-footer';
  const hfEl = pageEl.querySelector(selector);
  if (!hfEl) return;

  const pageNum = parseInt(pageEl.dataset.page, 10) || 1;
  state.hfEditingMode = kind;
  state.hfEditingPage = pageNum;

  // Make the header/footer editable
  hfEl.contentEditable = 'true';
  hfEl.classList.remove('hf-hoverable');
  hfEl.classList.add('hf-editing');
  hfEl.removeAttribute('title');

  // Add label badge
  const label = document.createElement('span');
  label.className = 'hf-editing-label';
  label.textContent = kind === 'header' ? 'Header' : 'Footer';
  hfEl.appendChild(label);

  // Add mini toolbar with options and close button
  const toolbar = document.createElement('span');
  toolbar.className = 'hf-toolbar';

  const pageNumBtn = document.createElement('button');
  pageNumBtn.textContent = 'Insert Page Number';
  pageNumBtn.title = 'Insert a page number field at cursor position';
  pageNumBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    _insertPageNumberField(hfEl);
  });
  toolbar.appendChild(pageNumBtn);

  const optionsBtn = document.createElement('button');
  optionsBtn.textContent = 'Options';
  optionsBtn.title = 'Open header and footer options';
  optionsBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    if (_openHeaderFooterModalFn) {
      _openHeaderFooterModalFn();
    }
  });
  toolbar.appendChild(optionsBtn);

  const closeBtn = document.createElement('button');
  closeBtn.textContent = 'Close';
  closeBtn.title = 'Exit header/footer editing (Escape)';
  closeBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    exitHeaderFooterEditMode();
  });
  toolbar.appendChild(closeBtn);

  hfEl.appendChild(toolbar);

  // Dim the main content area
  const contentEl = pageEl.querySelector('.page-content');
  if (contentEl) contentEl.classList.add('hf-dimmed');

  // Focus the header/footer
  hfEl.focus();

  // Place cursor at end of existing content (before our label/toolbar elements)
  try {
    const sel = window.getSelection();
    const range = document.createRange();
    // Find the last text-bearing child (skip our label/toolbar)
    const textNodes = [];
    for (const child of hfEl.childNodes) {
      if (child.nodeType === Node.TEXT_NODE ||
          (child.nodeType === Node.ELEMENT_NODE &&
           !child.classList.contains('hf-editing-label') &&
           !child.classList.contains('hf-toolbar'))) {
        textNodes.push(child);
      }
    }
    if (textNodes.length > 0) {
      const lastChild = textNodes[textNodes.length - 1];
      range.selectNodeContents(lastChild);
      range.collapse(false);
    } else {
      range.selectNodeContents(hfEl);
      range.collapse(false);
    }
    sel.removeAllRanges();
    sel.addRange(range);
  } catch (_) {}
}

/**
 * Exit header/footer editing mode and sync content back.
 */
export function exitHeaderFooterEditMode() {
  if (!state.hfEditingMode) return;

  const kind = state.hfEditingMode;
  const pageNum = state.hfEditingPage;
  state.hfEditingMode = null;
  state.hfEditingPage = null;

  // Find all editing header/footer elements and restore them
  const container = $('pageContainer');
  if (!container) return;

  container.querySelectorAll('.hf-editing').forEach(hfEl => {
    // Extract the user-entered content (excluding our UI elements)
    const label = hfEl.querySelector('.hf-editing-label');
    const toolbar = hfEl.querySelector('.hf-toolbar');
    if (label) label.remove();
    if (toolbar) toolbar.remove();

    // Get the text content the user entered, EXCLUDING field element text.
    // Field elements (page number, page count) have substituted text that
    // must not be synced back to the WASM model as plain text — doing so
    // would duplicate it alongside the Field nodes, causing garbled output
    // like "Page 1Page 1" or "12" instead of just "1".
    const userHtml = hfEl.innerHTML.trim();
    const cloneForText = hfEl.cloneNode(true);
    cloneForText.querySelectorAll('[data-field]').forEach(f => f.remove());
    const userText = cloneForText.textContent.trim();

    // Restore non-editable state
    hfEl.contentEditable = 'false';
    hfEl.classList.remove('hf-editing');
    hfEl.classList.add('hf-hoverable');
    hfEl.setAttribute('title',
      hfEl.dataset.hfKind === 'header' ? 'Double-click to edit header' : 'Double-click to edit footer');

    // Un-dim content
    const pageEl = hfEl.closest('.doc-page');
    if (pageEl) {
      const contentEl = pageEl.querySelector('.page-content');
      if (contentEl) contentEl.classList.remove('hf-dimmed');
    }

    // Sync the edited content back to state
    const isHeader = hfEl.dataset.hfKind === 'header';
    const isFirstPage = (pageNum === 1) && state.hasDifferentFirstPage;

    if (userText || userHtml) {
      // Preserve any data-field spans (page numbers) the user may have added
      const hasFields = hfEl.querySelector('[data-field]') !== null;
      let finalHtml;
      if (hasFields) {
        // Keep the HTML structure (it has field elements)
        finalHtml = userHtml;
      } else {
        // Wrap in styled span
        finalHtml = '<span style="display:block;text-align:center;color:var(--text-secondary,#5f6368);font-size:9pt">' +
          _escapeHtmlForHF(userText) + '</span>';
      }

      if (isHeader) {
        if (isFirstPage) {
          state.docFirstPageHeaderHtml = finalHtml;
        } else {
          state.docHeaderHtml = finalHtml;
        }
      } else {
        if (isFirstPage) {
          state.docFirstPageFooterHtml = finalHtml;
        } else {
          state.docFooterHtml = finalHtml;
        }
      }
    } else {
      // Empty content — clear
      if (isHeader) {
        if (isFirstPage) {
          state.docFirstPageHeaderHtml = '';
        } else {
          state.docHeaderHtml = '';
        }
      } else {
        if (isFirstPage) {
          state.docFirstPageFooterHtml = '';
        } else {
          state.docFooterHtml = '';
        }
      }
    }

    // Sync to WASM backend if available
    _syncHeaderFooterToWasm(hfEl.dataset.hfKind, isFirstPage ? 'first' : 'default', userText);
  });

  // Re-render pages to apply updated header/footer across all pages
  renderDocument();
}

/**
 * Sync header/footer text to the WASM model.
 */
function _syncHeaderFooterToWasm(kind, hfType, text) {
  const { doc } = state;
  if (!doc) return;
  try {
    if (typeof doc.set_header_footer_text === 'function') {
      doc.set_header_footer_text(0, kind, hfType, text);
    }
  } catch (e) {
    console.warn('Failed to sync header/footer to WASM:', e);
  }
}

/**
 * Insert a page number field element at the current cursor position.
 */
function _insertPageNumberField(hfEl) {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0) return;

  const range = sel.getRangeAt(0);
  // Verify the selection is within the header/footer element
  if (!hfEl.contains(range.startContainer)) {
    // Place cursor at end
    const newRange = document.createRange();
    newRange.selectNodeContents(hfEl);
    newRange.collapse(false);
    sel.removeAllRanges();
    sel.addRange(newRange);
  }

  const field = document.createElement('span');
  field.setAttribute('data-field', 'PageNumber');
  field.contentEditable = 'false';
  field.style.fontWeight = 'normal';

  // Show placeholder number
  const pageEl = hfEl.closest('.doc-page');
  const pageNum = pageEl ? (parseInt(pageEl.dataset.page, 10) || 1) : 1;
  field.textContent = String(pageNum);

  const updatedRange = sel.getRangeAt(0);
  updatedRange.deleteContents();
  updatedRange.insertNode(field);

  // Move cursor after the inserted field
  const afterRange = document.createRange();
  afterRange.setStartAfter(field);
  afterRange.collapse(true);
  sel.removeAllRanges();
  sel.addRange(afterRange);
}

/**
 * Escape HTML for header/footer content.
 */
function _escapeHtmlForHF(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}
