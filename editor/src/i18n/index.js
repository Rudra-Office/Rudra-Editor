/**
 * i18n — Internationalization module for s1 Editor.
 *
 * Loads translation strings from JSON files. Consumers can provide
 * custom translations by calling setLocale() with a language code
 * and optional translation overrides.
 *
 * @example
 * ```js
 * import { t, setLocale } from './i18n/index.js';
 *
 * // Use default English
 * console.log(t('toolbar.bold')); // "Bold"
 *
 * // Switch to Spanish
 * setLocale('es', { toolbar: { bold: 'Negrita' } });
 * console.log(t('toolbar.bold')); // "Negrita"
 * ```
 */

import en from './en.json';

let _strings = en;
let _locale = 'en';

/**
 * Get a translated string by dot-notation key.
 * @param {string} key - Dot-notation key (e.g., 'toolbar.bold')
 * @param {Record<string, string>} [params] - Interpolation params (e.g., {count: '3'})
 * @returns {string} The translated string, or the key itself if not found
 */
export function t(key, params) {
  const parts = key.split('.');
  let value = _strings;
  for (const part of parts) {
    if (value && typeof value === 'object' && part in value) {
      value = value[part];
    } else {
      return key; // Key not found — return as-is
    }
  }
  if (typeof value !== 'string') return key;

  // Interpolate {param} placeholders
  if (params) {
    return value.replace(/\{(\w+)\}/g, (_, k) => params[k] ?? `{${k}}`);
  }
  return value;
}

/**
 * Set the active locale and optionally merge translation overrides.
 * @param {string} locale - Language code (e.g., 'en', 'es', 'fr')
 * @param {object} [overrides] - Partial translation object to merge
 */
export function setLocale(locale, overrides) {
  _locale = locale;
  if (overrides) {
    _strings = deepMerge(en, overrides);
  }
}

/**
 * Get the current locale code.
 * @returns {string}
 */
export function getLocale() {
  return _locale;
}

/**
 * Load a full translation JSON object (replaces all strings).
 * @param {object} translations - Complete translation object matching en.json structure
 */
export function loadTranslations(translations) {
  _strings = deepMerge(en, translations);
}

/** Deep merge two objects. Source overrides target. */
function deepMerge(target, source) {
  const result = { ...target };
  for (const key of Object.keys(source)) {
    if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
      result[key] = deepMerge(target[key] || {}, source[key]);
    } else {
      result[key] = source[key];
    }
  }
  return result;
}
