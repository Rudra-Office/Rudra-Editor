import puppeteer from 'puppeteer';
import fs from 'fs';

const FILES = [
  { path: '/Users/sachin/Downloads/Aruljothi.docx', name: 'aruljothi' },
  { path: '/Users/sachin/Downloads/Chat Reaction.docx', name: 'chat_reaction' },
  { path: '/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx', name: 'sds_zh' },
];

async function main() {
  const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
  
  for (const file of FILES) {
    const page = await browser.newPage();
    await page.setViewport({ width: 1280, height: 900 });
    await page.goto('http://localhost:8080', { waitUntil: 'networkidle0', timeout: 30000 });
    await new Promise(r => setTimeout(r, 3000));
    
    const b64 = fs.readFileSync(file.path).toString('base64');
    await page.evaluate(async (d) => {
      const b = atob(d), a = new Uint8Array(b.length);
      for (let i = 0; i < b.length; i++) a[i] = b.charCodeAt(i);
      const { openDocx } = await import('./adapter.js');
      const api = window._api || window.editor;
      await openDocx(a, api);
    }, b64);
    
    // Wait for fonts and rendering
    await new Promise(r => setTimeout(r, 3000));
    
    const shot = `/tmp/docy_${file.name}.png`;
    await page.screenshot({ path: shot });
    console.log(`${file.name}: screenshot saved to ${shot}`);
    await page.close();
  }
  
  await browser.close();
}
main().catch(e => { console.error(e); process.exit(1); });
