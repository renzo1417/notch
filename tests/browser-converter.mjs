// Headless UI checks with a synthetic Tauri bridge. Native process tests are separate.
import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import assert from "node:assert/strict";

const output = path.resolve(process.argv[2] ?? "src-tauri/target/ui-check");
await mkdir(output, { recursive: true });
const server = spawn(process.execPath, ["node_modules/vite/bin/vite.js", "--host", "127.0.0.1", "--port", "1421"], { windowsHide: true, stdio: "ignore" });
const browser = spawn("C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", ["--headless=new", "--remote-debugging-port=9335", "--no-first-run", "--no-default-browser-check", `--user-data-dir=${path.resolve("src-tauri/target/ui-browser")}`, "about:blank"], { windowsHide: true, stdio: "ignore" });
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
let ws, next = 0;
const pending = new Map();
const errors = [];
async function command(method, params = {}) {
  const id = ++next;
  return new Promise((resolve, reject) => { pending.set(id, { resolve, reject }); ws.send(JSON.stringify({ id, method, params })); });
}
async function evaluate(expression) {
  const result = await command("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  if (result.exceptionDetails) throw new Error(result.exceptionDetails.text + ": " + result.exceptionDetails.exception?.description);
  return result.result.value;
}
async function until(expression, label) {
  const end = Date.now() + 10000;
  while (Date.now() < end) { if (await evaluate(expression)) return; await delay(50); }
  throw new Error(`Timed out: ${label}\n${await evaluate("document.body.innerText")}`);
}
async function click(label) {
  await until(`[...document.querySelectorAll('button')].some(b => b.getClientRects().length && !b.disabled && (b.getAttribute('aria-label') === ${JSON.stringify(label)} || b.textContent.trim() === ${JSON.stringify(label)}))`, `button ${label}`);
  await until(`(() => {const b=[...document.querySelectorAll('button')].find(b => b.getClientRects().length && !b.disabled && (b.getAttribute('aria-label') === ${JSON.stringify(label)} || b.textContent.trim() === ${JSON.stringify(label)}));if(!b)return false;b.click();return true;})()`, `click ${label}`);
}
async function screenshot(name) {
  const image = await command("Page.captureScreenshot", { format: "png" });
  await writeFile(path.join(output, name + ".png"), Buffer.from(image.data, "base64"));
}
const mock = `(() => {
  const callbacks = new Map(), listeners = new Map(); let id = 0;
  window.mockAvailable = true;
  window.mockListenerCount = () => [...listeners.values()].reduce((count, items) => count + items.length, 0);
  window.mockEmit = (name, payload) => { for (const handler of listeners.get(name) ?? []) callbacks.get(handler)?.({event:name,id:0,payload}); };
  let snapshot = {jobId:'',running:false,files:[],target:'',message:''}, finish;
  const file = p => ({path:p,name:p.split(/\\\\/).pop(),extension:'png',bytes:245760,state:'pending',progress:null,outputs:[],error:null});
  window.__TAURI_INTERNALS__ = {
    metadata: {currentWindow:{label:'main'},currentWebview:{label:'main'}},
    transformCallback(fn) { const key=++id; callbacks.set(key,fn); return key; },
    unregisterCallback(key) { callbacks.delete(key); },
    async invoke(cmd,args={}) {
      if(cmd==='plugin:event|listen') { const items=listeners.get(args.event)??[];items.push(args.handler);listeners.set(args.event,items);return args.handler; }
      if(cmd==='plugin:event|unlisten') return;
      if(cmd==='get_settings') return {schemaVersion:4,mascot:{enabled:false},appearance:{reducedMotion:true}};
      if(cmd==='set_notch_state') return;
      if(cmd==='list_monitors') return [{name:'Test display',width:1920,height:1080,scale_factor:1,is_primary:true}];
      if(cmd==='get_converter_status') return {status:window.mockAvailable?'ready':'missing',version:'Filee test fixture',message:window.mockAvailable?'Ready · offline conversion':'Filee CLI was not found. Install Filee or locate filee-cli.exe.'};
      if(cmd==='get_conversion_state') return snapshot;
      if(cmd==='inspect_conversion_files') return {files:args.paths.map(file),targets:['jpg','png','pdf'],unavailableTargets:['docx']};
      if(cmd==='pick_conversion_path') return args.kind==='files'?['C:\\\\Sample Files\\\\holiday-photo.png']:[];
      if(cmd==='start_conversion') {
        snapshot={jobId:'test-job',running:true,files:args.paths.map(p=>({...file(p),state:'running',progress:.5})),target:args.target,message:'Converting…'};
        window.mockEmit('converter.changed',structuredClone(snapshot));
        return new Promise(resolve=>{finish=()=>{snapshot={...snapshot,running:false,message:'1 of 1 converted',files:snapshot.files.map(f=>({...f,state:'done',progress:1,outputs:['C:\\\\Converted\\\\photo.jpg']}))};window.mockEmit('converter.changed',structuredClone(snapshot));resolve(snapshot);};setTimeout(finish,400);});
      }
      if(cmd==='open_conversion_result') {window.mockOpened=args;return;}
      if(cmd==='save_settings') return args.settings;
      throw new Error('Unavailable in UI fixture: '+cmd);
    }
  };
})();`;
try {
  let pages;
  const end = Date.now() + 15000;
  while (Date.now() < end) {
    try { pages = await fetch("http://127.0.0.1:9335/json/list").then(r => r.json()); if (pages.some(p => p.type === "page")) break; } catch {}
    await delay(100);
  }
  const page = pages?.find(p => p.type === "page");
  assert.ok(page, "isolated browser started");
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
  ws.onmessage = event => {
    const data = JSON.parse(event.data);
    if (data.id) { const item = pending.get(data.id); pending.delete(data.id); if (data.error) item?.reject(new Error(data.error.message)); else item?.resolve(data.result); }
    if (data.method === "Runtime.exceptionThrown") errors.push(data.params.exceptionDetails.exception?.description ?? data.params.exceptionDetails.text);
  };
  await command("Page.enable"); await command("Runtime.enable");
  await command("Emulation.setDeviceMetricsOverride", { width: 760, height: 520, deviceScaleFactor: 1, mobile: false });
  await command("Page.addScriptToEvaluateOnNewDocument", { source: mock });
  const serverDeadline = Date.now() + 15000;
  while (Date.now() < serverDeadline) {
    try { if ((await fetch("http://127.0.0.1:1421")).ok) break; } catch {}
    await delay(100);
  }
  await command("Page.navigate", { url: "http://127.0.0.1:1421" });
  await click("Open notch"); await click("More");
  await screenshot("more-tools");
  await click("Convert files"); await until("document.body.innerText.includes('Drop a file.')", "converter ready");
  await screenshot("converter-empty");
  const listenersBefore = await evaluate("window.mockListenerCount()");
  await command("HeapProfiler.collectGarbage");
  const heapBefore = (await command("Runtime.getHeapUsage")).usedSize;
  for (let index = 0; index < 100; index++) {
    await click("Back"); await click("Convert files");
    await until("document.querySelector('.converter-panel') !== null", "converter remount");
  }
  await command("HeapProfiler.collectGarbage");
  const heapGrowth = (await command("Runtime.getHeapUsage")).usedSize - heapBefore;
  assert.equal(await evaluate("window.mockListenerCount()"), listenersBefore);
  assert.ok(heapGrowth < 2 * 1024 * 1024, `UI heap grew by ${heapGrowth} bytes`);
  await click("Choose files"); await until("document.body.innerText.includes('holiday-photo.png')", "file inspection");
  assert.equal(await evaluate("document.querySelector('#conversion-format').options.length"), 3);
  await screenshot("converter-selected");
  await click("Convert"); await until("document.body.innerText.includes('1 of 1 converted')", "conversion result");
  assert.equal(await evaluate("[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='Open'&&b.getClientRects().length)"), true);
  await click("Open");
  assert.equal(await evaluate("window.mockOpened.folder"), false);
  await click("Folder");
  assert.equal(await evaluate("window.mockOpened.folder"), true);
  await screenshot("converter-result");
  await click("Clear");
  await evaluate("window.mockEmit('tauri://drag-enter',{paths:['C:\\\\photo.png'],position:{x:380,y:20}})");
  await until("[...document.querySelectorAll('span')].some(s=>s.textContent==='Drop to convert'&&getComputedStyle(s.parentElement).opacity==='1')", "drag feedback");
  await screenshot("converter-drag");
  await evaluate("window.mockEmit('tauri://drag-drop',{paths:['C:\\\\photo.png'],position:{x:380,y:20}})");
  await until("document.querySelector('#conversion-format')!==null", "native drop selection");
  assert.equal(await evaluate("document.querySelector('.notch-shell').scrollWidth <= document.querySelector('.notch-shell').clientWidth"), true);
  await click("Back"); await evaluate("window.mockAvailable=false");
  await click("Settings"); await click("Tools"); await click("Check again");
  await until("document.body.innerText.includes('Filee CLI was not found')", "missing engine status");
  await click("Back to More"); await click("Convert files");
  await until("document.body.innerText.includes('Connect Filee once')", "missing engine setup");
  await screenshot("converter-setup");
  assert.deepEqual(errors, []);
  console.log(JSON.stringify({ passed: true, openCloseCycles: 100, heapGrowthBytes: heapGrowth, listeners: listenersBefore, scenarios: ["navigation", "file selection", "compatible targets", "conversion results", "native drag feedback", "native drop selection", "missing Filee"], screenshots: output }));
} finally {
  if (ws?.readyState === WebSocket.OPEN) { try { await command("Browser.close"); } catch {} ws.close(); }
  browser.kill();
  server.kill();
}
