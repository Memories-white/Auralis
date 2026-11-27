const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
const root=path.resolve(__dirname,'..'),assets=path.join(root,'android/app/src/main/assets/ui'),output=path.join(root,'device-debug/rust-ui/dock');fs.mkdirSync(output,{recursive:true});
const rows=Array.from({length:18},(_,i)=>({mid:`dock-${i}`,title:`Dock song ${i}`,artist:'Layout study',duration:180}));
const state={page:'songs',detailTitle:'Layout study',tracks:rows,currentSong:rows[0],queue:rows,likes:rows,recent:[],discovery:[],playlists:[],recommendations:[],history:[],artists:[],counts:{tracks:18,likes:18},safeTop:24,safeBottom:20,uiPreferences:{},lyrics:'[00:00.000]A quiet song',wordLyrics:'[]',colors:{bg:'#141414',card:'#242424',fg:'#f2f2f2',muted:'#96979b',primary:'#91b6ff','on-primary':'#182c4b',border:'#3d3e42'}};
let live={mid:rows[0].mid,title:rows[0].title,artist:rows[0].artist,playing:false,position:1000,duration:180000};
(async()=>{const browser=await chromium.launch({headless:true});try{
 const page=await browser.newPage({viewport:{width:411,height:914}}),failures=[],report=[];
 await page.addInitScript(()=>{window.AuralisNative={postMessage:()=>{}}});
 await page.route('https://auralis.local/**',route=>{const u=new URL(route.request().url()),file=path.resolve(assets,'.'+u.pathname);assert.ok(file.startsWith(assets+path.sep));return route.fulfill({contentType:file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':'text/html',body:fs.readFileSync(file)});});
 await page.goto('https://auralis.local/index.html');await page.waitForFunction(()=>window.auralisUpdate);
 const content=()=>page.evaluate(state=>auralisUpdate(JSON.stringify({type:'content',state})),state),pulse=()=>page.evaluate(state=>auralisUpdate(JSON.stringify({type:'playback',state})),live);
 for(const bar of ['normal','floating'])for(const safe of [0,20,48]){
   await page.setViewportSize({width:411,height:914});state.page='songs';state.safeBottom=safe;state.uiPreferences={bar};await content();live.playing=false;await pulse();live.playing=true;await pulse();await page.waitForTimeout(550);
   await page.locator('#content').evaluate(e=>e.scrollTop=e.scrollHeight);await page.waitForTimeout(40);
   const m=await page.evaluate(()=>({last:document.querySelector('#song-tracks-17').getBoundingClientRect().bottom,mini:document.querySelector('#mini-player').getBoundingClientRect().top,locate:document.querySelector('#float-actions').getBoundingClientRect().bottom,maskHeight:parseFloat(getComputedStyle(document.querySelector('#dock'),'::after').height)||0,maskColor:getComputedStyle(document.querySelector('#dock'),'::after').backgroundColor}));
   report.push({bar,safe,...m});if(m.last>m.mini-8)failures.push(`${bar}, safe=${safe}: expanded dock hides last row (${m.last}>${m.mini})`);
   if(m.mini-m.locate<8||m.mini-m.locate>40)failures.push(`${bar}: locate overlaps or moves twice with dock expansion`);
   if(safe>0&&(m.maskHeight<safe||m.maskColor==='rgba(0, 0, 0, 0)'))failures.push(`${bar}: gesture area has no opaque surface`);
 }
 for(const bar of ['normal','floating']){
   state.page='home';state.uiPreferences={bar};state.safeBottom=20;await page.setViewportSize({width:900,height:650});await content();await page.waitForTimeout(300);
   const m=await page.evaluate(()=>{const n=document.querySelector('#navigation'),r=n.getBoundingClientRect();return{width:r.width,height:r.height,buttonsFit:[...n.querySelectorAll('button')].every(e=>{const b=e.getBoundingClientRect();return b.x>=r.x&&b.right<=r.right+1&&b.y>=r.y&&b.bottom<=r.bottom+1;})}});report.push({wide:true,bar,...m});if(m.width>110||!m.buttonsFit)failures.push(`${bar}: wide navigation overflows rail`);
 }
 await page.setViewportSize({width:411,height:914});state.uiPreferences={bar:'normal',compact:'on'};await content();await page.waitForTimeout(550);
 assert.equal(await page.locator('.mini-expanded').evaluate(e=>e.inert),true,'Hidden mini controls still receive focus');
 assert.equal(await page.locator('.mini-compact').evaluate(e=>e.inert),false);
 state.uiPreferences={bar:'normal',compact:'off'};state.reducedMotion=true;await content();live.playing=false;await pulse();live.playing=true;await pulse();await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
 assert.equal(await page.locator('#mini-player').evaluate(e=>Number(e.style.getPropertyValue('--mini'))),1,'Reduced motion still animates mini expansion');
 fs.writeFileSync(path.join(output,'layout-report.json'),JSON.stringify({report,failures},null,2));assert.deepEqual(failures,[]);console.log('PASS: expanded list clearance, opaque gesture area, wide navigation, hidden controls and reduced motion.');
}finally{await browser.close()}})().catch(e=>{console.error(e);process.exitCode=1});
