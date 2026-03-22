// Status/state rendering helpers — loading, empty, error states.
//
// Usage:
//   import { renderLoading, renderError, renderEmpty } from './ui/status.js';
//   container.innerHTML = renderLoading('Fetching sessions...');

/**
 * Render a loading indicator.
 * @param {string} [message='Loading...']
 * @returns {string} HTML string
 */
export function renderLoading(message = 'Loading...') {
  return `<div class="s1-status s1-status-loading" style="text-align:center;color:#888;padding:24px;font-size:13px">${esc(message)}</div>`;
}

/**
 * Render an error state.
 * @param {string} message
 * @param {Function} [onRetry] — if provided, includes a retry button
 * @returns {string} HTML string
 */
export function renderError(message, onRetry) {
  const retryBtn = onRetry
    ? ' <button class="s1-retry-btn" style="margin-top:8px;padding:4px 12px;border:1px solid #c62828;background:transparent;color:#c62828;border-radius:4px;cursor:pointer;font-size:12px">Retry</button>'
    : '';
  return `<div class="s1-status s1-status-error" style="text-align:center;color:#c62828;padding:16px;font-size:13px">${esc(message)}${retryBtn}</div>`;
}

/**
 * Render an empty state.
 * @param {string} [message='No items']
 * @returns {string} HTML string
 */
export function renderEmpty(message = 'No items') {
  return `<div class="s1-status s1-status-empty" style="text-align:center;color:#aaa;padding:24px;font-size:13px">${esc(message)}</div>`;
}

/**
 * Escape HTML special characters.
 * @param {string} s
 * @returns {string}
 */
function esc(s) {
  const d = document.createElement('div');
  d.textContent = s != null ? String(s) : '';
  return d.innerHTML;
}
