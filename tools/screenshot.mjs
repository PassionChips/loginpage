// Takes a screenshot of an official portal page for a guide, using headless Chrome.
//
//   node tools/screenshot.mjs URL OUT [width] [height] [wait-ms]
//
// Writes OUT.<FMT> (default png; use FMT=webp for guides) and OUT.txt (the page text and links, handy for
// checking a guide against the source; set TXT=/some/other/base to write it elsewhere).
// Options, as environment variables:
//   HL="Link text"        outline the element with exactly this text in red (and scroll it into view)
//   CROP="hl:W,H"         crop a W x H box centred on the outlined element; CROP="X,Y,W,H" for a fixed box
//   CLICK="A|B#2"         click elements by exact text, in order (#2 = the second match)
//   EVAL="js"             run one expression first; STEPS='["js1","js2"]' runs several, 4 s apart
// Only capture public, pre-login screens, and only from portals whose copyright policy allows reuse with
// credit (see README "Guide screenshots").
import { spawn } from 'node:child_process';
import { writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
const [,, url, out, w = '1280', h = '800', wait = '6000'] = process.argv;
const port = 9400 + Math.floor(Math.random() * 400);
const dir = mkdtempSync(join(tmpdir(), 'guide-shot-'));
const chrome = spawn('google-chrome', ['--headless=new','--no-sandbox','--disable-gpu','--hide-scrollbars',`--remote-debugging-port=${port}`,'--user-data-dir=' + dir,'--lang=en-IN','about:blank'], {stdio:'ignore'});
await new Promise(r => setTimeout(r, 2000));
const tabs = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
const ws = new WebSocket(tabs.find(t => t.type === 'page').webSocketDebuggerUrl);
await new Promise(r => ws.onopen = r);
let id = 0; const pending = new Map();
ws.onmessage = e => { const m = JSON.parse(e.data); if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); } };
const send = (method, params={}) => new Promise(r => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({id:i, method, params})); });
const ev = async (expr) => (await send('Runtime.evaluate', {expression: expr, returnByValue: true})).result.result?.value;
await send('Emulation.setDeviceMetricsOverride', {width:+w, height:+h, deviceScaleFactor:1, mobile:+w<500});
await send('Network.enable');
await send('Network.setUserAgentOverride', {userAgent: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36'});
await send('Page.navigate', {url});
await new Promise(r => setTimeout(r, +wait));
if (process.env.EVAL) { console.error('eval', await ev(process.env.EVAL)); await new Promise(r => setTimeout(r, 3000)); }
for (const label of (process.env.CLICK || '').split('|').filter(Boolean)) {
  // Click the first visible element whose text is exactly `label`, then let the page settle.
  const [text, nth = '1'] = label.split('#');
  const r = await ev(`(() => { const want = ${JSON.stringify(text)}.toLowerCase();
    const all = [...document.querySelectorAll('a,button,[role=button],span,div,label')].filter(e => (e.innerText||'').trim().toLowerCase() === want && e.offsetWidth && !e.querySelector('a,button,label'));
    const el = all[${+nth - 1}];
    if (!el) return 'none'; el.click(); return 'ok'; })()`);
  console.error('click', label, r);
  await new Promise(r => setTimeout(r, 4000));
}
// STEPS: JSON list of JS expressions run one after another, 4 s apart (for multi-page flows).
for (const step of JSON.parse(process.env.STEPS || '[]')) { console.error('step', await ev(step)); await new Promise(r => setTimeout(r, 4000)); }
if (process.env.HL) {
  // Outline the element whose visible text is exactly HL, and scroll it into the middle of the screen.
  await ev(`(() => { const want = ${JSON.stringify(process.env.HL)}.toLowerCase();
    const els = [...document.querySelectorAll('a,button,span,div,p,li,label,h1,h2,h3,h4,input')].filter(e => (e.innerText||e.value||'').trim().toLowerCase() === want && e.offsetWidth);
    const el = els.sort((a,b) => a.offsetWidth*a.offsetHeight - b.offsetWidth*b.offsetHeight)[0];
    if (!el) return 'none'; el.scrollIntoView({block:'center'}); el.style.outline='4px solid #e11d48'; el.style.outlineOffset='4px'; el.style.borderRadius='6px'; return 'ok'; })()`);
  await new Promise(r => setTimeout(r, 1200));
}
const txt = await ev(`document.title + "\\n" + location.href + "\\n\\n" + document.body.innerText + "\\n\\nLINKS:\\n" + [...document.querySelectorAll('a[href]')].map(a => (a.innerText.trim().replace(/\\s+/g,' ') || a.title) + ' -> ' + a.href).join('\\n')`);
writeFileSync((process.env.TXT || out) + '.txt', txt || '');
const fmt = process.env.FMT || 'png';
// CROP="hl:W,H" crops a W x H box centred on the outlined element; CROP="X,Y,W,H" crops that box.
let clip;
if (process.env.CROP) {
  let [x, y, cw, ch] = process.env.CROP.replace('hl:', '').split(',').map(Number);
  if (process.env.CROP.startsWith('hl:')) {
    cw = x; ch = y;
    const r = await ev(`(() => { const e = [...document.querySelectorAll('*')].find(e => e.style.outline.includes('4px')); if (!e) return null; const b = e.getBoundingClientRect(); return [b.x + b.width/2, b.y + b.height/2]; })()`);
    const [cx, cy] = r || [+w/2, +h/2];
    x = Math.max(0, Math.min(+w - cw, cx - cw/2)); y = Math.max(0, Math.min(+h - ch, cy - ch/2));
  }
  const sy = await ev('window.scrollY'); clip = {x, y: y + sy, width: cw, height: ch, scale: 1};
}
const shot = await send('Page.captureScreenshot', {format: fmt, quality: fmt === 'png' ? undefined : 82, clip}); writeFileSync(out + '.' + fmt, Buffer.from(shot.result.data, 'base64'));
ws.close(); chrome.kill(); setTimeout(() => { try { rmSync(dir, {recursive:true, force:true}); } catch {} process.exit(0); }, 500);
