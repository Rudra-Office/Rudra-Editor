// Spreadsheet toolbar — all spreadsheet-specific toolbar, menu bar, and formatting handlers.
//
// Extracted from main.js to reduce entrypoint size and enable independent evolution
// of the spreadsheet UI without touching the core editor boot path.

import { state, $ } from '../../state.js';

/**
 * Custom modal prompt for spreadsheet comments (replaces window.prompt).
 */
function _ssCommentPrompt(message, defaultValue) {
  return new Promise(resolve => {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay show';
    const modal = document.createElement('div');
    modal.className = 'modal';
    const h3 = document.createElement('h3');
    h3.textContent = message;
    modal.appendChild(h3);
    const input = document.createElement('textarea');
    input.value = defaultValue || '';
    input.rows = 3;
    input.style.cssText = 'width:100%;padding:8px;margin:8px 0 16px;border:1px solid #dadce0;border-radius:4px;font-size:14px;box-sizing:border-box;resize:vertical;';
    modal.appendChild(input);
    const actions = document.createElement('div');
    actions.className = 'modal-actions';
    const cancelBtn = document.createElement('button');
    cancelBtn.textContent = 'Cancel';
    cancelBtn.className = 'modal-cancel';
    const okBtn = document.createElement('button');
    okBtn.textContent = 'OK';
    okBtn.className = 'modal-ok primary';
    actions.appendChild(cancelBtn);
    actions.appendChild(okBtn);
    modal.appendChild(actions);
    overlay.appendChild(modal);
    document.body.appendChild(overlay);
    const close = (val) => { document.body.removeChild(overlay); resolve(val); };
    cancelBtn.onclick = () => close(null);
    okBtn.onclick = () => close(input.value);
    overlay.onclick = (e) => { if (e.target === overlay) close(null); };
    input.addEventListener('keydown', (e) => { if (e.key === 'Escape') close(null); });
    input.focus();
  });
}

function updateSSToolbarState() {
  if (!state.spreadsheetView) return;
  const style = state.spreadsheetView.getActiveStyle();

  const toggleBtn = (id, prop) => {
    const btn = $(id);
    if (btn) {
      if (style[prop]) btn.classList.add('active');
      else btn.classList.remove('active');
    }
  };
  toggleBtn('ssBold', 'bold');
  toggleBtn('ssItalic', 'italic');
  toggleBtn('ssUnderline', 'underline');
  toggleBtn('ssStrikethrough', 'strikethrough');

  ['ssAlignLeft', 'ssAlignCenter', 'ssAlignRight'].forEach(id => {
    const btn = $(id);
    if (btn) btn.classList.remove('active');
  });
  if (style.align === 'left') $('ssAlignLeft')?.classList.add('active');
  else if (style.align === 'center') $('ssAlignCenter')?.classList.add('active');
  else if (style.align === 'right') $('ssAlignRight')?.classList.add('active');

  const nfSelect = $('ssNumberFormat');
  if (nfSelect) nfSelect.value = style.numberFormat || 'general';

  const ffSelect = $('ssFontFamily');
  if (ffSelect && style.fontFamily) ffSelect.value = style.fontFamily;
  else if (ffSelect) ffSelect.value = 'Arial, sans-serif';

  const fsSelect = $('ssFontSize');
  if (fsSelect) fsSelect.value = String(style.fontSize || 13);

  const fontBar = $('ssFontColorBar');
  if (fontBar) fontBar.style.background = style.color || '#000000';
  const fillBar = $('ssFillColorBar');
  if (fillBar) fillBar.style.background = style.fill || '#ffffff';
}

export function initSpreadsheetToolbar() {
  // ── Toolbar buttons ──
  $('ssUndo')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.undo(); });
  $('ssRedo')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.redo(); });
  $('ssCut')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.cutCells(); });
  $('ssCopy')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.copyCells(); });
  $('ssPaste')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.pasteCells(); });
  $('ssSortAsc')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.sort(state.spreadsheetView.selectedCell.col, true); });
  $('ssSortDesc')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.sort(state.spreadsheetView.selectedCell.col, false); });
  $('ssFilter')?.addEventListener('click', () => {
    if (!state.spreadsheetView) return;
    const col = state.spreadsheetView.selectedCell.col;
    if (state.spreadsheetView.filterState[col]) state.spreadsheetView.removeFilter(col);
    else state.spreadsheetView.addFilter(col);
  });
  $('ssFreeze')?.addEventListener('click', () => {
    if (!state.spreadsheetView) return;
    const { col, row } = state.spreadsheetView.selectedCell;
    if (state.spreadsheetView.frozenCols === col && state.spreadsheetView.frozenRows === row) {
      state.spreadsheetView.freezePanes(0, 0);
    } else {
      state.spreadsheetView.freezePanes(col, row);
    }
  });
  $('ssExportCSV')?.addEventListener('click', () => {
    if (!state.spreadsheetView) return;
    const filename = $('docName')?.value || 'spreadsheet';
    state.spreadsheetView.downloadCSV(filename);
  });

  // ── Formatting buttons ──
  $('ssBold')?.addEventListener('click', () => { if (!state.spreadsheetView) return; state.spreadsheetView.toggleFormat('bold'); updateSSToolbarState(); });
  $('ssItalic')?.addEventListener('click', () => { if (!state.spreadsheetView) return; state.spreadsheetView.toggleFormat('italic'); updateSSToolbarState(); });
  $('ssUnderline')?.addEventListener('click', () => { if (!state.spreadsheetView) return; state.spreadsheetView.toggleFormat('underline'); updateSSToolbarState(); });
  $('ssStrikethrough')?.addEventListener('click', () => { if (!state.spreadsheetView) return; state.spreadsheetView.toggleFormat('strikethrough'); updateSSToolbarState(); });

  $('ssFontColor')?.addEventListener('input', (e) => { if (!state.spreadsheetView) return; state.spreadsheetView.setFormat('color', e.target.value); const bar = $('ssFontColorBar'); if (bar) bar.style.background = e.target.value; });
  $('ssFillColor')?.addEventListener('input', (e) => { if (!state.spreadsheetView) return; state.spreadsheetView.setFormat('fill', e.target.value); const bar = $('ssFillColorBar'); if (bar) bar.style.background = e.target.value; });

  $('ssAlignLeft')?.addEventListener('click', () => { if (!state.spreadsheetView) return; state.spreadsheetView.setFormat('align', 'left'); updateSSToolbarState(); });
  $('ssAlignCenter')?.addEventListener('click', () => { if (!state.spreadsheetView) return; state.spreadsheetView.setFormat('align', 'center'); updateSSToolbarState(); });
  $('ssAlignRight')?.addEventListener('click', () => { if (!state.spreadsheetView) return; state.spreadsheetView.setFormat('align', 'right'); updateSSToolbarState(); });

  $('ssFontFamily')?.addEventListener('change', (e) => { if (!state.spreadsheetView) return; state.spreadsheetView.setFormat('fontFamily', e.target.value); });
  $('ssFontSize')?.addEventListener('change', (e) => { if (!state.spreadsheetView) return; const size = parseInt(e.target.value, 10); if (!isNaN(size) && size > 0) state.spreadsheetView.setFormat('fontSize', size); });
  $('ssNumberFormat')?.addEventListener('change', (e) => { if (!state.spreadsheetView) return; const fmt = e.target.value; state.spreadsheetView.setFormat('numberFormat', fmt === 'general' ? null : fmt); });
  $('ssFindReplace')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.openFindBar(false); });
  $('ssMenuFind')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.openFindBar(true); });
  $('ssMergeCells')?.addEventListener('click', () => { if (state.spreadsheetView) state.spreadsheetView.mergeCells(); });
  $('ssTextWrap')?.addEventListener('click', () => { if (!state.spreadsheetView) return; state.spreadsheetView.toggleFormat('wrap'); updateSSToolbarState(); });

  // ── Menu bar ──
  function closeSsMenus() {
    document.querySelectorAll('#ssMenubar .app-menu-item').forEach(m => {
      m.classList.remove('open');
      const btn = m.querySelector('.app-menu-btn');
      if (btn) btn.setAttribute('aria-expanded', 'false');
    });
  }

  let ssMenubarActive = false;
  Array.from(document.querySelectorAll('#ssMenubar .app-menu-item')).forEach((item) => {
    const btn = item.querySelector('.app-menu-btn');
    if (!btn) return;
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const wasOpen = item.classList.contains('open');
      closeSsMenus();
      ssMenubarActive = false;
      if (!wasOpen) { item.classList.add('open'); btn.setAttribute('aria-expanded', 'true'); ssMenubarActive = true; }
    });
    btn.addEventListener('mouseenter', () => {
      if (ssMenubarActive) { closeSsMenus(); item.classList.add('open'); btn.setAttribute('aria-expanded', 'true'); }
    });
  });
  document.addEventListener('click', (e) => { if (!e.target.closest('#ssMenubar')) { closeSsMenus(); ssMenubarActive = false; } });

  // File menu
  $('ssMenuNewSheet')?.addEventListener('click', async () => {
    closeSsMenus();
    try {
      const { switchView } = await import('../../file.js');
      const { SpreadsheetView } = await import('../../spreadsheet.js');
      const { addFileTab } = await import('../../tabs.js');
      if (state.spreadsheetView) state.spreadsheetView.destroy();
      state.currentFormat = 'CSV';
      state.spreadsheetView = new SpreadsheetView($('spreadsheetContainer'));
      state.spreadsheetView.loadWorkbook('', 'Sheet1.csv');
      $('docName').value = 'Untitled Spreadsheet';
      const info = $('statusInfo'); if (info) info.textContent = '0 cells';
      $('statusFormat').textContent = 'CSV';
      addFileTab('Untitled Spreadsheet', 'spreadsheet', null);
    } catch (e) { console.error('New sheet error:', e); }
  });
  $('ssMenuOpen')?.addEventListener('click', () => { closeSsMenus(); const input = $('csvInput') || $('fileInput'); if (input) input.click(); });
  $('ssMenuSaveXLSX')?.addEventListener('click', () => {
    closeSsMenus();
    if (!state.spreadsheetView) return;
    try {
      const bytes = state.spreadsheetView.exportXLSX();
      const blob = new Blob([bytes], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a'); a.href = url; a.download = ($('docName')?.value || 'spreadsheet') + '.xlsx'; a.click();
      setTimeout(() => { URL.revokeObjectURL(url); }, 60000);
    } catch (e) { console.error('XLSX export error:', e); }
  });
  $('ssMenuSaveCSV')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.downloadCSV($('docName')?.value || 'spreadsheet'); });
  $('ssMenuPrint')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.printToPDF(); });
  $('ssMenuClose')?.addEventListener('click', async () => {
    closeSsMenus();
    if (state.spreadsheetView) { state.spreadsheetView.destroy(); state.spreadsheetView = null; }
    const { deactivateEditor } = await import('../../file.js');
    const { closeFileTab } = await import('../../tabs.js');
    if (state.activeFileId) closeFileTab(state.activeFileId);
    else deactivateEditor();
  });

  // Edit menu
  $('ssMenuUndo')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.undo(); });
  $('ssMenuRedo')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.redo(); });
  $('ssMenuCut')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.cutCells(); });
  $('ssMenuCopy')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.copyCells(); });
  $('ssMenuPaste')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.pasteCells(); });
  $('ssMenuPasteSpecial')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.showPasteSpecialDialog(); });
  $('ssMenuSelectAll')?.addEventListener('click', () => {
    closeSsMenus();
    if (!state.spreadsheetView) return;
    const sheet = state.spreadsheetView._sheet();
    if (sheet) { state.spreadsheetView.selectionRange = { startCol: 0, startRow: 0, endCol: sheet.maxCol, endRow: sheet.maxRow }; state.spreadsheetView.render(); }
  });

  // View menu
  $('ssMenuFreezePanes')?.addEventListener('click', () => {
    closeSsMenus();
    if (!state.spreadsheetView) return;
    const { col, row } = state.spreadsheetView.selectedCell;
    if (state.spreadsheetView.frozenCols === col && state.spreadsheetView.frozenRows === row) state.spreadsheetView.freezePanes(0, 0);
    else state.spreadsheetView.freezePanes(col, row);
  });
  $('ssMenuGridlines')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) { state.spreadsheetView._showGridlines = state.spreadsheetView._showGridlines !== false ? false : true; state.spreadsheetView.render(); } });
  $('ssMenuFullScreen')?.addEventListener('click', () => { closeSsMenus(); if (document.fullscreenElement) document.exitFullscreen(); else document.documentElement.requestFullscreen().catch(() => {}); });

  // Insert menu
  $('ssMenuFunction')?.addEventListener('click', () => { closeSsMenus(); if (!state.spreadsheetView) return; const { col, row } = state.spreadsheetView.selectedCell; state.spreadsheetView.startEdit(col, row, '='); });
  $('ssMenuInsertImage')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.insertImage(); });
  $('ssMenuInsertShape')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.showInsertShapeDialog(); });
  $('ssMenuInsertChart')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) _openChartTypeModal(); });

  // Chart type modal
  function _openChartTypeModal() {
    const modal = $('chartTypeModal'); if (!modal) return;
    modal.style.display = ''; modal.classList.add('show');
    modal.querySelectorAll('.chart-type-grid button').forEach(b => b.classList.remove('selected'));
    const insertBtn = $('chartInsertBtn'); if (insertBtn) insertBtn.disabled = true;
    state._pendingChartType = null;
  }
  function _closeChartTypeModal() {
    const modal = $('chartTypeModal'); if (!modal) return;
    modal.style.display = 'none'; modal.classList.remove('show'); state._pendingChartType = null;
  }
  function _insertChartFromModal() {
    const type = state._pendingChartType; if (!type || !state.spreadsheetView) return;
    state.spreadsheetView.insertChart(type); _closeChartTypeModal();
  }

  const chartGrid = document.querySelector('#chartTypeModal .chart-type-grid');
  if (chartGrid) {
    chartGrid.addEventListener('click', (e) => {
      const btn = e.target.closest('button[data-chart-type]'); if (!btn) return;
      chartGrid.querySelectorAll('button').forEach(b => b.classList.remove('selected'));
      btn.classList.add('selected'); state._pendingChartType = btn.dataset.chartType;
      const insertBtn = $('chartInsertBtn'); if (insertBtn) insertBtn.disabled = false;
    });
    chartGrid.addEventListener('dblclick', (e) => {
      const btn = e.target.closest('button[data-chart-type]'); if (!btn) return;
      state._pendingChartType = btn.dataset.chartType; _insertChartFromModal();
    });
  }
  $('chartInsertBtn')?.addEventListener('click', () => _insertChartFromModal());
  $('chartCancelBtn')?.addEventListener('click', () => _closeChartTypeModal());
  $('chartTypeModal')?.addEventListener('click', (e) => { if (e.target === $('chartTypeModal')) _closeChartTypeModal(); });

  // Format menu
  $('ssMenuNumberFormat')?.addEventListener('click', () => { closeSsMenus(); const sel = $('ssNumberFormat'); if (sel) sel.focus(); });
  $('ssMenuCellStyle')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) { state.spreadsheetView.toggleFormat('bold'); updateSSToolbarState(); } });
  $('ssMenuMergeCellsMenu')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.mergeCells(); });
  $('ssMenuConditionalFormat')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.showConditionalFormatDialog(); });

  // Data menu
  $('ssMenuSortAZ')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.sort(state.spreadsheetView.selectedCell.col, true); });
  $('ssMenuSortZA')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.sort(state.spreadsheetView.selectedCell.col, false); });
  $('ssMenuSortDialog')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.showSortDialog(); });
  $('ssMenuFilter')?.addEventListener('click', () => {
    closeSsMenus();
    if (!state.spreadsheetView) return;
    const col = state.spreadsheetView.selectedCell.col;
    if (state.spreadsheetView.filterState[col]) state.spreadsheetView.removeFilter(col);
    else state.spreadsheetView.addFilter(col);
  });
  $('ssMenuRemoveDuplicates')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.showRemoveDuplicatesDialog(); });
  $('ssMenuDataValidation')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.showDataValidationDialog(); });

  // Comments
  $('ssMenuInsertComment')?.addEventListener('click', async () => {
    closeSsMenus();
    if (!state.spreadsheetView) return;
    const { col, row } = state.spreadsheetView.selectedCell;
    const sheet = state.spreadsheetView._sheet();
    const cell = sheet ? sheet.getCell(col, row) : null;
    const existing = cell?.comment?.text || '';
    const text = await _ssCommentPrompt(existing ? 'Edit comment:' : 'Add comment:', existing);
    if (text === null) return;
    const trimmed = text.trim();
    if (trimmed.length > 10000) {
      const { showToast } = await import('../../toolbar-handlers.js');
      showToast('Comment too long (max 10,000 characters)', 'error');
      return;
    }
    state.spreadsheetView.setCellComment(col, row, text);
  });
  $('ssMenuShowComments')?.addEventListener('click', () => { closeSsMenus(); if (state.spreadsheetView) state.spreadsheetView.showCommentsPanel(); });

  // Update toolbar state on interaction
  const ssContainer = $('spreadsheetContainer');
  if (ssContainer) {
    ssContainer.addEventListener('mouseup', () => setTimeout(updateSSToolbarState, 50));
    ssContainer.addEventListener('keyup', () => setTimeout(updateSSToolbarState, 50));
  }
}
