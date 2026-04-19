import puppeteer from 'puppeteer';
import fs from 'fs';
const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
const fontRequests = [];
page.on('request', req => { if (req.url().includes('.ttf') || req.url().includes('font')) fontRequests.push(req.url()); });
page.on('requestfailed', req => { if (req.url().includes('.ttf')) console.log('FAILED:', req.url(), req.failure().errorText); });
await page.goto('http://localhost:8080', { waitUntil: 'networkidle0', timeout: 30000 });
await new Promise(r => setTimeout(r, 3000));

// Open a file to trigger font loading
const b64 = fs.readFileSync('/Users/sachin/Downloads/Aruljothi.docx').toString('base64');
await page.evaluate(async (d) => {
  const b = atob(d), a = new Uint8Array(b.length);
  for (let i = 0; i < b.length; i++) a[i] = b.charCodeAt(i);
  const { openDocx } = await import('./adapter.js');
  const api = window._api || window.editor;
  await openDocx(a, api);
}, b64);
await new Promise(r => setTimeout(r, 3000));

console.log('Font network requests:', fontRequests.length);
for (const r of fontRequests.slice(0, 10)) console.log(' ', r);

// Check if any font data is in the engine
const state = await page.evaluate(() => {
  const loader = AscCommon.g_font_loader;
  return {
    fontsLoadedCount: loader.fonts_loaded ? loader.fonts_loaded.length : -1,
    fontsLoadingCount: loader.fonts_loading ? loader.fonts_loading.length : -1,
    nLoadedFonts: loader.nLoadedFonts || 0,
    bLoadRecursion: loader.bLoadRecursion || false,
  };
});
console.log('Loader state:', JSON.stringify(state));

// Take screenshot
await page.screenshot({ path: '/tmp/docy_render.png', fullPage: false });
console.log('Screenshot saved to /tmp/docy_render.png');

await browser.close();
