// @ts-check
import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, '..', '..');
const COMPLEX_DOCX_PATH = path.join(REPO_ROOT, 'complex.docx');
const OUTPUT_DIR = path.join(REPO_ROOT, 'fidelity-results');

async function waitForEngine(page) {
  await page.waitForFunction(() => {
    const label = document.getElementById('wasmLabel');
    return label && (label.textContent === 's1engine ready' || label.textContent.includes('ready'));
  }, { timeout: 60000 });
}

async function openDocx(page, filePath, canvasMode) {
  console.log(`Opening DOCX (${canvasMode ? 'Canvas' : 'DOM'}):`, filePath);
  
  await page.goto('/');
  await page.evaluate((mode) => {
    localStorage.setItem('s1-canvas-mode', mode ? '1' : '0');
  }, canvasMode);
  await page.reload();
  
  await waitForEngine(page);
  
  const fileInput = page.locator('#fileInput');
  await page.evaluate(() => {
    const input = document.getElementById('fileInput');
    if (input) {
      input.style.display = 'block';
      input.style.opacity = '1';
      input.style.visibility = 'visible';
    }
  });
  
  await fileInput.setInputFiles(filePath);
  
  await page.waitForFunction((mode) => {
    const state = window.__s1_state;
    const selector = mode ? '#pageContainer .s1-canvas-page' : '#pageContainer .doc-page';
    const pages = document.querySelectorAll(selector);
    return !!state?.doc && pages.length > 0;
  }, canvasMode, { timeout: 60000 });
  
  await page.waitForFunction(() => !document.querySelector('.loading-overlay'), { timeout: 60000 });
  await page.waitForTimeout(5000); 

  const stats = await page.evaluate((mode) => {
    const selector = mode ? '#pageContainer .s1-canvas-page' : '#pageContainer .doc-page';
    const pages = document.querySelectorAll(selector);
    const nodes = document.querySelectorAll('[data-node-id]');
    return {
      pageCount: pages.length,
      nodeCount: nodes.length
    };
  }, canvasMode);
  
  return stats;
}

test.beforeAll(async () => {
  if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  }
});

test('capture screenshots and stats of complex.docx', async ({ page }) => {
  test.setTimeout(300000); 
  await page.setViewportSize({ width: 1280, height: 4000 });

  const results = {};

  // 1. DOM Mode
  results.dom = await openDocx(page, COMPLEX_DOCX_PATH, false);
  await page.screenshot({ path: path.join(OUTPUT_DIR, 'complex-dom.png'), fullPage: true });

  // 2. Canvas Mode
  results.canvas = await openDocx(page, COMPLEX_DOCX_PATH, true);
  await page.screenshot({ path: path.join(OUTPUT_DIR, 'complex-canvas.png'), fullPage: true });
  
  console.log('Fidelity Comparison Results:');
  console.log('DOM Mode:', results.dom);
  console.log('Canvas Mode:', results.canvas);
  
  if (results.dom.pageCount !== results.canvas.pageCount) {
    console.warn(`WARNING: Page count mismatch! DOM: ${results.dom.pageCount}, Canvas: ${results.canvas.pageCount}`);
  }
  
  console.log(`Screenshots saved to ${OUTPUT_DIR}`);
});
