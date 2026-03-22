// Auto-correct map and enabled state helpers.
// Extracted from toolbar-handlers.js to break circular dependency.

const AC_STORAGE_KEY = 's1-autocorrect';
const AC_ENABLED_KEY = 's1-autocorrect-enabled';

const DEFAULT_AUTOCORRECT = {
  'teh': 'the', 'adn': 'and', 'hte': 'the', 'taht': 'that',
  'wiht': 'with', 'thier': 'their', 'recieve': 'receive',
  'occured': 'occurred', 'seperate': 'separate', 'definately': 'definitely',
  'accomodate': 'accommodate', 'acheive': 'achieve', 'occurence': 'occurrence',
  'enviroment': 'environment', 'goverment': 'government', 'begining': 'beginning',
  'beleive': 'believe', 'calender': 'calendar', 'collegue': 'colleague',
  'commitee': 'committee', 'concensus': 'consensus',
};

export function getAutoCorrectMap() {
  try {
    const raw = localStorage.getItem(AC_STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch (_) {}
  return { ...DEFAULT_AUTOCORRECT };
}

export function isAutoCorrectEnabled() {
  try {
    const val = localStorage.getItem(AC_ENABLED_KEY);
    if (val === null) return true; // enabled by default
    return val === 'true';
  } catch (_) {}
  return true;
}
