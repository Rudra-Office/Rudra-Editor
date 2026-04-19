import puppeteer from 'puppeteer';
import fs from 'fs';

const FILES = [
  { path: '/Users/sachin/Downloads/Aruljothi.docx', name: 'Aruljothi' },
  { path: '/Users/sachin/Downloads/Chat Reaction.docx', name: 'ChatReaction' },
  { path: '/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx', name: 'SDS_ZH' },
];

async function main() {
  const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });

  for (const file of FILES) {
    const page = await browser.newPage();
    await page.goto('http://localhost:8080', { waitUntil: 'networkidle0', timeout: 30000 });
    await new Promise(r => setTimeout(r, 3000));
    const b64 = fs.readFileSync(file.path).toString('base64');

    const r = await page.evaluate(async (d) => {
      const b = atob(d), a = new Uint8Array(b.length);
      for (let i = 0; i < b.length; i++) a[i] = b.charCodeAt(i);
      const { openDocx } = await import('./adapter.js');
      const api = window._api || window.editor;
      await openDocx(a, api);
      const ld = api.WordControl.m_oLogicDocument;
      if (!ld) return null;

      // Count formatting features
      let boldRuns = 0, italicRuns = 0, colorRuns = 0, fontRuns = 0;
      let totalRuns = 0, totalText = 0;
      let tabCount = 0, listParas = 0, styledParas = 0;
      let imgCount = 0, hdrContent = 0, ftrContent = 0;

      for (let i = 0; i < ld.Content.length; i++) {
        const el = ld.Content[i];
        if (!el.Content) continue;
        // Check paragraph style
        if (el.Pr && el.Pr.PStyle) styledParas++;
        if (el.Pr && el.Pr.NumPr && el.Pr.NumPr.NumId !== undefined) listParas++;

        for (let j = 0; j < el.Content.length; j++) {
          const run = el.Content[j];
          if (run && run.Content) {
            for (let k = 0; k < run.Content.length; k++) {
              const item = run.Content[k];
              if (item && item.constructor && item.constructor.name === 'CRunText') totalText++;
              if (item && item.constructor && item.constructor.name === 'CRunTab') tabCount++;
            }
          }
          if (run && run.Pr) {
            totalRuns++;
            if (run.Pr.Bold) boldRuns++;
            if (run.Pr.Italic) italicRuns++;
            if (run.Pr.Color && !run.Pr.Color.Auto) colorRuns++;
            if (run.Pr.RFonts && run.Pr.RFonts.Ascii) fontRuns++;
          }
        }
      }

      // Headers/footers content
      const sp = ld.SectPr;
      if (sp) {
        if (sp.HeaderDefault && sp.HeaderDefault.Content) hdrContent = sp.HeaderDefault.Content.Content.length;
        if (sp.FooterDefault && sp.FooterDefault.Content) ftrContent = sp.FooterDefault.Content.Content.length;
      }

      return {
        elements: ld.Content.length,
        pages: ld.Pages ? ld.Pages.length : 0,
        totalRuns, totalText, boldRuns, italicRuns, colorRuns, fontRuns,
        tabCount, listParas, styledParas, imgCount,
        hdrContent, ftrContent,
        tables: ld.Content.filter(e => e.constructor.name.includes('Table')).length,
      };
    }, b64);

    if (r) {
      console.log(`\n=== ${file.name} ===`);
      console.log(`  Elements: ${r.elements} (${r.tables} tables)`);
      console.log(`  Pages: ${r.pages}`);
      console.log(`  Text chars: ${r.totalText}, Runs: ${r.totalRuns}`);
      console.log(`  Bold: ${r.boldRuns}, Italic: ${r.italicRuns}, Color: ${r.colorRuns}, Font: ${r.fontRuns}`);
      console.log(`  Tabs: ${r.tabCount}, Lists: ${r.listParas}, Styled: ${r.styledParas}`);
      console.log(`  Header content: ${r.hdrContent}, Footer content: ${r.ftrContent}`);
    }
    await page.close();
  }
  await browser.close();
}
main().catch(e => { console.error(e); process.exit(1); });
