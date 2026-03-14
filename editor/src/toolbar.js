// Toolbar state & formatting handlers
import { state, $ } from './state.js';
import { getSelectionInfo, saveSelection, setCursorAtOffset, setSelectionRange } from './selection.js';
import { renderNodeById, syncParagraphText } from './render.js';
import { updatePageBreaks } from './pagination.js';
import { broadcastOp } from './collab.js';

// Detect which style best matches the current paragraph formatting
function detectCurrentStyle(fmt) {
  const level = parseInt(fmt.headingLevel || '0') || 0;
  if (level === 1) return 'heading1';
  if (level === 2) return 'heading2';
  if (level === 3) return 'heading3';
  if (level === 4) return 'heading4';
  const fam = fmt.fontFamily || '';
  if (fam.toLowerCase().includes('courier') || fam.toLowerCase().includes('mono')) return 'code';
  const size = parseFloat(fmt.fontSize || '0');
  if (size >= 24) return 'title';
  if (size >= 14 && size <= 16 && fmt.color === '666666') return 'subtitle';
  if ((fmt.italic === true || fmt.italic === 'true') && fmt.color === '666666') return 'quote';
  return 'normal';
}

let _toolbarRAF = 0;
export function updateToolbarState() {
  // Debounce via requestAnimationFrame — selectionchange fires very frequently
  cancelAnimationFrame(_toolbarRAF);
  _toolbarRAF = requestAnimationFrame(_updateToolbarStateImpl);
}
function _updateToolbarStateImpl() {
  const { doc } = state;
  if (!doc || state.currentView !== 'editor') return;
  saveSelection();
  const info = state.lastSelInfo;
  if (!info) return;
  try {
    let fmt;
    if (info.collapsed) {
      fmt = JSON.parse(doc.get_formatting_json(info.startNodeId));
    } else {
      try {
        fmt = JSON.parse(doc.get_selection_formatting_json(
          info.startNodeId, info.startOffset, info.endNodeId, info.endOffset));
      } catch (_) { fmt = JSON.parse(doc.get_formatting_json(info.startNodeId)); }
    }
    const on = (k) => fmt[k] === true || fmt[k] === 'true';
    const setToggle = (id, active) => {
      const el = $(id);
      el.classList.toggle('active', active);
      el.setAttribute('aria-pressed', String(active));
    };
    setToggle('btnBold', on('bold'));
    setToggle('btnItalic', on('italic'));
    setToggle('btnUnderline', on('underline'));
    setToggle('btnStrike', on('strikethrough'));
    setToggle('btnSuperscript', on('superscript'));
    setToggle('btnSubscript', on('subscript'));
    if (fmt.fontSize && fmt.fontSize !== 'mixed') $('fontSize').value = Math.round(parseFloat(fmt.fontSize));
    if (fmt.fontFamily && fmt.fontFamily !== 'mixed') $('fontFamily').value = fmt.fontFamily;
    else if (!fmt.fontFamily) $('fontFamily').value = '';
    if (fmt.color && fmt.color !== 'mixed') $('colorSwatch').style.background = '#' + fmt.color;
    const paraFmt = info.collapsed ? fmt : JSON.parse(doc.get_formatting_json(info.startNodeId));
    // Update style gallery
    const styleName = detectCurrentStyle(paraFmt);
    const STYLE_LABELS = {
      normal: 'Normal', title: 'Title', subtitle: 'Subtitle',
      heading1: 'Heading 1', heading2: 'Heading 2', heading3: 'Heading 3', heading4: 'Heading 4',
      quote: 'Quote', code: 'Code',
    };
    $('styleGalleryLabel').textContent = STYLE_LABELS[styleName] || 'Normal';
    const panel = $('styleGalleryPanel');
    if (panel) {
      panel.querySelectorAll('.style-gallery-item').forEach(item => {
        const isActive = item.dataset.style === styleName;
        item.classList.toggle('active', isActive);
        item.setAttribute('aria-selected', String(isActive));
      });
    }
    setToggle('btnAlignL', !paraFmt.alignment || paraFmt.alignment === 'left');
    setToggle('btnAlignC', paraFmt.alignment === 'center');
    setToggle('btnAlignR', paraFmt.alignment === 'right');
    setToggle('btnAlignJ', paraFmt.alignment === 'justify');
  } catch (_) {}
}

export function updateUndoRedo() {
  if (!state.doc) return;
  try {
    $('btnUndo').disabled = !state.doc.can_undo();
    $('btnRedo').disabled = !state.doc.can_redo();
  } catch (_) {}
}

export function applyFormat(key, value) {
  const { doc } = state;
  if (!doc) return;
  const info = getSelectionInfo();
  if (!info) return;

  const page = $('docPage');
  const startEl = page.querySelector(`[data-node-id="${info.startNodeId}"]`) || info.startEl;
  const endEl = info.endNodeId !== info.startNodeId
    ? (page.querySelector(`[data-node-id="${info.endNodeId}"]`) || info.endEl)
    : startEl;

  syncParagraphText(startEl);
  if (endEl !== startEl) syncParagraphText(endEl);

  try {
    let sn, so, en, eo;
    if (info.collapsed) {
      const textLen = Array.from(startEl.textContent || '').length;
      if (textLen > 0) {
        doc.format_selection(info.startNodeId, 0, info.startNodeId, textLen, key, value);
        sn = info.startNodeId; so = 0; en = info.startNodeId; eo = textLen;
      }
    } else {
      doc.format_selection(info.startNodeId, info.startOffset, info.endNodeId, info.endOffset, key, value);
      sn = info.startNodeId; so = info.startOffset; en = info.endNodeId; eo = info.endOffset;
    }

    const newStartEl = renderNodeById(info.startNodeId);
    let newEndEl = null;
    if (info.endNodeId !== info.startNodeId) newEndEl = renderNodeById(info.endNodeId);

    page.focus();
    if (info.collapsed && newStartEl) setCursorAtOffset(newStartEl, info.startOffset);
    else if (newStartEl) setSelectionRange(newStartEl, info.startOffset, newEndEl || newStartEl, info.endOffset);

    if (newStartEl) state.lastSelInfo = { ...info, startEl: newStartEl, endEl: newEndEl || newStartEl };
    state.pagesRendered = false;
    updatePageBreaks();
    updateToolbarState();
    updateUndoRedo();
    if (sn) broadcastOp({ action: 'formatSelection', startNode: sn, startOffset: so, endNode: en, endOffset: eo, key, value });
  } catch (e) { console.error('format error:', e); }
}

export function toggleFormat(key) {
  const { doc } = state;
  if (!doc) return;
  const info = getSelectionInfo();
  if (!info) return;

  const page = $('docPage');
  const startEl = page.querySelector(`[data-node-id="${info.startNodeId}"]`) || info.startEl;
  const endEl = info.endNodeId !== info.startNodeId
    ? (page.querySelector(`[data-node-id="${info.endNodeId}"]`) || info.endEl)
    : startEl;

  let isActive = false;
  try {
    if (info.collapsed) {
      isActive = !!JSON.parse(doc.get_formatting_json(info.startNodeId))[key];
    } else {
      syncParagraphText(startEl);
      if (endEl !== startEl) syncParagraphText(endEl);
      try {
        isActive = JSON.parse(doc.get_selection_formatting_json(
          info.startNodeId, info.startOffset, info.endNodeId, info.endOffset))[key] === true;
      } catch (_) { isActive = !!JSON.parse(doc.get_formatting_json(info.startNodeId))[key]; }
    }
  } catch (_) {}

  const newVal = isActive ? 'false' : 'true';
  if (key === 'superscript' && newVal === 'true') applyFormat('subscript', 'false');
  if (key === 'subscript' && newVal === 'true') applyFormat('superscript', 'false');
  applyFormat(key, newVal);
}
