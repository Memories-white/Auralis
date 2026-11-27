const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright'),assert=require('node:assert/strict'),fs=require('fs'),path=require('path');
(async()=>{const b=await chromium.connectOverCDP(process.env.AURALIS_CDP||'http://127.0.0.1:9228'),p=b.contexts()[0].pages().find(p=>p.url().includes('auralis.local'));if(!p)throw Error('Auralis WebView missing');const report={};let translation;try{
 if(await p.locator('#player').evaluate(e=>e.classList.contains('visible'))){await p.evaluate(()=>auralisBack());await p.waitForTimeout(800);}
 translation=await p.locator('#app').getAttribute('data-ui-translation');
 await p.locator('#mini-player .mini-meta').click();await p.waitForTimeout(800);
 await p.locator('#cover-art').evaluate(e=>{window.__stableCover=e;window.__stableWash=document.querySelector('.player-wash');window.__stablePager=document.querySelector('#player-pages');});
 await p.locator('[data-action=player_lyrics]').click();await p.waitForTimeout(600);await p.locator('#lyric-viewport').evaluate(e=>{window.__stableLyrics=e;window.__stableLine=document.querySelector('.lyric-line.active');});
 await p.locator('.lyric-options [data-action^="ui:translation:"]').click();await p.waitForFunction(original=>document.querySelector('#app').dataset.uiTranslation!==original,translation);await p.waitForTimeout(100);
 report.controls=await p.evaluate(()=>({cover:window.__stableCover===document.querySelector('#cover-art'),wash:window.__stableWash===document.querySelector('.player-wash'),pager:window.__stablePager===document.querySelector('#player-pages')}));assert.ok(Object.values(report.controls).every(Boolean),'Preference update remounted the cover/background/pager');
 report.lyrics=await p.evaluate(()=>({viewport:window.__stableLyrics===document.querySelector('#lyric-viewport'),row:window.__stableLine===document.querySelector('.lyric-line.active')}));assert.ok(Object.values(report.lyrics).every(Boolean),'Preference update remounted lyric state');
 for(let i=0;i<3;i++){await p.evaluate(()=>auralisBack());await p.waitForTimeout(800);await p.locator('#mini-player .mini-meta').click();await p.waitForTimeout(750);assert.ok(await p.evaluate(()=>window.__stableCover===document.querySelector('#cover-art')),'Warm opening discarded the cover');}
 report.warmOpens=3;report.result='passed';console.log(JSON.stringify(report));
}finally{
 if(translation&&(await p.locator('#app').getAttribute('data-ui-translation'))!==translation){await p.evaluate(()=>document.querySelector('.lyric-options [data-action^="ui:translation:"]')?.click());await p.waitForTimeout(150);}
 if(await p.locator('#player').evaluate(e=>e.classList.contains('visible'))){await p.evaluate(()=>auralisBack());await p.waitForTimeout(800);}
 const dir=path.resolve(__dirname,'../device-debug/usb-0511');fs.mkdirSync(dir,{recursive:true});fs.writeFileSync(path.join(dir,'stability.json'),JSON.stringify(report,null,2));await b.close();
}})().catch(e=>{console.error(e);process.exitCode=1});
