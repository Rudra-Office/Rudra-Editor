// PDF toolbar — page navigation, zoom, tool selection, save, signatures, keyboard shortcuts.
//
// Extracted from main.js to reduce entrypoint size and improve maintainability.

import { state, $ } from '../../state.js';
import { updatePdfStatusBar } from '../../file.js';

export function initPdfToolbar() {
  // Page navigation
  $('pdfPrevPage')?.addEventListener('click', () => {
    if (state.pdfViewer) { state.pdfViewer.prevPage(); updatePdfStatusBar(); }
  });
  $('pdfNextPage')?.addEventListener('click', () => {
    if (state.pdfViewer) { state.pdfViewer.nextPage(); updatePdfStatusBar(); }
  });

  // Zoom controls
  $('pdfZoomOut')?.addEventListener('click', () => {
    if (!state.pdfViewer) return;
    const sel = $('pdfZoomSelect');
    const scales = [0.5, 0.75, 1, 1.25, 1.5, 2];
    const current = state.pdfZoom;
    for (let i = scales.length - 1; i >= 0; i--) {
      if (scales[i] < current - 0.01) {
        state.pdfViewer.setZoom(scales[i]);
        sel.value = String(scales[i]);
        return;
      }
    }
  });
  $('pdfZoomIn')?.addEventListener('click', () => {
    if (!state.pdfViewer) return;
    const sel = $('pdfZoomSelect');
    const scales = [0.5, 0.75, 1, 1.25, 1.5, 2];
    const current = state.pdfZoom;
    for (const s of scales) {
      if (s > current + 0.01) {
        state.pdfViewer.setZoom(s);
        sel.value = String(s);
        return;
      }
    }
  });
  $('pdfZoomSelect')?.addEventListener('change', (e) => {
    if (!state.pdfViewer) return;
    const val = parseFloat(e.target.value);
    if (!isNaN(val) && val >= 0.25 && val <= 4.0) {
      state.pdfViewer.setZoom(val);
    } else {
      e.target.value = String(state.pdfZoom);
    }
  });

  // Tool selection
  const validPdfTools = ['select', 'highlight', 'comment', 'draw', 'text', 'redact'];
  document.querySelectorAll('.pdf-tool-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const toolName = btn.dataset.tool;
      if (!toolName) return;
      if (!validPdfTools.includes(toolName)) return;
      document.querySelectorAll('.pdf-tool-btn').forEach(b => {
        b.classList.remove('active');
        b.setAttribute('aria-pressed', 'false');
      });
      btn.classList.add('active');
      btn.setAttribute('aria-pressed', 'true');
      state.pdfTool = toolName;
      const container = $('pdfCanvasContainer');
      if (container) container.dataset.tool = btn.dataset.tool;
    });
  });

  // Download PDF button — bake annotations into PDF via WASM editor, then download
  $('pdfSave')?.addEventListener('click', async () => {
    if (!state.pdfBytes) return;
    try {
      let outputBytes = state.pdfBytes;

      if (state.pdfAnnotations.length > 0 || state.pdfTextEdits.length > 0) {
        const wasm = await import(/* @vite-ignore */ '../../../wasm-pkg/s1engine_wasm.js');
        const editor = wasm.WasmPdfEditor.open(state.pdfBytes);

        for (const ann of state.pdfAnnotations) {
          const page = ann.pageNum - 1;
          try {
            switch (ann.type) {
              case 'highlight':
                if (ann.props.quads?.length) {
                  const quads = [];
                  for (const q of ann.props.quads) {
                    quads.push(q.x, q.y, q.x + q.width, q.y, q.x, q.y + q.height, q.x + q.width, q.y + q.height);
                  }
                  editor.add_highlight_annotation(page, new Float64Array(quads), 1.0, 0.92, 0.23, ann.author || 'User', ann.props.selectedText || '');
                }
                break;
              case 'comment':
                editor.add_text_annotation(page, ann.props.x, ann.props.y, ann.author || 'User', ann.props.content || '');
                break;
              case 'ink':
                if (ann.props.paths?.[0]?.length) {
                  const pts = [];
                  for (const p of ann.props.paths[0]) { pts.push(p.x, p.y); }
                  editor.add_ink_annotation(page, new Float64Array(pts), 0.85, 0.07, 0.14, ann.props.strokeWidth || 2);
                }
                break;
              case 'text':
                editor.add_freetext_annotation(page, ann.props.x, ann.props.y, ann.props.width || 100, ann.props.height || 20, ann.props.content || '', ann.props.fontSize || 12);
                break;
              case 'redact':
                for (const r of (ann.props.rects || [])) {
                  editor.add_redaction(page, r.x, r.y, r.width, r.height);
                }
                break;
              case 'stamp':
                editor.add_freetext_annotation(page, ann.props.x, ann.props.y, ann.props.width || 150, ann.props.height || 60, '[Signature]', 12);
                break;
            }
          } catch (err) { console.warn(`Failed to write ${ann.type} annotation:`, err); }
        }

        for (const edit of state.pdfTextEdits) {
          try {
            const page = edit.pageNum - 1;
            const p = edit.position;
            editor.add_white_rect(page, p.x, p.y, p.width, p.height);
            editor.add_text_overlay(page, p.x, p.y, p.width, p.height, edit.newText, edit.fontInfo?.size || 12);
          } catch (err) { console.warn('Failed to write text edit:', err); }
        }

        outputBytes = editor.save();
        editor.free();
      }

      const blob = new Blob([outputBytes], { type: 'application/pdf' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = ($('docName').value || 'document') + '.pdf';
      document.body.appendChild(a);
      a.click();
      setTimeout(() => { try { document.body.removeChild(a); } catch(_) {} URL.revokeObjectURL(url); }, 60000);
      state.pdfModified = false;

      const { showToast } = await import('../../toolbar-handlers.js');
      showToast('PDF saved with annotations');
    } catch (err) {
      console.error('PDF save error:', err);
      const blob = new Blob([state.pdfBytes], { type: 'application/pdf' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = ($('docName').value || 'document') + '.pdf';
      document.body.appendChild(a);
      a.click();
      setTimeout(() => { try { document.body.removeChild(a); } catch(_) {} URL.revokeObjectURL(url); }, 60000);

      const { showToast } = await import('../../toolbar-handlers.js');
      showToast('Saved without annotations (WASM editor unavailable)', 'error');
    }
  });

  // Signature button
  $('pdfToolSignature')?.addEventListener('click', async () => {
    try {
      const { openSignatureModal } = await import('../../pdf-signatures.js');
      openSignatureModal();
    } catch (err) { console.error('Signature module error:', err); }
  });

  // Annotations panel close
  $('pdfAnnotClose')?.addEventListener('click', () => {
    $('pdfAnnotationsPanel')?.classList.remove('show');
  });

  // Keyboard shortcuts for PDF tools (only active in PDF view)
  document.addEventListener('keydown', (e) => {
    if (state.currentView !== 'pdf') return;
    const tag = e.target.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || e.target.isContentEditable) return;

    const toolMap = { v: 'select', h: 'highlight', c: 'comment', d: 'draw', t: 'text', r: 'redact' };
    const key = e.key.toLowerCase();

    if (toolMap[key] && !e.ctrlKey && !e.metaKey && !e.altKey) {
      const btn = document.querySelector(`.pdf-tool-btn[data-tool="${toolMap[key]}"]`);
      if (btn) btn.click();
      return;
    }

    if ((e.ctrlKey || e.metaKey) && key === 's') {
      e.preventDefault();
      $('pdfSave')?.click();
    }
  });
}
