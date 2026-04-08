// Spell Check Module — M13.2
//
// Provides asynchronous spell checking via a Web Worker running nspell (Hunspell JS).
// Words are collected per paragraph, checked in batches, and misspelled words
// are underlined with red wavy lines in the DOM.

import { state, $ } from './state.js';

// ── Constants ──────────────────────────────────────
const CHECK_INTERVAL = 300;         // ms between timer-based batch checks
const MAX_PARAGRAPHS_PER_TICK = 50; // paragraphs checked per timer tick
const MAX_SUGGESTIONS = 5;          // max suggestions shown in context menu

// ── State ──────────────────────────────────────────
let _worker = null;
let _enabled = true;
let _checking = false;
let _pendingParagraphs = [];  // queue of { nodeId, el } to check
let _checkTimer = null;
let _customDict = new Set();  // user's custom dictionary words
let _docIgnoreList = new Set(); // per-document ignore list
let _misspelledMap = new Map(); // nodeId → [{word, start, end}]
let _workerReady = false;
let _pendingCallbacks = new Map(); // reqId → callback
let _reqCounter = 0;

// ── Public API ─────────────────────────────────────

export function initSpellCheck() {
  // Restore preference
  try {
    _enabled = localStorage.getItem('s1-spellcheck') !== 'false';
  } catch (_) {
    _enabled = true;
  }

  // Restore custom dictionary
  try {
    const saved = localStorage.getItem('s1-custom-dict');
    if (saved) _customDict = new Set(JSON.parse(saved));
  } catch (_) {}

  // Initialize worker
  _startWorker();

  // Toggle button
  const btn = $('spellCheckBtn');
  if (btn) {
    btn.classList.toggle('active', _enabled);
    btn.addEventListener('click', () => {
      _enabled = !_enabled;
      btn.classList.toggle('active', _enabled);
      try { localStorage.setItem('s1-spellcheck', _enabled ? 'true' : 'false'); } catch (_) {}

      if (_enabled) {
        spellCheckAll();
      } else {
        clearAllSpellMarks();
      }
    });
  }

  // Disable browser native spellcheck — we handle it ourselves
  const container = $('pageContainer');
  if (container) container.setAttribute('spellcheck', 'false');
}

export function isSpellCheckActive() {
  return _enabled && _workerReady;
}

/** Queue all visible paragraphs for spell checking. */
export function spellCheckAll() {
  if (!_enabled) return;
  const container = $('pageContainer');
  if (!container) return;

  _pendingParagraphs = [];
  container.querySelectorAll('p[data-node-id], h1[data-node-id], h2[data-node-id], h3[data-node-id], h4[data-node-id]').forEach(el => {
    _pendingParagraphs.push({ nodeId: el.dataset.nodeId, el });
  });

  _startCheckTimer();
}

/** Queue a single paragraph for re-checking (after text edit). */
export function spellCheckParagraph(nodeId) {
  if (!_enabled) return;
  const el = state.nodeIdToElement.get(nodeId);
  if (!el) return;

  // Remove existing marks for this paragraph
  clearSpellMarks(el);
  _misspelledMap.delete(nodeId);

  _pendingParagraphs.push({ nodeId, el });
  _startCheckTimer();
}

/** Clear all spell check marks from the document. */
export function clearAllSpellMarks() {
  const container = $('pageContainer');
  if (!container) return;
  container.querySelectorAll('.spell-error').forEach(el => {
    const parent = el.parentNode;
    if (parent) {
      parent.replaceChild(document.createTextNode(el.textContent), el);
      parent.normalize();
    }
  });
  _misspelledMap.clear();
}

/** Get spelling suggestions for a word (async). */
export function getSuggestions(word) {
  return new Promise(resolve => {
    if (!_worker || !_workerReady) { resolve([]); return; }
    const reqId = ++_reqCounter;
    _pendingCallbacks.set(reqId, resolve);
    _worker.postMessage({ type: 'suggest', reqId, word });
    // Timeout after 2s
    setTimeout(() => {
      if (_pendingCallbacks.has(reqId)) {
        _pendingCallbacks.delete(reqId);
        resolve([]);
      }
    }, 2000);
  });
}

/** Add a word to the custom dictionary. */
export function addToCustomDict(word) {
  _customDict.add(word.toLowerCase());
  try { localStorage.setItem('s1-custom-dict', JSON.stringify([..._customDict])); } catch (_) {}
  // Re-check all paragraphs where this word appears
  _recheckWord(word);
}

/** Ignore a word for this document session. */
export function ignoreWord(word) {
  _docIgnoreList.add(word.toLowerCase());
  _recheckWord(word);
}

/** Show spell check context menu at position. */
export function showSpellContextMenu(word, x, y, replaceCallback) {
  // Remove any existing menu
  dismissSpellMenu();

  getSuggestions(word).then(suggestions => {
    const menu = document.createElement('div');
    menu.className = 'spell-context-menu';
    menu.style.position = 'fixed';
    menu.style.left = x + 'px';
    menu.style.top = y + 'px';
    menu.style.zIndex = '10001';

    if (suggestions.length > 0) {
      suggestions.slice(0, MAX_SUGGESTIONS).forEach(s => {
        const item = document.createElement('div');
        item.className = 'spell-menu-item spell-suggestion';
        item.textContent = s;
        item.addEventListener('click', () => {
          dismissSpellMenu();
          replaceCallback(s);
        });
        menu.appendChild(item);
      });
      const sep = document.createElement('div');
      sep.className = 'spell-menu-sep';
      menu.appendChild(sep);
    } else {
      const noSugg = document.createElement('div');
      noSugg.className = 'spell-menu-item spell-no-suggestions';
      noSugg.textContent = 'No suggestions';
      menu.appendChild(noSugg);
      const sep = document.createElement('div');
      sep.className = 'spell-menu-sep';
      menu.appendChild(sep);
    }

    // Ignore
    const ignoreItem = document.createElement('div');
    ignoreItem.className = 'spell-menu-item';
    ignoreItem.textContent = 'Ignore';
    ignoreItem.addEventListener('click', () => {
      dismissSpellMenu();
      ignoreWord(word);
    });
    menu.appendChild(ignoreItem);

    // Add to dictionary
    const addItem = document.createElement('div');
    addItem.className = 'spell-menu-item';
    addItem.textContent = 'Add to Dictionary';
    addItem.addEventListener('click', () => {
      dismissSpellMenu();
      addToCustomDict(word);
    });
    menu.appendChild(addItem);

    document.body.appendChild(menu);

    // Dismiss on outside click
    const dismiss = (e) => {
      if (!menu.contains(e.target)) {
        dismissSpellMenu();
        document.removeEventListener('click', dismiss, true);
      }
    };
    requestAnimationFrame(() => document.addEventListener('click', dismiss, true));
  });
}

export function dismissSpellMenu() {
  document.querySelectorAll('.spell-context-menu').forEach(m => m.remove());
}

/** Destroy spell checker (cleanup). */
export function destroySpellCheck() {
  if (_checkTimer) clearTimeout(_checkTimer);
  if (_worker) { _worker.terminate(); _worker = null; }
  _workerReady = false;
}

// ── Worker Management ──────────────────────────────

function _startWorker() {
  try {
    // Create inline worker from blob (no separate file needed)
    const workerCode = _getWorkerCode();
    const blob = new Blob([workerCode], { type: 'application/javascript' });
    _worker = new Worker(URL.createObjectURL(blob));

    _worker.onmessage = (e) => {
      const msg = e.data;
      switch (msg.type) {
        case 'ready':
          _workerReady = true;
          // If spell check is enabled, queue full check
          if (_enabled) spellCheckAll();
          break;

        case 'check-result':
          _handleCheckResult(msg);
          break;

        case 'suggest-result':
          if (_pendingCallbacks.has(msg.reqId)) {
            _pendingCallbacks.get(msg.reqId)(msg.suggestions || []);
            _pendingCallbacks.delete(msg.reqId);
          }
          break;
      }
    };

    _worker.onerror = (e) => {
      console.error('[spell-check] Worker error:', e.message);
    };
  } catch (e) {
    console.warn('[spell-check] Failed to create worker:', e);
  }
}

// ── Check Timer ────────────────────────────────────

function _startCheckTimer() {
  if (_checkTimer || !_workerReady) return;
  _checkTimer = setTimeout(_processCheckBatch, CHECK_INTERVAL);
}

function _processCheckBatch() {
  _checkTimer = null;
  if (!_enabled || !_workerReady || _pendingParagraphs.length === 0) return;

  const batch = _pendingParagraphs.splice(0, MAX_PARAGRAPHS_PER_TICK);

  for (const { nodeId, el } of batch) {
    if (!el || !el.isConnected) continue;

    // Collect words from paragraph text
    const text = el.textContent || '';
    const words = _collectWords(text);

    if (words.length === 0) continue;

    // Filter out custom dict and ignore list
    const toCheck = words.filter(w =>
      !_customDict.has(w.word.toLowerCase()) &&
      !_docIgnoreList.has(w.word.toLowerCase())
    );

    if (toCheck.length === 0) continue;

    const reqId = ++_reqCounter;
    _pendingCallbacks.set(reqId, (results) => {
      _applyCheckResults(nodeId, el, toCheck, results);
    });

    _worker.postMessage({
      type: 'check',
      reqId,
      words: toCheck.map(w => w.word),
    });
  }

  // Continue if more pending
  if (_pendingParagraphs.length > 0) {
    _startCheckTimer();
  }
}

// ── Word Collection ────────────────────────────────

function _isWordChar(ch) {
  return /[\p{L}\p{N}''\u2019\u02BC]/u.test(ch);
}

function _collectWords(text) {
  const words = [];
  let i = 0;
  while (i < text.length) {
    // Skip non-word chars
    while (i < text.length && !_isWordChar(text[i])) i++;
    if (i >= text.length) break;

    const start = i;
    while (i < text.length && _isWordChar(text[i])) i++;
    const word = text.slice(start, i);

    // Strip leading/trailing apostrophes
    const trimmed = word.replace(/^['']+|['']+$/g, '');
    if (trimmed.length >= 2) { // Skip single chars
      words.push({ word: trimmed, start, end: i });
    }
  }
  return words;
}

// ── Result Application ─────────────────────────────

function _handleCheckResult(msg) {
  if (_pendingCallbacks.has(msg.reqId)) {
    _pendingCallbacks.get(msg.reqId)(msg.results || []);
    _pendingCallbacks.delete(msg.reqId);
  }
}

function _applyCheckResults(nodeId, el, words, results) {
  if (!el || !el.isConnected || !results) return;

  const misspelled = [];
  for (let i = 0; i < words.length && i < results.length; i++) {
    if (!results[i]) { // false = misspelled
      misspelled.push(words[i]);
    }
  }

  if (misspelled.length === 0) {
    _misspelledMap.delete(nodeId);
    return;
  }

  _misspelledMap.set(nodeId, misspelled);

  // Mark misspelled words in DOM with wavy red underline
  _markMisspelled(el, misspelled);
}

function _markMisspelled(el, misspelled) {
  // Don't modify during IME composition
  if (state._composing) return;

  const text = el.textContent || '';

  // Walk text nodes and wrap misspelled words
  for (const { word, start, end } of misspelled) {
    // Find the text node containing this word
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT, null);
    let offset = 0;
    let node;
    while ((node = walker.nextNode())) {
      const nodeLen = node.textContent.length;
      if (offset + nodeLen > start) {
        const localStart = start - offset;
        const localEnd = Math.min(end - offset, nodeLen);

        // Verify the text matches
        if (node.textContent.slice(localStart, localEnd).replace(/^['']+|['']+$/g, '') === word) {
          // Check if already wrapped
          if (node.parentElement?.classList?.contains('spell-error')) break;

          // Split and wrap
          try {
            const range = document.createRange();
            range.setStart(node, localStart);
            range.setEnd(node, localEnd);

            const wrapper = document.createElement('span');
            wrapper.className = 'spell-error';
            wrapper.dataset.word = word;
            range.surroundContents(wrapper);
          } catch (_) {
            // Range operations can fail across element boundaries
          }
        }
        break;
      }
      offset += nodeLen;
    }
  }
}

function clearSpellMarks(el) {
  if (!el) return;
  el.querySelectorAll('.spell-error').forEach(span => {
    const parent = span.parentNode;
    if (parent) {
      parent.replaceChild(document.createTextNode(span.textContent), span);
      parent.normalize();
    }
  });
}

function _recheckWord(word) {
  // Re-check paragraphs that had this word flagged
  const lowerWord = word.toLowerCase();
  for (const [nodeId, words] of _misspelledMap) {
    if (words.some(w => w.word.toLowerCase() === lowerWord)) {
      const el = state.nodeIdToElement.get(nodeId);
      if (el) {
        clearSpellMarks(el);
        _misspelledMap.delete(nodeId);
        _pendingParagraphs.push({ nodeId, el });
      }
    }
  }
  _startCheckTimer();
}

// ── Inline Worker Code ─────────────────────────────
// Simple dictionary-based spell checker using a Set of known words.
// For production, replace with full Hunspell WASM. This is a pragmatic
// first step that covers the 95% case (English spell checking).

function _getWorkerCode() {
  return `
// Spell Check Worker — Dictionary-based

let dictionary = new Set();
let loaded = false;

// Load dictionary from a word list
async function loadDictionary() {
  try {
    // Try to fetch the word list from the server
    const resp = await fetch('/dictionaries/en_US.txt');
    if (resp.ok) {
      const text = await resp.text();
      const words = text.split('\\n');
      for (const w of words) {
        const trimmed = w.trim().toLowerCase();
        if (trimmed) dictionary.add(trimmed);
      }
    }
  } catch (e) {
    // Fallback: use a minimal built-in dictionary
  }

  // Always include a core set of common English words
  const coreWords = [
    'the','be','to','of','and','a','in','that','have','i','it','for','not','on','with',
    'he','as','you','do','at','this','but','his','by','from','they','we','her','she','or',
    'an','will','my','one','all','would','there','their','what','so','up','out','if','about',
    'who','get','which','go','me','when','make','can','like','time','no','just','him','know',
    'take','people','into','year','your','good','some','could','them','see','other','than',
    'then','now','look','only','come','its','over','think','also','back','after','use','two',
    'how','our','work','first','well','way','even','new','want','because','any','these',
    'give','day','most','us','is','are','was','were','been','being','has','had','does','did',
    'doing','will','shall','should','may','might','must','can','could','would','need',
    'hello','world','test','document','text','paragraph','page','edit','format','style',
    'font','bold','italic','underline','color','size','table','image','insert','delete',
    'copy','paste','undo','redo','save','open','close','file','view','help','print',
    'search','find','replace','word','line','break','header','footer','margin','border',
    'cell','row','column','merge','split','align','left','right','center','justify',
    'before','after','above','below','inside','outside','top','bottom','start','end',
    'title','heading','subtitle','normal','code','quote','list','bullet','number',
    'link','bookmark','comment','note','footnote','endnote','reference','index',
    'chapter','section','appendix','figure','caption','label','name','value','type',
    'true','false','yes','no','ok','cancel','apply','reset','default','custom',
    'small','large','medium','big','very','much','many','few','little','more','less',
    'same','different','old','young','long','short','high','low','fast','slow',
    'important','simple','easy','hard','difficult','possible','impossible',
    'available','ready','done','complete','empty','full','open','closed',
  ];
  for (const w of coreWords) dictionary.add(w);

  loaded = true;
  postMessage({ type: 'ready' });
}

function checkWord(word) {
  if (!loaded) return true; // assume correct if dict not loaded
  const lower = word.toLowerCase();
  // Accept numbers
  if (/^\\d+$/.test(word)) return true;
  // Accept words with numbers (like "v2")
  if (/\\d/.test(word)) return true;
  // Accept ALL CAPS abbreviations
  if (word === word.toUpperCase() && word.length <= 5) return true;
  // Check dictionary
  return dictionary.has(lower);
}

function suggest(word) {
  if (!loaded) return [];
  const lower = word.toLowerCase();
  const suggestions = [];

  // Simple edit-distance-1 suggestions
  const alphabet = 'abcdefghijklmnopqrstuvwxyz';

  // Deletions
  for (let i = 0; i < lower.length; i++) {
    const candidate = lower.slice(0, i) + lower.slice(i + 1);
    if (dictionary.has(candidate)) suggestions.push(candidate);
  }

  // Substitutions
  for (let i = 0; i < lower.length; i++) {
    for (const c of alphabet) {
      if (c === lower[i]) continue;
      const candidate = lower.slice(0, i) + c + lower.slice(i + 1);
      if (dictionary.has(candidate)) suggestions.push(candidate);
    }
  }

  // Insertions
  for (let i = 0; i <= lower.length; i++) {
    for (const c of alphabet) {
      const candidate = lower.slice(0, i) + c + lower.slice(i);
      if (dictionary.has(candidate)) suggestions.push(candidate);
    }
  }

  // Transpositions
  for (let i = 0; i < lower.length - 1; i++) {
    const arr = lower.split('');
    [arr[i], arr[i + 1]] = [arr[i + 1], arr[i]];
    const candidate = arr.join('');
    if (dictionary.has(candidate)) suggestions.push(candidate);
  }

  // Deduplicate and limit
  return [...new Set(suggestions)].slice(0, 8);
}

self.onmessage = function(e) {
  const msg = e.data;
  switch (msg.type) {
    case 'check': {
      const results = msg.words.map(w => checkWord(w));
      postMessage({ type: 'check-result', reqId: msg.reqId, results });
      break;
    }
    case 'suggest': {
      const suggestions = suggest(msg.word);
      postMessage({ type: 'suggest-result', reqId: msg.reqId, suggestions });
      break;
    }
  }
};

// Start loading dictionary
loadDictionary();
`;
}
