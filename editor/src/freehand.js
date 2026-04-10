// Freehand Drawing Tool — M15.6
//
// Pen, highlighter, and eraser tools for drawing on document pages.
// Drawings are rendered on a transparent canvas overlay per page.

import { state, $ } from './state.js';

let _tool = null; // null | 'pen' | 'highlighter' | 'eraser'
let _color = '#000000';
let _lineWidth = 2;
let _drawing = false;
let _drawCanvas = null;
let _drawCtx = null;
let _lastPoint = null;
let _strokes = new Map(); // pageIndex → [{points, color, width, tool}]

export function initFreehand() {
  // Drawing tools are activated from Insert > Drawing menu or a toolbar button
  // This module provides the drawing capability when activated
}

export function startFreehandMode(tool) {
  _tool = tool || 'pen';
  _color = tool === 'highlighter' ? 'rgba(255, 255, 0, 0.4)' : '#000000';
  _lineWidth = tool === 'highlighter' ? 12 : tool === 'eraser' ? 20 : 2;

  const container = $('pageContainer');
  if (!container) return;

  container.style.cursor = tool === 'eraser' ? 'crosshair' : 'default';
  container.classList.add('freehand-active');

  // Create overlay canvases on each page
  container.querySelectorAll('.doc-page').forEach((page, i) => {
    if (page.querySelector('.freehand-canvas')) return;
    const canvas = document.createElement('canvas');
    canvas.className = 'freehand-canvas';
    canvas.style.cssText = 'position:absolute;inset:0;z-index:50;pointer-events:auto;cursor:crosshair;';
    canvas.width = page.offsetWidth;
    canvas.height = page.offsetHeight;
    canvas.dataset.pageIndex = i;

    const ctx = canvas.getContext('2d');
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';

    // Replay existing strokes
    const pageStrokes = _strokes.get(i) || [];
    for (const stroke of pageStrokes) {
      drawStroke(ctx, stroke);
    }

    canvas.addEventListener('pointerdown', onPointerDown);
    canvas.addEventListener('pointermove', onPointerMove);
    canvas.addEventListener('pointerup', onPointerUp);
    canvas.addEventListener('pointerleave', onPointerUp);

    page.appendChild(canvas);
  });
}

export function stopFreehandMode() {
  _tool = null;
  _drawing = false;
  const container = $('pageContainer');
  if (container) {
    container.style.cursor = '';
    container.classList.remove('freehand-active');
    container.querySelectorAll('.freehand-canvas').forEach(c => {
      c.style.pointerEvents = 'none';
      c.style.cursor = 'default';
    });
  }
}

export function clearFreehandDrawings() {
  _strokes.clear();
  document.querySelectorAll('.freehand-canvas').forEach(c => {
    const ctx = c.getContext('2d');
    ctx.clearRect(0, 0, c.width, c.height);
  });
}

export function isFreehandActive() {
  return _tool !== null;
}

function onPointerDown(e) {
  if (!_tool) return;
  _drawing = true;
  _drawCanvas = e.target;
  _drawCtx = _drawCanvas.getContext('2d');
  const rect = _drawCanvas.getBoundingClientRect();
  _lastPoint = { x: e.clientX - rect.left, y: e.clientY - rect.top };

  _drawCtx.beginPath();
  _drawCtx.moveTo(_lastPoint.x, _lastPoint.y);

  if (_tool === 'eraser') {
    _drawCtx.globalCompositeOperation = 'destination-out';
  } else {
    _drawCtx.globalCompositeOperation = 'source-over';
  }
  _drawCtx.strokeStyle = _color;
  _drawCtx.lineWidth = _lineWidth;

  // Start recording stroke
  const pageIdx = parseInt(_drawCanvas.dataset.pageIndex || '0');
  if (!_strokes.has(pageIdx)) _strokes.set(pageIdx, []);
  _strokes.get(pageIdx).push({
    points: [_lastPoint],
    color: _color,
    width: _lineWidth,
    tool: _tool,
  });

  e.preventDefault();
}

function onPointerMove(e) {
  if (!_drawing || !_drawCtx) return;
  const rect = _drawCanvas.getBoundingClientRect();
  const point = { x: e.clientX - rect.left, y: e.clientY - rect.top };

  _drawCtx.lineTo(point.x, point.y);
  _drawCtx.stroke();
  _drawCtx.beginPath();
  _drawCtx.moveTo(point.x, point.y);

  // Record point
  const pageIdx = parseInt(_drawCanvas.dataset.pageIndex || '0');
  const strokes = _strokes.get(pageIdx);
  if (strokes && strokes.length > 0) {
    strokes[strokes.length - 1].points.push(point);
  }

  _lastPoint = point;
  e.preventDefault();
}

function onPointerUp(e) {
  if (!_drawing) return;
  _drawing = false;
  if (_drawCtx) {
    _drawCtx.globalCompositeOperation = 'source-over';
  }
  _drawCtx = null;
  _drawCanvas = null;
  _lastPoint = null;
}

function drawStroke(ctx, stroke) {
  if (stroke.points.length < 2) return;
  ctx.save();
  ctx.globalCompositeOperation = stroke.tool === 'eraser' ? 'destination-out' : 'source-over';
  ctx.strokeStyle = stroke.color;
  ctx.lineWidth = stroke.width;
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  ctx.beginPath();
  ctx.moveTo(stroke.points[0].x, stroke.points[0].y);
  for (let i = 1; i < stroke.points.length; i++) {
    ctx.lineTo(stroke.points[i].x, stroke.points[i].y);
  }
  ctx.stroke();
  ctx.restore();
}
