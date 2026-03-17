// Find & Replace
import { state, $ } from './state.js';
import { renderDocument, syncAllText } from './render.js';
import { updateUndoRedo } from './toolbar.js';
import { broadcastOp } from './collab.js';

let _findRefreshTimer = null;
let _matchCase = false;
let _wholeWord = false;

export function initFind() {
  // E1.5: Register callback so render.js can trigger find refresh without circular import
  state._onTextChanged = refreshFindIfOpen;

  $('btnFind').addEventListener('click', () => {
    $('findBar').classList.add('show');
    $('findInput').focus();
  });

  $('findClose').addEventListener('click', () => {
    $('findBar').classList.remove('show');
    clearHighlights();
  });

  $('findInput').addEventListener('input', () => doFind());
  $('findNext').addEventListener('click', () => navigateMatch(1));
  $('findPrev').addEventListener('click', () => navigateMatch(-1));

  $('replaceBtn').addEventListener('click', () => doReplace());
  $('replaceAllBtn').addEventListener('click', () => doReplaceAll());

  // Match case toggle
  $('findMatchCase').addEventListener('click', () => {
    _matchCase = !_matchCase;
    $('findMatchCase').classList.toggle('active', _matchCase);
    doFind();
  });

  // Whole word toggle
  $('findWholeWord').addEventListener('click', () => {
    _wholeWord = !_wholeWord;
    $('findWholeWord').classList.toggle('active', _wholeWord);
    doFind();
  });

  // Escape to close, Tab to cycle within find bar
  const findBarKeydown = e => {
    if (e.key === 'Escape') { $('findBar').classList.remove('show'); clearHighlights(); }
    if (e.key === 'Tab') {
      e.preventDefault();
      const focusable = $('findBar').querySelectorAll('input, button');
      const idx = Array.from(focusable).indexOf(document.activeElement);
      const next = e.shiftKey ? (idx - 1 + focusable.length) % focusable.length : (idx + 1) % focusable.length;
      focusable[next].focus();
    }
  };
  $('findInput').addEventListener('keydown', e => {
    findBarKeydown(e);
    if (e.key === 'Enter') navigateMatch(e.shiftKey ? -1 : 1);
    // Alt+C = toggle match case, Alt+W = toggle whole word
    if (e.altKey && e.key === 'c') { e.preventDefault(); $('findMatchCase').click(); }
    if (e.altKey && e.key === 'w') { e.preventDefault(); $('findWholeWord').click(); }
  });
  $('replaceInput').addEventListener('keydown', e => {
    findBarKeydown(e);
    if (e.key === 'Enter') doReplace();
  });
}

function doFind() {
  clearHighlights();
  let query = $('findInput').value;
  if (!query || !state.doc) { $('findCount').textContent = ''; return; }

  syncAllText();

  // Whole word: wrap query in word boundary markers for DOM fallback
  // For WASM find_text, we filter results post-hoc
  const caseSensitive = _matchCase;

  try {
    const results = JSON.parse(state.doc.find_text(query, caseSensitive));

    // Filter for whole word matches if enabled
    let filtered = results;
    if (_wholeWord) {
      filtered = results.filter(m => {
        const page = $('pageContainer');
        const el = page.querySelector(`[data-node-id="${m.nodeId}"]`);
        if (!el) return true;
        const text = el.textContent || '';
        const chars = Array.from(text);
        const before = m.offset > 0 ? chars[m.offset - 1] : ' ';
        const after = m.offset + m.length < chars.length ? chars[m.offset + m.length] : ' ';
        return /\W/.test(before) && /\W/.test(after);
      });
    }

    state.findMatches = filtered;
    state.findIndex = filtered.length > 0 ? 0 : -1;
    $('findCount').textContent = filtered.length + ' match' + (filtered.length !== 1 ? 'es' : '');

    // Highlight matches in DOM
    filtered.forEach((m, i) => {
      highlightMatch(m, i === state.findIndex);
    });
  } catch (_) {
    // find_text may not be available — fall back to DOM search
    domFind(query);
  }
}

function domFind(query) {
  const page = $('pageContainer');
  const text = page.textContent || '';
  const searchIn = _matchCase ? text : text.toLowerCase();
  const q = _matchCase ? query : query.toLowerCase();
  let count = 0, idx = 0;
  while ((idx = searchIn.indexOf(q, idx)) !== -1) {
    if (_wholeWord) {
      const before = idx > 0 ? searchIn[idx - 1] : ' ';
      const after = idx + q.length < searchIn.length ? searchIn[idx + q.length] : ' ';
      if (/\w/.test(before) || /\w/.test(after)) { idx += q.length; continue; }
    }
    count++;
    idx += q.length;
  }
  state.findMatches = [];
  state.findIndex = -1;
  $('findCount').textContent = count + ' match' + (count !== 1 ? 'es' : '');
}

function highlightMatch(match, active) {
  const page = $('pageContainer');
  const el = page.querySelector(`[data-node-id="${match.nodeId}"]`);
  if (!el) return;

  // Walk text nodes to find the match offset
  const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT, null);
  let counted = 0, node;
  while ((node = walker.nextNode())) {
    const chars = Array.from(node.textContent);
    const nodeStart = counted;
    const nodeEnd = counted + chars.length;
    if (match.offset >= nodeStart && match.offset < nodeEnd) {
      const localOffset = match.offset - nodeStart;
      // Convert char offset to string offset (for surrogate pairs)
      let strOff = 0;
      for (let i = 0; i < localOffset && i < chars.length; i++) strOff += chars[i].length;
      let endStrOff = strOff;
      for (let i = localOffset; i < localOffset + match.length && i < chars.length; i++) endStrOff += chars[i].length;

      try {
        const range = document.createRange();
        range.setStart(node, strOff);
        range.setEnd(node, Math.min(endStrOff, node.textContent.length));
        const span = document.createElement('mark');
        span.className = active ? 'find-highlight active' : 'find-highlight';
        // Use extractContents instead of surroundContents to handle cross-element ranges safely
        span.appendChild(range.extractContents());
        range.insertNode(span);
        if (active) span.scrollIntoView({ block: 'center', behavior: 'smooth' });
      } catch (_) {}
      return;
    }
    counted = nodeEnd;
  }
}

function clearHighlights() {
  const container = $('pageContainer');
  if (!container) return;
  container.querySelectorAll('mark.find-highlight').forEach(m => {
    const parent = m.parentNode;
    while (m.firstChild) parent.insertBefore(m.firstChild, m);
    m.remove();
    parent.normalize();
  });
}

function navigateMatch(dir) {
  if (state.findMatches.length === 0) return;
  clearHighlights();
  state.findIndex = (state.findIndex + dir + state.findMatches.length) % state.findMatches.length;
  state.findMatches.forEach((m, i) => highlightMatch(m, i === state.findIndex));
  $('findCount').textContent = (state.findIndex + 1) + '/' + state.findMatches.length;
}

function doReplace() {
  if (!state.doc || state.findIndex < 0) return;
  const match = state.findMatches[state.findIndex];
  const replacement = $('replaceInput').value;
  syncAllText();
  try {
    state.doc.replace_text(match.nodeId, match.offset, match.length, replacement);
    broadcastOp({ action: 'replaceText', nodeId: match.nodeId, offset: match.offset, length: match.length, replacement });
    renderDocument();
    updateUndoRedo();
    doFind(); // re-search
  } catch (e) { console.error('replace:', e); }
}

function doReplaceAll() {
  if (!state.doc) return;
  const query = $('findInput').value;
  const replacement = $('replaceInput').value;
  if (!query) return;
  syncAllText();
  try {
    const count = state.doc.replace_all(query, replacement, _matchCase);
    broadcastOp({ action: 'replaceAll', query, replacement, caseSensitive: _matchCase });
    renderDocument();
    updateUndoRedo();
    $('findCount').textContent = count + ' replaced';
    state.findMatches = [];
    state.findIndex = -1;
  } catch (e) { console.error('replace all:', e); }
}

/**
 * E1.5: Re-run find if the find bar is open.
 * Debounced to 300ms so rapid typing doesn't cause perf issues.
 */
export function refreshFindIfOpen() {
  if (!$('findBar').classList.contains('show')) return;
  if (!$('findInput').value) return;
  clearTimeout(_findRefreshTimer);
  _findRefreshTimer = setTimeout(() => {
    // Remember the previous match position to stay near it
    const prevMatch = state.findIndex >= 0 && state.findMatches[state.findIndex]
      ? state.findMatches[state.findIndex] : null;
    doFind();
    // Try to restore closest match index
    if (prevMatch && state.findMatches.length > 0) {
      let best = 0, bestDist = Infinity;
      state.findMatches.forEach((m, i) => {
        const dist = m.nodeId === prevMatch.nodeId
          ? Math.abs(m.offset - prevMatch.offset)
          : Infinity;
        if (dist < bestDist) { bestDist = dist; best = i; }
      });
      if (best !== state.findIndex) {
        clearHighlights();
        state.findIndex = best;
        state.findMatches.forEach((m, i) => highlightMatch(m, i === state.findIndex));
        $('findCount').textContent = (state.findIndex + 1) + '/' + state.findMatches.length;
      }
    }
  }, 300);
}
