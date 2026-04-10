// Form Controls Interactivity — M15.1
//
// Handles checkbox toggle, dropdown selection, and text input
// for content control form fields rendered from DOCX SDTs.

import { state, $ } from './state.js';
import { renderDocument } from './render.js';
import { markDirty } from './file.js';
import { broadcastOp } from './collab.js';

let _initialized = false;

export function initFormControls() {
  if (_initialized) return;
  const container = $('pageContainer');
  if (!container) return;
  _initialized = true;

  // Checkbox toggle via event delegation
  container.addEventListener('change', e => {
    const checkbox = e.target.closest('input[type="checkbox"][data-node-id]');
    if (checkbox && state.doc) {
      const nodeId = checkbox.dataset.nodeId;
      try {
        if (typeof state.doc.toggle_form_checkbox === 'function') {
          const newValue = state.doc.toggle_form_checkbox(nodeId);
          checkbox.checked = newValue;
          broadcastOp({ action: 'toggleCheckbox', nodeId });
          markDirty();
        }
      } catch (err) {
        console.error('toggle checkbox:', err);
      }
    }
  });

  // Dropdown change via event delegation
  container.addEventListener('change', e => {
    const select = e.target.closest('select.form-dropdown[data-node-id]');
    if (select && state.doc) {
      const nodeId = select.dataset.nodeId;
      const value = select.value;
      try {
        // Store selected value as text content of the paragraph
        if (typeof state.doc.set_paragraph_text === 'function') {
          state.doc.set_paragraph_text(nodeId, value);
          broadcastOp({ action: 'setFormValue', nodeId, value });
          markDirty();
        }
      } catch (err) {
        console.error('set dropdown value:', err);
      }
    }
  });

  // Text input change via event delegation
  container.addEventListener('input', e => {
    const input = e.target.closest('input[type="text"].form-text[data-node-id]');
    if (input && state.doc) {
      // Debounce text input sync
      clearTimeout(input._syncTimer);
      input._syncTimer = setTimeout(() => {
        const nodeId = input.dataset.nodeId;
        try {
          if (typeof state.doc.set_paragraph_text === 'function') {
            state.doc.set_paragraph_text(nodeId, input.value);
            broadcastOp({ action: 'setFormValue', nodeId, value: input.value });
            markDirty();
          }
        } catch (err) {
          console.error('set form text:', err);
        }
      }, 300);
    }
  });
}
