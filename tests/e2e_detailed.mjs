import puppeteer from 'puppeteer';
import fs from 'fs';

const BASE_URL = 'http://localhost:8080';

const FILES = [
  '/Users/sachin/Downloads/Aruljothi.docx',
  '/Users/sachin/Downloads/Nishtriya.docx', 
  '/Users/sachin/Downloads/Chat Reaction.docx',
  '/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx',
];

async function testFile(browser, filePath) {
  const page = await browser.newPage();
  const consoleLines = [];
  page.on('console', msg => consoleLines.push(msg.text()));
  page.on('pageerror', err => consoleLines.push('PAGE_ERROR: ' + err.message));

  await page.goto(BASE_URL, { waitUntil: 'networkidle0', timeout: 30000 });
  await new Promise(r => setTimeout(r, 3000));

  const fileData = fs.readFileSync(filePath);
  const b64 = fileData.toString('base64');
  const name = filePath.split('/').pop();

  const result = await page.evaluate(async (b64data, fname) => {
    const binary = atob(b64data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);

    const { openDocx } = await import('./adapter.js');
    const api = window._api || window.editor;
    if (!api) return { error: 'No API' };

    try {
      await openDocx(bytes, api);
    } catch(e) {}

    const ld = api.WordControl.m_oLogicDocument;
    if (!ld) return { error: 'No logicDocument' };

    // Count content types
    let paras = 0, tables = 0, sdts = 0, others = 0;
    for (let i = 0; i < ld.Content.length; i++) {
      const el = ld.Content[i];
      const type = el.constructor.name || el.GetType?.() || '?';
      if (type.includes('Paragraph') || el.GetType?.() === 0) paras++;
      else if (type.includes('Table') || el.GetType?.() === 2) tables++;
      else if (type.includes('Sdt')) sdts++;
      else others++;
    }

    // Check headers/footers
    const sectPr = ld.SectPr;
    const hasHdrDefault = !!(sectPr && sectPr.HeaderDefault);
    const hasFtrDefault = !!(sectPr && sectPr.FooterDefault);
    const hasHdrFirst = !!(sectPr && sectPr.HeaderFirst);
    const hasFtrFirst = !!(sectPr && sectPr.FooterFirst);

    // Check numbering
    const numCount = ld.Numbering ? Object.keys(ld.Numbering.Num || {}).length : 0;

    // Check styles
    const styleCount = ld.Styles ? Object.keys(ld.Styles.Style || {}).length : 0;

    // Check for render errors
    let canRender = true;
    try {
      ld.Recalculate();
    } catch(e) {
      canRender = false;
    }

    // Page count
    const pageCount = ld.Pages ? ld.Pages.length : 0;

    return {
      elements: ld.Content.length,
      paras, tables, sdts, others,
      hasHdrDefault, hasFtrDefault, hasHdrFirst, hasFtrFirst,
      numCount, styleCount, pageCount, canRender,
      error: null
    };
  }, b64, name);

  const errors = consoleLines.filter(l => l.includes('ERROR') || l.includes('error') || l.includes('FAIL'));
  
  await page.close();
  return { name, ...result, consoleErrors: errors.length };
}

async function main() {
  const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
  
  for (const f of FILES) {
    if (!fs.existsSync(f)) { console.log(`SKIP ${f}`); continue; }
    const r = await testFile(browser, f);
    console.log(`\n=== ${r.name} ===`);
    console.log(`  Elements: ${r.elements} (${r.paras} paras, ${r.tables} tables, ${r.sdts} sdts)`);
    console.log(`  Headers: default=${r.hasHdrDefault} first=${r.hasHdrFirst}`);
    console.log(`  Footers: default=${r.hasFtrDefault} first=${r.hasFtrFirst}`);
    console.log(`  Styles: ${r.styleCount}, Numbering: ${r.numCount}`);
    console.log(`  Pages: ${r.pageCount}, CanRender: ${r.canRender}`);
    console.log(`  Console errors: ${r.consoleErrors}`);
    if (r.error) console.log(`  ERROR: ${r.error}`);
  }

  await browser.close();
}

main().catch(e => { console.error(e); process.exit(1); });
