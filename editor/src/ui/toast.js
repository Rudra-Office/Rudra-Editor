// Toast notification module — shared across editor and admin.
//
// Usage:
//   import { showToast } from './ui/toast.js';
//   showToast('Document saved', 'success');
//   showToast('Upload failed', 'error');

let _container = null;
const TOAST_DURATION = 3500;

function ensureContainer() {
  if (_container && _container.isConnected) return _container;
  _container = document.createElement('div');
  _container.className = 's1-toast-container';
  _container.setAttribute('role', 'alert');
  _container.setAttribute('aria-live', 'polite');
  Object.assign(_container.style, {
    position: 'fixed',
    bottom: '20px',
    right: '20px',
    zIndex: '99999',
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    pointerEvents: 'none',
  });
  document.body.appendChild(_container);
  return _container;
}

/**
 * Show a toast notification.
 *
 * @param {string} message
 * @param {'info'|'success'|'error'|'warning'} [type='info']
 * @param {number} [duration=3500] — ms before auto-dismiss. 0 = persistent.
 * @returns {HTMLElement} — the toast element (for manual removal)
 */
export function showToast(message, type = 'info', duration = TOAST_DURATION) {
  const container = ensureContainer();
  const el = document.createElement('div');
  el.className = `s1-toast s1-toast-${type}`;
  el.textContent = message;
  Object.assign(el.style, {
    padding: '10px 20px',
    borderRadius: '6px',
    color: '#fff',
    fontSize: '13px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    boxShadow: '0 2px 8px rgba(0,0,0,0.2)',
    opacity: '0',
    transition: 'opacity 0.3s, transform 0.3s',
    transform: 'translateY(10px)',
    pointerEvents: 'auto',
    cursor: 'pointer',
    maxWidth: '400px',
    wordBreak: 'break-word',
    background: type === 'success' ? '#2e7d32'
      : type === 'error' ? '#c62828'
      : type === 'warning' ? '#e65100'
      : '#333',
  });

  el.addEventListener('click', () => dismiss(el));
  container.appendChild(el);

  // Trigger enter animation
  requestAnimationFrame(() => {
    el.style.opacity = '1';
    el.style.transform = 'translateY(0)';
  });

  if (duration > 0) {
    setTimeout(() => dismiss(el), duration);
  }
  return el;
}

function dismiss(el) {
  el.style.opacity = '0';
  el.style.transform = 'translateY(10px)';
  setTimeout(() => el.remove(), 300);
}
