// Table Formulas — M14.3
//
// Simple formula evaluation for table cells, matching Word-style syntax:
//   =SUM(ABOVE)    — sum all numeric cells above current cell in same column
//   =SUM(LEFT)     — sum all numeric cells to the left in same row
//   =SUM(BELOW)    — sum all numeric cells below in same column
//   =AVERAGE(ABOVE) — average of cells above
//   =COUNT(ABOVE)   — count of numeric cells above
//   =MIN(ABOVE)     — minimum
//   =MAX(ABOVE)     — maximum
//   =PRODUCT(ABOVE) — product of cells above

import { state } from './state.js';

/**
 * Evaluate a table formula in the context of a specific cell.
 *
 * @param {string} formula - e.g., "=SUM(ABOVE)"
 * @param {string} tableId - node ID of the table
 * @param {number} row - 0-based row index of the cell
 * @param {number} col - 0-based column index of the cell
 * @returns {string|null} - evaluated result as string, or null if invalid
 */
export function evaluateTableFormula(formula, tableId, row, col) {
  if (!formula || !formula.startsWith('=') || !state.doc) return null;

  const expr = formula.slice(1).trim().toUpperCase();
  const match = expr.match(/^(SUM|AVERAGE|AVG|COUNT|MIN|MAX|PRODUCT)\s*\(\s*(ABOVE|BELOW|LEFT|RIGHT)\s*\)$/);
  if (!match) return null;

  const func = match[1] === 'AVG' ? 'AVERAGE' : match[1];
  const direction = match[2];

  // Get table dimensions
  let dims;
  try {
    dims = JSON.parse(state.doc.get_table_dimensions(tableId));
  } catch (_) {
    return null;
  }

  const numRows = dims.rows || 0;
  const numCols = dims.cols || 0;

  // Collect values from the specified direction
  const values = [];

  if (direction === 'ABOVE') {
    for (let r = 0; r < row; r++) {
      const val = getCellNumericValue(tableId, r, col);
      if (val !== null) values.push(val);
    }
  } else if (direction === 'BELOW') {
    for (let r = row + 1; r < numRows; r++) {
      const val = getCellNumericValue(tableId, r, col);
      if (val !== null) values.push(val);
    }
  } else if (direction === 'LEFT') {
    for (let c = 0; c < col; c++) {
      const val = getCellNumericValue(tableId, row, c);
      if (val !== null) values.push(val);
    }
  } else if (direction === 'RIGHT') {
    for (let c = col + 1; c < numCols; c++) {
      const val = getCellNumericValue(tableId, row, c);
      if (val !== null) values.push(val);
    }
  }

  if (values.length === 0 && func !== 'COUNT') return '0';

  // Apply function
  switch (func) {
    case 'SUM':
      return formatNumber(values.reduce((a, b) => a + b, 0));
    case 'AVERAGE':
      return values.length > 0 ? formatNumber(values.reduce((a, b) => a + b, 0) / values.length) : '0';
    case 'COUNT':
      return String(values.length);
    case 'MIN':
      return formatNumber(Math.min(...values));
    case 'MAX':
      return formatNumber(Math.max(...values));
    case 'PRODUCT':
      return formatNumber(values.reduce((a, b) => a * b, 1));
    default:
      return null;
  }
}

/**
 * Get the numeric value of a table cell, or null if not numeric.
 */
function getCellNumericValue(tableId, row, col) {
  try {
    const cellId = state.doc.get_cell_id(tableId, row, col);
    if (!cellId) return null;
    const text = state.doc.get_cell_text(cellId).trim();
    // Strip currency symbols, commas, percentage
    const cleaned = text.replace(/[$€£¥,]/g, '').replace(/%$/, '');
    const num = parseFloat(cleaned);
    return isNaN(num) ? null : num;
  } catch (_) {
    return null;
  }
}

function formatNumber(n) {
  if (Number.isInteger(n)) return String(n);
  // Round to 2 decimal places
  return String(Math.round(n * 100) / 100);
}

/**
 * Check if text looks like a formula.
 */
export function isFormula(text) {
  return typeof text === 'string' && text.startsWith('=') && /^=\w+\(/i.test(text);
}
