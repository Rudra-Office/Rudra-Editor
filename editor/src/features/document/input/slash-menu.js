// Slash menu close helper.
// Extracted from input.js to break circular dependency.
import { state, $ } from '../../../state.js';

export function closeSlashMenu() {
  state.slashMenuOpen = false;
  state.slashQuery = '';
  state.slashMenuIndex = 0;
  const menu = $('slashMenu');
  if (menu) menu.style.display = 'none';
}
