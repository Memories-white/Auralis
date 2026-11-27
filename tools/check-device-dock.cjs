// Real Auralis WebView only. Preserves account/queue and restores the bar choice.
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright'),fs=require('fs'),path=require('path'),assert=require('node:assert/strict');
const output=path.resolve(__dirname,'../device-debug/usb-059');fs.mkdirSync(output,{recursive:true});
(async()=>{const b=await chromium.connectOverCDP(process.env.AURALIS_CDP||'http://127.0.0.1:9228');const p=b.contexts()[0].pages().find(p=>p.url().includes('auralis.local'));if(!p)throw Error('Auralis page missing');const report={styles:[],returns:[]};let originalBar,originalPlaying=false;try{
 const send=event=>p.evaluate(event=>AuralisNative.postMessage(JSON.stringify(event)),event);
 originalBar=await p.locator('#app').getAttribute('data-ui-bar');
 originalPlaying=(await p.locator('.mini-compact [data-action=play]').getAttribute('aria-label'))==='暂停';
 const originalIdentity=await p.locator('.mini-meta strong').innerText();
 for(const bar of ['normal','floating']){
   await send({type:'navigate',page:'settings'});await p.locator('[data-action="navigate:ui-customize"]').click();await p.locator('[data-action="navigate:ui-dock"]').click();await p.locator(`[data-action="ui:bar:${bar}"]`).click();
   await p.waitForFunction(bar=>document.querySelector('#app').dataset.uiBar===bar,bar);await send({type:'navigate',page:'home'});await p.waitForTimeout(250);
   const before=await p.locator('[data-time=position]').first().innerText();
   if((await p.locator('.mini-compact [data-action=play]').getAttribute('aria-label'))!=='暂停')await p.locator('.mini-compact [data-action=play]').click();
   let played=false;try{await p.waitForFunction(()=>document.querySelector('.mini-five [data-action=play]')?.getAttribute('aria-label')==='暂停',null,{timeout:15000});played=true;}catch{}
   await p.waitForTimeout(1200);
   const geometry=await p.evaluate(()=>{const mini=document.querySelector('#mini-player'),content=document.querySelector('#content'),mask=getComputedStyle(document.querySelector('#dock'),'::after'),r=mini.getBoundingClientRect();return{height:r.height,miniTop:r.top,padding:parseFloat(getComputedStyle(content).paddingBottom),viewport:innerHeight,maskHeight:parseFloat(mask.height),maskColor:mask.backgroundColor,controlsFit:[...mini.querySelectorAll('.mini-five button')].every(e=>{const a=e.getBoundingClientRect();return a.bottom<=r.bottom+1&&a.top>=r.top;})};});
   const after=await p.locator('[data-time=position]').first().innerText();report.styles.push({bar,played,positionAdvanced:before!==after,geometry});assert.ok(geometry.maskHeight>0&&geometry.maskColor!=='rgba(0, 0, 0, 0)');
   if(played){assert.ok(geometry.height>150&&geometry.controlsFit);assert.ok(geometry.padding>=geometry.viewport-geometry.miniTop+8);await p.locator('.mini-five [data-action=queue]').click();assert.ok(await p.locator('.sheet').isVisible());await p.locator('.sheet [data-action=close_sheet]').click();await p.waitForTimeout(350);}
   await send({type:'transport',action:'PAUSE'});
   if(await p.locator('.collapse-mini').isVisible())await p.locator('.collapse-mini').click();await p.waitForTimeout(400);
   assert.equal(await p.locator('.mini-expanded').evaluate(e=>e.inert),true);
   await p.screenshot({path:path.join(output,`dock-${bar}.png`)});
   await p.locator('#mini-player .mini-meta').click();await p.waitForTimeout(650);await p.locator('[data-action=player_lyrics]').click();await p.waitForTimeout(500);
   report.knownLyrics=await p.locator('.lyric-line').count();await p.evaluate(()=>auralisBack());await p.waitForTimeout(750);
   report.returns.push(await p.evaluate(()=>({mounted:document.querySelector('#player').classList.contains('mounted'),inert:document.querySelector('#dock').inert,opacity:Number(getComputedStyle(document.querySelector('#mini-player')).opacity)})));
   assert.equal(await p.locator('.mini-meta strong').innerText(),originalIdentity,'Testing changed the current track');
 }
 report.result=report.styles.every(s=>s.played)?'passed':'layout-passed-playback-unavailable';console.log(JSON.stringify(report,null,2));fs.writeFileSync(path.join(output,'device-dock-report.json'),JSON.stringify(report,null,2));
}finally{
 try{
 for(let i=0;i<4;i++){if(await p.evaluate(()=>!!document.querySelector('.sheet')||document.querySelector('#player')?.classList.contains('visible'))){await p.evaluate(()=>auralisBack());await p.waitForTimeout(250);}else break;}
 // Restore with the actual Rust option so the native schema stays the same.
 if(originalBar){await p.evaluate(()=>AuralisNative.postMessage(JSON.stringify({type:'navigate',page:'settings'})));await p.locator('[data-action="navigate:ui-customize"]').click();await p.locator('[data-action="navigate:ui-dock"]').click();await p.locator(`[data-action="ui:bar:${originalBar}"]`).click();await p.evaluate(()=>AuralisNative.postMessage(JSON.stringify({type:'navigate',page:'home'})));}
 await p.evaluate(playing=>AuralisNative.postMessage(JSON.stringify({type:'transport',action:playing?'RESUME':'PAUSE'})),originalPlaying).catch(()=>{});
 }finally{await b.close();}
}})().catch(e=>{console.error(e);process.exitCode=1});
