/**
 * adapter.js — Bridge between s1engine WASM and OnlyOffice sdkjs
 *
 * M1: Opens DOCX via s1engine WASM, extracts plain text per paragraph,
 * and inserts into sdkjs editor. Formatting fidelity is M2 work.
 */

import init, { WasmEngine } from './pkg/s1engine_wasm.js';

let wasmEngine = null;
let wasmReady = false;
let currentDoc = null;

export async function initWasm() {
  if (wasmReady) return;
  await init();
  wasmEngine = new WasmEngine();
  wasmReady = true;
  console.log('[adapter] s1engine WASM initialized');
}

export async function openDocx(docxBytes, api) {
  if (!wasmReady) await initWasm();

  console.log('[adapter] Opening DOCX (' + docxBytes.length + ' bytes)...');

  var doc = wasmEngine.open(docxBytes);
  currentDoc = doc;

  var bodyChildren = JSON.parse(doc.body_children_json());
  console.log('[adapter] Document has ' + bodyChildren.length + ' top-level elements');

  var logicDoc = api.WordControl.m_oLogicDocument;
  if (!logicDoc) throw new Error('No logic document');

  // Clear existing content
  logicDoc.SelectAll();
  logicDoc.Remove(1, true, false, true);
  logicDoc.RemoveSelection();

  // Turn off underline (empty doc template has it on)
  try { api.put_TextPrUnderline(false); } catch(e) {}

  // Insert paragraphs as plain text
  var isFirst = true;
  for (var i = 0; i < bodyChildren.length; i++) {
    var child = bodyChildren[i];
    if (child.type !== 'Paragraph') continue;

    try {
      var text = doc.get_paragraph_text(child.id);

      if (!isFirst) {
        logicDoc.AddNewParagraph(false);
      }
      isFirst = false;

      if (text && text.length > 0) {
        for (var j = 0; j < text.length; j++) {
          var code = text.charCodeAt(j);
          if (code === 0x0A || code === 0x0D) continue;
          if (code === 0x09) {
            logicDoc.AddToParagraph(new AscWord.CRunTab());
          } else {
            logicDoc.AddToParagraph(new AscWord.CRunText(code));
          }
        }
      }
    } catch (e) {
      if (!isFirst) logicDoc.AddNewParagraph(false);
      isFirst = false;
    }
  }

  logicDoc.MoveCursorToStartPos(false);
  logicDoc.Recalculate();
  api.Resize();

  console.log('[adapter] Document loaded (' + bodyChildren.length + ' paragraphs)');
  return doc;
}

export function saveDocx() {
  if (!currentDoc) throw new Error('No document open');
  return currentDoc.export('docx');
}

export function getPlainText() {
  if (!currentDoc) throw new Error('No document open');
  return currentDoc.to_plain_text();
}
