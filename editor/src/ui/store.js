// Lightweight reactive store for UI state.
//
// Provides a simple pub/sub model for views/components that need to react
// to state changes without tight coupling. Does NOT replace the main
// editor `state` object — this is specifically for UI-layer concerns
// like view routing, modal state, and async request status.
//
// Usage:
//   import { uiStore } from './ui/store.js';
//   uiStore.set('currentView', 'pdf');
//   uiStore.on('currentView', (val) => switchView(val));

/** @type {Map<string, any>} */
const _state = new Map();

/** @type {Map<string, Set<Function>>} */
const _listeners = new Map();

export const uiStore = {
  /**
   * Get a value from the store.
   * @param {string} key
   * @param {*} [fallback]
   */
  get(key, fallback) {
    return _state.has(key) ? _state.get(key) : fallback;
  },

  /**
   * Set a value and notify subscribers.
   * @param {string} key
   * @param {*} value
   */
  set(key, value) {
    const prev = _state.get(key);
    _state.set(key, value);
    if (prev !== value) {
      const subs = _listeners.get(key);
      if (subs) subs.forEach(fn => { try { fn(value, prev); } catch (e) { console.error('[store]', key, e); } });
    }
  },

  /**
   * Subscribe to changes on a key. Returns unsubscribe function.
   * @param {string} key
   * @param {Function} fn
   * @returns {() => void}
   */
  on(key, fn) {
    if (!_listeners.has(key)) _listeners.set(key, new Set());
    _listeners.get(key).add(fn);
    return () => _listeners.get(key)?.delete(fn);
  },

  /**
   * Batch-set multiple keys, notifying once per key.
   * @param {Record<string, any>} updates
   */
  batch(updates) {
    for (const [key, value] of Object.entries(updates)) {
      this.set(key, value);
    }
  },
};

// Initialize default UI state
uiStore.batch({
  currentView: 'editor',       // 'editor' | 'spreadsheet' | 'pdf' | 'admin'
  activeModal: null,            // string ID of open modal, or null
  asyncStatus: 'idle',          // 'idle' | 'loading' | 'success' | 'error'
  asyncError: null,             // error message string, or null
});
