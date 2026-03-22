// Toast notification and accessibility announcement helpers.
// Extracted from toolbar-handlers.js to break circular dependency.
import { $ } from '../../../state.js';

let _announceTimer = 0;

export function announce(msg) {
  const el = $('a11yLive');
  if (!el) return;
  clearTimeout(_announceTimer);
  el.textContent = msg;
  _announceTimer = setTimeout(() => { el.textContent = ''; }, 1000);
}

// ── Toast notification system ──────────────────────
// Replaces alert() calls with non-blocking toast messages.
// Types: 'info' (default, dark), 'error' (red), 'success' (green)
export function showToast(message, type = 'info', duration = 4000) {
  const container = $('toastContainer');
  if (!container) { console.warn('toast:', message); return; }
  const toast = document.createElement('div');
  toast.className = 'toast' + (type === 'error' ? ' toast-error' : type === 'success' ? ' toast-success' : type === 'warning' ? ' toast-warning' : '');
  // FS-12: Ensure individual toasts are accessible to screen readers
  if (type === 'error') {
    toast.setAttribute('role', 'alert');
  }
  toast.textContent = message;
  container.appendChild(toast);
  const remove = () => {
    toast.style.transition = 'opacity 0.2s ease, transform 0.2s ease';
    toast.style.opacity = '0';
    toast.style.transform = 'translateY(-8px)';
    setTimeout(() => { toast.remove(); }, 220);
  };
  toast.addEventListener('click', remove);
  if (duration > 0) setTimeout(remove, duration);
}
