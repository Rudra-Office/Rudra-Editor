import puppeteer from 'puppeteer';
import fs from 'fs';
const browser = await puppeteer.launch({ headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage();
const logs = [];
page.on('console', msg => { const t = msg.text(); if (t.includes('R1ERR') || t.includes('R2ERR')) logs.push(t); });
await page.goto('http://localhost:8080', { waitUntil: 'networkidle0', timeout: 30000 });
await new Promise(r => setTimeout(r, 3000));
const b64 = fs.readFileSync('/Users/sachin/Downloads/SDS_ANTI-T..._ZH.docx').toString('base64');
await page.evaluate(async (d) => {
  const b = atob(d), a = new Uint8Array(b.length);
  for (let i = 0; i < b.length; i++) a[i] = b.charCodeAt(i);
  // Add Read2 error logging
  var _r2 = AscCommon.Binary_CommonReader.prototype.Read2;
  AscCommon.Binary_CommonReader.prototype.Read2 = function(l, f) {
    var c = this.stream.cur; var r = _r2.call(this, l, f);
    if (r === -1) console.warn('[R2ERR] len=' + l + ' at=' + c + ' cur=' + this.stream.cur +
      ' byte=0x' + (this.stream.data[this.stream.cur]||0).toString(16) +
      ' prev=0x' + (this.stream.data[this.stream.cur-1]||0).toString(16));
    return r;
  };
  const { openDocx } = await import('./adapter.js');
  const api = window._api || window.editor;
  try { await openDocx(a, api); } catch(e) {}
  AscCommon.Binary_CommonReader.prototype.Read2 = _r2;
}, b64);
await new Promise(r => setTimeout(r, 1000));
console.log('Errors:', logs.length);
for (const l of logs) console.log(' ', l);
await browser.close();
