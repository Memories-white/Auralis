// App-scoped, read-only playback state: inspect the real click/expand path.
const fs=require('node:fs'),path=require('node:path');
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
(async()=>{const browser=await chromium.connectOverCDP(process.env.AURALIS_CDP||'http://127.0.0.1:9228');
try{const p=browser.contexts()[0].pages().find(p=>p.url().includes('auralis.local'));if(!p)throw Error('Auralis WebView missing');
if(await p.locator('#player').evaluate(e=>e.classList.contains('visible'))){await p.evaluate(()=>auralisBack());await p.waitForTimeout(900);}
// Warm the expensive case: lyrics have already been mounted and retained.
await p.locator('#mini-player .mini-meta').click();await p.waitForTimeout(850);
await p.locator('[data-action=player_lyrics]').click();await p.waitForTimeout(750);
await p.evaluate(()=>auralisBack());await p.waitForTimeout(900);
const session=await p.context().newCDPSession(p);await session.send('Performance.enable');
const report={scenario:'warm cover opening after lyrics; WebView callbacks are not presented GPU frames',runs:[]};
for(let i=0;i<3;i++){
const trace=[];session.on('Tracing.dataCollected',collect);function collect(e){trace.push(...e.value);}
await session.send('Tracing.start',{categories:'devtools.timeline,blink.user_timing',transferMode:'ReportEvents'});
const before=(await session.send('Performance.getMetrics')).metrics;
const run=await p.evaluate(async()=>{
 const frames=[],longTasks=[];let last=0,mutations=0;
 const observer=new PerformanceObserver(list=>{for(const e of list.getEntries())longTasks.push({start:e.startTime,duration:e.duration});});observer.observe({type:'longtask',buffered:false});
 const dom=new MutationObserver(list=>{mutations+=list.length});dom.observe(document.querySelector('#player'),{subtree:true,attributes:true,childList:true});
 await new Promise(requestAnimationFrame);const start=performance.now();
 document.querySelector('#mini-player .mini-meta').click();const clickMs=performance.now()-start;
 await new Promise(resolve=>{function frame(t){if(last)frames.push(t-last);last=t;if(t-start<900)requestAnimationFrame(frame);else resolve();}requestAnimationFrame(frame);});
 observer.disconnect();dom.disconnect();const sorted=[...frames].sort((a,b)=>a-b);
 return {clickMs,callbacks:frames.length,p50:sorted[Math.floor(sorted.length*.5)],p95:sorted[Math.floor(sorted.length*.95)],max:Math.max(...frames),over25ms:frames.filter(v=>v>25).length,longTasks,mutations,lyricVisibility:getComputedStyle(document.querySelector('#lyric-viewport')).visibility};
});
const after=(await session.send('Performance.getMetrics')).metrics;
const done=new Promise(resolve=>session.once('Tracing.tracingComplete',resolve));await session.send('Tracing.end');await done;session.off('Tracing.dataCollected',collect);
run.metrics={};for(const name of ['LayoutCount','RecalcStyleCount','LayoutDuration','RecalcStyleDuration','ScriptDuration','TaskDuration']){run.metrics[name]=after.find(m=>m.name===name).value-before.find(m=>m.name===name).value;}
run.trace={};for(const e of trace){if(!['Layout','UpdateLayoutTree','Paint','PrePaint','FunctionCall','RunMicrotasks'].includes(e.name)||!e.dur)continue;const row=run.trace[e.name]??={count:0,totalMs:0,maxMs:0};row.count++;row.totalMs+=e.dur/1000;row.maxMs=Math.max(row.maxMs,e.dur/1000);}
run.longEvents=trace.filter(e=>e.dur>30000&&e.ph==='X').sort((a,b)=>b.dur-a.dur).slice(0,8).map(e=>({name:e.name,duration:e.dur/1000}));
report.runs.push(run);await p.evaluate(()=>auralisBack());await p.waitForTimeout(900);
}
const dir=path.resolve(__dirname,'../device-debug/usb-0512');fs.mkdirSync(dir,{recursive:true});const label=process.env.PROFILE_LABEL||'current';fs.writeFileSync(path.join(dir,`open-${label}.json`),JSON.stringify(report,null,2));console.log(JSON.stringify(report,null,2));
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1});
