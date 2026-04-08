# Spell Check Specification — M13.2

**Reference**: OnlyOffice `ParagraphSpellChecker.js`, `DocumentSpellChecker.js`, Hunspell WASM
**Status**: Specification

---

## Architecture

```
Main Thread                          Web Worker (spell-worker.js)
-----------                          ---------------------------
Word Collection                      Hunspell WASM Engine
  (per paragraph)                      Dictionary Loading (lazy)
       |                                   |
       v                                   v
  postMessage({                      Check words → [bool]
    type: "spell",                   Suggest → [string[]]
    words: [...],                         |
    langs: [...]                          v
  })                                 postMessage({
       |                               type: "spell-result",
       v                               paraId, correct: [...]
  Apply Results                      })
    Mark misspelled
    Render underlines
```

## Word Collection Algorithm

1. Walk paragraph runs character-by-character
2. `isWordChar(ch)`: Unicode letter or digit
3. `isApostrophe(ch)`: 0x0027, 0x02BC, 0x02BD, 0x2018, 0x2019
4. Accumulate word characters, flush on non-word boundary
5. Strip leading/trailing apostrophes
6. Track run index + offset for each word (for underline positioning)
7. Per-word language from run's `Language` attribute (BiDi-aware: LTR → Lang.Val, RTL → Lang.Bidi)
8. Performance limit: 2000 elements per paragraph per pass

## Dictionary System

- Format: Hunspell `.aff` + `.dic` files
- WASM: Compile nuspell or hunspell-reader to WASM
- Lazy loading: only load dictionaries for languages found in document
- LRU cache: max 3 simultaneous dictionaries, evict least-recently-used
- Default: `en_US` bundled, others downloadable

## Red Underline Rendering

### Canvas Mode (primary)
- During layout paint phase, collect misspelled word X-ranges per line
- After drawing text, draw red horizontal lines at baseline + descent + 1px
- Line style: solid, 1px width, color `rgb(255, 0, 0)`
- Use separate collection pass (deferred rendering pattern)

### DOM Mode (fallback)
- Wrap misspelled text in `<span class="misspelled">` with CSS `text-decoration: underline wavy red`

## Context Menu

On right-click over misspelled word:
1. Top section: up to 5 suggestions (clickable to replace)
2. Separator
3. "Ignore" — mark this instance as correct
4. "Ignore All" — add to document-level ignore list
5. "Add to Dictionary" — add to persistent custom dictionary
6. Separator
7. Normal context menu items continue below

## Settings

- `spellCheckEnabled`: boolean (localStorage `s1-spellcheck`)
- `ignoreAllCaps`: boolean (skip words that are ALL UPPERCASE)
- `ignoreWordsWithNumbers`: boolean (skip "abc123")
- `customDictionary`: string[] (localStorage `s1-custom-dict`)
- `documentIgnoreList`: Set<string> (per-document, in-memory)

## Incremental Checking

- Timer-based: check 50 paragraphs per tick (requestIdleCallback or setTimeout 100ms)
- On text change: re-check only modified paragraph
- On document open: queue all paragraphs for background check
- Cancel pending checks on document close

## API

```javascript
// Public
initSpellCheck()                    // Initialize worker + load default dict
destroySpellCheck()                 // Terminate worker
spellCheckParagraph(paraId)         // Queue single paragraph
spellCheckAll()                     // Queue all paragraphs
addToCustomDictionary(word)         // Persistent
removeFromCustomDictionary(word)
ignoreWord(word)                    // Document-level
ignoreAllWord(word)                 // Document-level
replaceWord(paraId, offset, len, replacement)
getSuggestions(word, lang) → Promise<string[]>
isSpellCheckEnabled() → boolean
setSpellCheckEnabled(bool)
```
