// Real WebView test. Does not play, change track, log in, or alter preferences.
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright'),assert=require('node:assert/strict'),fs=require('fs'),path=require('path'),{CARD_RECT_SOURCE}=require('./sheet-geometry.cjs');
(async()=>{const b=await chromium.connectOverCDP(process.env.AURALIS_CDP||'http://127.0.0.1:9228');const p=b.contexts()[0].pages().find(p=>p.url().includes('auralis.local'));if(!p)throw Error('Auralis WebView missing');await p.evaluate(CARD_RECT_SOURCE+'; true;');const report={};try{
 if(await p.locator('#player').evaluate(e=>e.classList.contains('visible'))){await p.evaluate(()=>auralisBack());await p.waitForTimeout(850);}
 await p.locator('#mini-player .mini-meta').click();await p.waitForTimeout(800);
 report.tail=await p.evaluate(async()=>{auralisBack();const start=performance.now();return new Promise((resolve,reject)=>{function sample(){const mini=document.querySelector('#mini-player').getBoundingClientRect(),card=window.__auralisCard();const progress=(card.h-mini.height)/(innerHeight-mini.height);if(progress>0&&progress<.00045){const before=document.querySelector('#player').classList.contains('mounted');auralisUpdate(JSON.stringify({type:'open_player'}));resolve({progress,before,after:document.querySelector('#player').classList.contains('mounted')});}else if(performance.now()-start>2200)reject(Error('Closing-tail sampling missed'));else requestAnimationFrame(sample);}requestAnimationFrame(sample);});});
 await p.waitForTimeout(800);assert.ok(await p.locator('#player').evaluate(e=>e.classList.contains('mounted')&&getComputedStyle(e).visibility==='visible'),'Closing-tail reopen left the player hidden');
 report.cycles=0;
 for(const delay of [0,20,80,200,500,560,600,640]){
   await p.evaluate(()=>auralisBack());await p.waitForTimeout(delay);await p.evaluate(()=>auralisUpdate(JSON.stringify({type:'open_player'})));await p.waitForTimeout(30);await p.evaluate(()=>auralisBack());await p.waitForTimeout(800);
   const end=await p.evaluate(()=>({opacity:Number(getComputedStyle(document.querySelector('#mini-player')).opacity),mounted:document.querySelector('#player').classList.contains('mounted'),inert:document.querySelector('#dock').inert,shared:!!document.querySelector('#shared-artwork')}));assert.deepEqual(end,{opacity:1,mounted:false,inert:false,shared:false});report.cycles++;await p.locator('#mini-player .mini-meta').click();await p.waitForTimeout(80);
 }
 report.result='passed';console.log(JSON.stringify(report));
}finally{
 if(await p.locator('#player').evaluate(e=>e.classList.contains('visible'))){await p.evaluate(()=>auralisBack());await p.waitForTimeout(850);}
 const dir=path.resolve(__dirname,'../device-debug/usb-0510');fs.mkdirSync(dir,{recursive:true});fs.writeFileSync(path.join(dir,'rapid-toggle.json'),JSON.stringify(report,null,2));await b.close();
}})().catch(e=>{console.error(e);process.exitCode=1});
