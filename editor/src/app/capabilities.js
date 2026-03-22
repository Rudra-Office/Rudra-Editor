// Capability registry — single source of truth for feature availability.
//
// Checked once at boot (after WASM loads), then queried by menus, toolbars,
// buttons, shortcuts, and help text. Replaces scattered `typeof doc.xxx === 'function'`
// checks across the codebase.
//
// Usage:
//   import { capabilities, initCapabilities } from './app/capabilities.js';
//   initCapabilities(wasmEngine, wasmModule, config);
//   if (capabilities.canInsertFootnote) { ... }

/** @type {Record<string, boolean>} */
export const capabilities = {
  // Document editing
  canInsertFootnote: false,
  canInsertEndnote: false,
  canInsertBookmark: false,
  canInsertComment: false,
  canTrackChanges: false,
  canInsertTableOfContents: false,
  canInsertEquation: false,
  canInsertDrawing: false,

  // PDF
  canOpenPdf: false,
  canEditPdf: false,
  canSignPdf: false,
  canFillPdfForms: false,
  canManagePdfPages: false,

  // Collaboration
  canCollaborate: false,
  canForceSyncRoom: false,

  // Spreadsheet
  canOpenSpreadsheet: false,

  // AI
  canUseAI: false,

  // Admin (server-side, detected from config)
  canAccessAdmin: false,
};

/**
 * Initialize capabilities by probing the WASM engine and config.
 *
 * @param {object} engine - WasmEngine instance (state.engine)
 * @param {object} wasmModule - WASM module (for checking class existence)
 * @param {object} config - window.S1_CONFIG
 */
export function initCapabilities(engine, wasmModule, config = {}) {
  // Probe document editing capabilities via a temporary doc
  if (engine) {
    try {
      const probe = engine.create();
      capabilities.canInsertFootnote = typeof probe.insert_footnote === 'function';
      capabilities.canInsertEndnote = typeof probe.insert_endnote === 'function';
      capabilities.canInsertBookmark = typeof probe.insert_bookmark === 'function';
      capabilities.canInsertComment = typeof probe.insert_comment === 'function';
      capabilities.canTrackChanges = typeof probe.accept_change === 'function';
      capabilities.canInsertTableOfContents = typeof probe.insert_table_of_contents === 'function';
      capabilities.canInsertEquation = typeof probe.insert_equation === 'function';
      capabilities.canInsertDrawing = typeof probe.insert_drawing === 'function';
      try { probe.close(); } catch (_) {}
    } catch (_) {}
  }

  // PDF capabilities
  if (wasmModule) {
    capabilities.canOpenPdf = typeof wasmModule.WasmPdfEditor === 'function'
      || typeof wasmModule.WasmPdfEditor?.open === 'function';
    capabilities.canEditPdf = capabilities.canOpenPdf;
    capabilities.canSignPdf = capabilities.canOpenPdf
      && typeof wasmModule.WasmPdfEditor?.prototype?.sign === 'function';
    capabilities.canFillPdfForms = capabilities.canOpenPdf;
    capabilities.canManagePdfPages = capabilities.canOpenPdf;
  }

  // Collaboration: requires relay URL or integrated mode
  const hasRelay = !!(config.relayUrl);
  const isIntegrated = config.mode === 'integrated';
  capabilities.canCollaborate = hasRelay || isIntegrated;

  // Spreadsheet
  capabilities.canOpenSpreadsheet = true; // Always available via built-in grid

  // AI: requires configured URL
  capabilities.canUseAI = !!(config.aiUrl) && config.enableAI !== false;

  // Admin
  capabilities.canAccessAdmin = true; // Available if server is running

  console.debug('[capabilities]', { ...capabilities });
}

/**
 * Check a single capability.
 * @param {string} name
 * @returns {boolean}
 */
export function can(name) {
  return capabilities[name] === true;
}

/**
 * Disable a DOM element if a capability is missing.
 * Adds 'disabled' class and sets a tooltip explaining why.
 *
 * @param {HTMLElement|null} el
 * @param {string} capName
 * @param {string} [label] - Human-readable feature name for the tooltip
 */
export function gateElement(el, capName, label) {
  if (!el) return;
  if (!capabilities[capName]) {
    el.classList.add('disabled');
    el.title = `${label || capName}: not available in this build`;
    el.setAttribute('aria-disabled', 'true');
  }
}
