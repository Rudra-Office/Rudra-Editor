/**
 * End-to-end DOCY browser test using Puppeteer.
 * Opens real DOCX files through the actual sdkjs pipeline and checks results.
 */
import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';

const BASE_URL = 'http://localhost:8080';
const TIMEOUT = 30000;

const FILES = [
  { path: '/Users/sachin/Downloads/Aruljothi.docx', name: 'Aruljothi', minElements: 50 },
  { path: '/Users/sachin/Downloads/Nishtriya.docx', name: 'Nishtriya', minElements: 30 },
  { path: '/Users/sachin/Downloads/Chat Reaction.docx', name: 'ChatReaction', minElements: 50 },
  { path: '/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx', name: 'SDS_ZH', minElements: 30 },
];

async function testFile(browser, file) {
  const page = await browser.newPage();
  const errors = [];
  const logs = [];

  page.on('console', msg => logs.push(`[${msg.type()}] ${msg.text()}`));
  page.on('pageerror', err => errors.push(err.message));

  try {
    await page.goto(BASE_URL, { waitUntil: 'networkidle0', timeout: TIMEOUT });

    // Wait for WASM to be ready
    await page.waitForFunction(() => window._wasmReady === true || document.querySelector('#editor_sdk'), { timeout: TIMEOUT });
    await new Promise(r => setTimeout(r, 2000)); // Wait for sdkjs init

    // Read file and trigger open
    const fileData = fs.readFileSync(file.path);
    const base64 = fileData.toString('base64');

    const result = await page.evaluate(async (b64, fileName) => {
      try {
        // Convert base64 to Uint8Array
        const binary = atob(b64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);

        // Call the adapter
        const { openDocx } = await import('./adapter.js');
        const api = window._api || window.editor;
        if (!api) return { error: 'No API found' };

        const doc = await openDocx(bytes, api);
        const logicDoc = api.WordControl && api.WordControl.m_oLogicDocument;
        const elements = logicDoc ? logicDoc.Content.length : 0;

        return { elements, error: null };
      } catch (e) {
        return { elements: 0, error: e.message };
      }
    }, base64, file.name);

    // Check console for DOCY loaded message
    const docyLog = logs.find(l => l.includes('DOCY loaded:'));
    const elemMatch = docyLog && docyLog.match(/(\d+) elements/);
    const reportedElements = elemMatch ? parseInt(elemMatch[1]) : (result.elements || 0);

    const jsErrors = errors.filter(e => !e.includes('non-fatal'));
    const hasFatalError = jsErrors.length > 0;

    return {
      name: file.name,
      elements: reportedElements,
      minExpected: file.minElements,
      pass: reportedElements >= file.minElements && !hasFatalError,
      error: result.error || (hasFatalError ? jsErrors[0] : null),
      logs: logs.filter(l => l.includes('[adapter]') || l.includes('[DOCY')).slice(0, 10),
    };
  } catch (e) {
    return { name: file.name, elements: 0, pass: false, error: e.message, logs };
  } finally {
    await page.close();
  }
}

async function main() {
  console.log('Starting E2E DOCY tests...\n');

  const browser = await puppeteer.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });

  let allPass = true;
  const results = [];

  for (const file of FILES) {
    if (!fs.existsSync(file.path)) {
      console.log(`SKIP ${file.name} — file not found`);
      continue;
    }
    const result = await testFile(browser, file);
    results.push(result);
    const status = result.pass ? 'PASS' : 'FAIL';
    console.log(`${status} ${result.name}: ${result.elements} elements (min: ${result.minExpected})${result.error ? ' ERROR: ' + result.error : ''}`);
    for (const log of (result.logs || [])) {
      console.log(`  ${log}`);
    }
    if (!result.pass) allPass = false;
  }

  console.log(`\n${results.filter(r => r.pass).length}/${results.length} passed`);

  await browser.close();
  process.exit(allPass ? 0 : 1);
}

main().catch(e => { console.error(e); process.exit(1); });
