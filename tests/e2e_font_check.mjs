import puppeteer from 'puppeteer';
const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
const logs = [];
page.on('console', msg => logs.push(msg.text()));
page.on('pageerror', err => logs.push('ERR: ' + err.message));
await page.goto('http://localhost:8080', { waitUntil: 'networkidle0', timeout: 30000 });
await new Promise(r => setTimeout(r, 5000));

// Check font state
const r = await page.evaluate(() => {
  const fl = AscCommon.g_font_loader;
  const fa = AscFonts.g_fontApplication;
  return {
    fontsLoading: fl ? fl.fonts_loading.length : -1,
    fontPickerMapKeys: fa ? Object.keys(fa.FontPickerMap).length : -1,
    mapFontIndexKeys: AscFonts.g_map_font_index ? Object.keys(AscFonts.g_map_font_index).length : -1,
    fontInfosLen: AscFonts.g_font_infos ? AscFonts.g_font_infos.length : -1,
    fontsFilesLen: window.__fonts_files ? window.__fonts_files.length : -1,
    // Try to load Arial and check
    testFont: (() => {
      try {
        var r = fa.GetFontFileWeb("Arial", 0);
        return r ? { name: r.m_wsFontName, idx: r.m_lIndex, path: r.m_wsFontPath } : null;
      } catch(e) { return 'error: ' + e.message; }
    })(),
  };
});

console.log('Font state:', JSON.stringify(r, null, 2));
const fontLogs = logs.filter(l => l.toLowerCase().includes('font') || l.includes('ERR'));
if (fontLogs.length) {
  console.log('\nFont-related logs:');
  for (const l of fontLogs.slice(0, 10)) console.log(' ', l);
}
await browser.close();
