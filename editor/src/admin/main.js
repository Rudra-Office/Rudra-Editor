// Admin panel — Vite entrypoint.
//
// Fetches data from /admin/api/* and renders the dashboard.
// Separated from the editor app for independent bundling and faster iteration.

import './styles.css';

// ─── Helpers ────────────────────────────────────────
function esc(s) { const d = document.createElement('div'); d.textContent = s != null ? String(s) : ''; return d.innerHTML; }
function fu(s) { if (s == null) return '-'; return s < 60 ? s + 's' : s < 3600 ? Math.floor(s / 60) + 'm' : Math.floor(s / 3600) + 'h'; }
function fs(b) { return b < 1024 ? b + 'B' : b < 1048576 ? (b / 1024).toFixed(1) + 'KB' : (b / 1048576).toFixed(1) + 'MB'; }

function toast(msg, ok) {
  const t = document.getElementById('toast');
  t.textContent = msg;
  t.className = 'toast show ' + (ok ? 'toast-ok' : 'toast-err');
  setTimeout(() => t.className = 'toast', 3000);
}

function la(eds) {
  if (!eds || !eds.length) return '-';
  const acts = eds.filter(e => e.last_activity).map(e => e.last_activity);
  if (!acts.length) return '-';
  acts.sort();
  const t = new Date(acts[acts.length - 1]);
  const ago = Math.floor((Date.now() - t.getTime()) / 1000);
  return ago < 0 ? 'now' : fu(ago) + ' ago';
}

async function fetchJson(url) {
  const r = await fetch(url);
  if (!r.ok) throw new Error(r.statusText);
  return r.json();
}

// ─── State ──────────────────────────────────────────
let paused = false;
let refreshTimer = null;

// ─── Tab Switching ──────────────────────────────────
document.getElementById('tabs').addEventListener('click', (e) => {
  const tab = e.target.closest('.tab');
  if (!tab) return;
  const name = tab.dataset.tab;
  document.querySelectorAll('.tab').forEach(t => t.classList.toggle('active', t.dataset.tab === name));
  document.querySelectorAll('.tab-panel').forEach(p => p.classList.toggle('active', p.id === 'panel-' + name));
});

// ─── Controls ───────────────────────────────────────
document.getElementById('refreshBtn').addEventListener('click', () => refresh());
document.getElementById('pauseBtn').addEventListener('click', () => {
  paused = !paused;
  document.getElementById('pauseBtn').textContent = paused ? 'Resume' : 'Pause';
  if (!paused) startRefresh();
});

function startRefresh() {
  clearInterval(refreshTimer);
  if (!paused) refreshTimer = setInterval(refresh, 10000);
}

function copyId(id) {
  navigator.clipboard?.writeText(id);
  toast('Copied: ' + id, true);
}

// ─── Main Refresh ───────────────────────────────────
async function refresh() {
  try {
    const [stats, sessions, config] = await Promise.all([
      fetchJson('/admin/api/stats'),
      fetchJson('/admin/api/sessions'),
      fetchJson('/admin/api/config'),
    ]);

    document.getElementById('ver').textContent = 'v' + stats.version;
    document.getElementById('lastUpdate').textContent = 'Updated: ' + new Date().toLocaleTimeString();

    // Stats cards
    document.getElementById('cards').innerHTML = `
      <div class="card"><div class="card-label">Uptime</div><div class="card-value">${fu(stats.uptime_secs)}</div></div>
      <div class="card"><div class="card-label">Sessions</div><div class="card-value">${stats.active_sessions}</div></div>
      <div class="card"><div class="card-label">Rooms</div><div class="card-value">${stats.active_rooms}</div></div>
      <div class="card"><div class="card-label">Editors</div><div class="card-value">${stats.total_editors}</div></div>
      <div class="card"><div class="card-label">Memory</div><div class="card-value">${stats.memory_mb.toFixed(1)}MB</div></div>`;

    // Sessions table
    const tb = document.getElementById('sessions');
    tb.innerHTML = sessions.sessions.length
      ? sessions.sessions.map(s => `<tr>
          <td class="id-cell" data-action="copy" data-id="${esc(s.file_id)}" title="Click to copy: ${esc(s.file_id)}">${esc(s.file_id.substring(0, 8))}</td>
          <td>${esc(s.filename)}</td><td>${esc(s.format)}</td><td>${fs(s.size)}</td>
          <td>${s.editor_count} <button class="btn-sm btn-outline" data-action="editors" data-id="${esc(s.file_id)}" title="View editors">...</button></td>
          <td>${la(s.editors)}</td>
          <td><span class="badge badge-${esc(s.status)}">${esc(s.status)}</span></td>
          <td>${fu(s.created_at_secs_ago)}</td>
          <td>
            <button class="btn-sm btn-primary" data-action="sync" data-id="${esc(s.file_id)}" title="Force sync">Sync</button>
            <button class="btn-sm btn-danger" data-action="close" data-id="${esc(s.file_id)}" title="Close session">Close</button>
          </td></tr>`).join('')
      : '<tr><td colspan="9" style="text-align:center;color:#ccc;padding:16px">No active sessions</td></tr>';

    // Config
    document.getElementById('config').textContent = JSON.stringify(config, null, 2);
  } catch (e) {
    document.getElementById('cards').innerHTML = '<div class="error-msg">Failed to load: ' + esc(e) + '</div>';
  }

  // Errors tab
  try {
    const err = await fetchJson('/admin/api/errors');
    const el = document.getElementById('errorsContent');
    if (!err.errors.length) {
      el.innerHTML = '<div class="loading">No recent errors</div>';
    } else {
      el.innerHTML = '<div class="table-wrap"><table><thead><tr><th>Time</th><th>Source</th><th>Message</th></tr></thead><tbody>' +
        err.errors.map(e => `<tr><td style="white-space:nowrap">${esc(e.timestamp?.substring(11, 19) || '')}</td><td style="color:#888;font-size:10px">${esc(e.source)}</td><td>${esc(e.message)}</td></tr>`).join('') +
        '</tbody></table></div><div style="font-size:11px;color:#888;margin-top:6px">Showing ' + err.errors.length + ' of ' + err.total + '</div>';
    }
  } catch (e) {
    document.getElementById('errorsContent').innerHTML = '<div class="error-msg">Failed: ' + esc(e) + '</div>';
  }

  // Health tab
  try {
    const h = await fetchJson('/admin/api/health');
    document.getElementById('healthContent').innerHTML = `
      <div class="cards" style="margin-bottom:12px">
        <div class="card"><div class="card-label">Status</div><div class="card-value" style="color:${h.status === 'ok' ? '#2e7d32' : '#c62828'}">${esc(h.status)}</div></div>
        <div class="card"><div class="card-label">PID</div><div class="card-value" style="font-size:18px">${h.pid || '-'}</div></div>
        <div class="card"><div class="card-label">Memory</div><div class="card-value">${(h.memory_mb || 0).toFixed(1)}MB</div></div>
        <div class="card"><div class="card-label">Uptime</div><div class="card-value">${fu(h.uptime_secs)}</div></div>
      </div>
      <pre>${JSON.stringify(h, null, 2)}</pre>`;
  } catch (e) {
    document.getElementById('healthContent').innerHTML = '<div class="error-msg">Failed: ' + esc(e) + '</div>';
  }
}

// ─── Event Delegation ───────────────────────────────
document.addEventListener('click', async (e) => {
  const btn = e.target.closest('[data-action]');
  if (!btn) return;
  const action = btn.dataset.action;
  const id = btn.dataset.id;

  if (action === 'copy' && id) {
    copyId(id);
  }

  if (action === 'close' && id) {
    if (!confirm('Close session ' + id.substring(0, 8) + '...?')) return;
    try {
      const r = await fetch('/admin/api/sessions/' + encodeURIComponent(id), { method: 'DELETE' });
      toast(r.ok ? 'Session closed' : 'Close failed: ' + r.statusText, r.ok);
    } catch (e) { toast('Close error: ' + e, false); }
    refresh();
  }

  if (action === 'sync' && id) {
    try {
      const r = await fetch('/admin/api/sessions/' + encodeURIComponent(id) + '/sync', { method: 'POST' });
      toast(r.ok ? 'Sync triggered' : 'Sync failed: ' + r.statusText, r.ok);
    } catch (e) { toast('Sync error: ' + e, false); }
  }

  if (action === 'editors' && id) {
    try {
      const r = await fetch('/admin/api/sessions/' + encodeURIComponent(id) + '/editors');
      const d = await r.json();
      const eds = d.editors || [];
      if (!eds.length) { toast('No editors connected', true); return; }
      alert('Editors in ' + id.substring(0, 8) + '...:\n\n' + eds.map(e => e.user_name + ' (' + e.mode + ')').join('\n'));
    } catch (e) { toast('Failed: ' + e, false); }
  }
});

// ─── Boot ───────────────────────────────────────────
refresh();
startRefresh();
