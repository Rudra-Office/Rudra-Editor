// Auto-correct map and enabled state helpers.
// Extracted from toolbar-handlers.js to break circular dependency.

const AC_STORAGE_KEY = 's1-autocorrect';
const AC_ENABLED_KEY = 's1-autocorrect-enabled';

const DEFAULT_AUTOCORRECT = {
  // Common typos
  'teh': 'the', 'adn': 'and', 'hte': 'the', 'taht': 'that',
  'wiht': 'with', 'thier': 'their', 'recieve': 'receive',
  'occured': 'occurred', 'seperate': 'separate', 'definately': 'definitely',
  'accomodate': 'accommodate', 'acheive': 'achieve', 'occurence': 'occurrence',
  'enviroment': 'environment', 'goverment': 'government', 'begining': 'beginning',
  'beleive': 'believe', 'calender': 'calendar', 'collegue': 'colleague',
  'commitee': 'committee', 'concensus': 'consensus',
  'apparantly': 'apparently', 'arguement': 'argument', 'basicly': 'basically',
  'buisness': 'business', 'catagory': 'category', 'completly': 'completely',
  'concious': 'conscious', 'dependant': 'dependent', 'embarass': 'embarrass',
  'explaination': 'explanation', 'familar': 'familiar', 'foriegn': 'foreign',
  'fourty': 'forty', 'grammer': 'grammar', 'harrass': 'harass',
  'immediatly': 'immediately', 'independant': 'independent', 'judgement': 'judgment',
  'knowlege': 'knowledge', 'liason': 'liaison', 'maintainance': 'maintenance',
  'millenium': 'millennium', 'neccessary': 'necessary', 'noticable': 'noticeable',
  'occassion': 'occasion', 'parliment': 'parliament', 'persistant': 'persistent',
  'posession': 'possession', 'prefered': 'preferred', 'privlege': 'privilege',
  'pronounciation': 'pronunciation', 'publically': 'publicly', 'recomend': 'recommend',
  'refered': 'referred', 'relevent': 'relevant', 'resistence': 'resistance',
  'responsability': 'responsibility', 'restauraunt': 'restaurant', 'rythm': 'rhythm',
  'sieze': 'seize', 'succesful': 'successful', 'suprise': 'surprise',
  'tendancy': 'tendency', 'tommorow': 'tomorrow', 'truely': 'truly',
  'untill': 'until', 'wich': 'which', 'writting': 'writing',
  'nto': 'not', 'yuo': 'you', 'cna': 'can', 'jsut': 'just',
  'dont': "don't", 'doesnt': "doesn't", 'didnt': "didn't",
  'wont': "won't", 'cant': "can't", 'shouldnt': "shouldn't",
  'wouldnt': "wouldn't", 'couldnt': "couldn't", 'hasnt': "hasn't",
  'hadnt': "hadn't", 'isnt': "isn't", 'wasnt': "wasn't",
  'arent': "aren't", 'werent': "weren't", 'Im': "I'm",
  'ive': "I've", 'youre': "you're", 'theyre': "they're",
  'weve': "we've", 'hes': "he's", 'shes': "she's",
  // Math symbols (OnlyOffice-style auto-correct)
  '(c)': '\u00A9',    // copyright
  '(r)': '\u00AE',    // registered
  '(tm)': '\u2122',   // trademark
  '...': '\u2026',    // ellipsis
  '->': '\u2192',     // right arrow
  '<-': '\u2190',     // left arrow
  '<->': '\u2194',    // left-right arrow
  '=>': '\u21D2',     // double right arrow
  '<=': '\u2264',     // less than or equal
  '>=': '\u2265',     // greater than or equal
  '!=': '\u2260',     // not equal
  '+-': '\u00B1',     // plus-minus
  '1/2': '\u00BD',    // one half
  '1/4': '\u00BC',    // one quarter
  '3/4': '\u00BE',    // three quarters
  '---': '\u2014',    // em dash
  '--': '\u2013',     // en dash
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
