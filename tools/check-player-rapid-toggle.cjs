const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
const root=path.resolve(__dirname,'..'),assets=path.join(root,'android/app/src/main/assets/ui');
const song={mid:'rapid-fixture',title:'Rapid return',artist:'Motion study',duration:180};
const state={page:'home',currentSong:song,queue:[song],tracks:[],likes:[],recent:[],discovery:[],recommendations:[],playlists:[],history:[],artists:[],counts:{},safeTop:24,safeBottom:16,uiPreferences:{},lyrics:'[00:00.000]A quiet return',wordLyrics:'[]',lyricState:'ready',colors:{bg:'#141414',card:'#242424',fg:'#f2f2f2',muted:'#96979b',primary:'#91b6ff',border:'#3d3e42'}};
const playback={mid:song.mid,title:song.title,artist:song.artist,playing:false,position:1000,duration:180000};
(async()=>{const b=await chromium.launch({headless:true});try{
 const p=await b.newPage({viewport:{width:411,height:914},isMobile:true,hasTouch:true});
 await p.addInitScript(()=>{window.AuralisNative={postMessage:()=>{}}});
 await p.route('https://auralis.local/**',r=>{const u=new URL(r.request().url()),f=path.resolve(assets,'.'+u.pathname);assert.ok(f.startsWith(assets+path.sep));return r.fulfill({contentType:f.endsWith('.wasm')?'application/wasm':f.endsWith('.js')?'text/javascript':'text/html',body:fs.readFileSync(f)});});
 await p.goto('https://auralis.local/index.html');await p.waitForFunction(()=>window.auralisUpdate);
 await p.evaluate(({state,playback})=>{auralisUpdate(JSON.stringify({type:'content',state}));auralisUpdate(JSON.stringify({type:'playback',state:playback}));},{state,playback});
 await p.locator('#mini-player .mini-meta').click();await p.waitForTimeout(800);
 const tail=await p.evaluate(async()=>{auralisBack();const start=performance.now();return new Promise((resolve,reject)=>{function sample(){const m=document.querySelector('#mini-player').getBoundingClientRect(),s=document.querySelector('#player-surface').getBoundingClientRect();const p=(s.height-m.height)/(innerHeight-m.height);if(p>0&&p<.00045){const mounted=document.querySelector('#player').classList.contains('mounted');auralisUpdate(JSON.stringify({type:'open_player'}));resolve({p,mountedBefore:mounted,mountedAfter:document.querySelector('#player').classList.contains('mounted')});}else if(performance.now()-start>2000)reject(Error('Missed closing-tail window'));else requestAnimationFrame(sample);}requestAnimationFrame(sample);});});
 await p.waitForTimeout(750);
 assert.ok(await p.locator('#player').evaluate(e=>e.classList.contains('mounted')&&getComputedStyle(e).visibility==='visible'),`Tail reversal hid both the player and mini: ${JSON.stringify(tail)}`);
 for(const delay of [0,16,40,85,160,350,500,560,600]){
   await p.evaluate(()=>auralisBack());await p.waitForTimeout(delay);await p.evaluate(()=>auralisUpdate(JSON.stringify({type:'open_player'})));await p.waitForTimeout(35);await p.evaluate(()=>auralisBack());await p.waitForTimeout(800);
   const end=await p.evaluate(()=>({mini:Number(getComputedStyle(document.querySelector('#mini-player')).opacity),mounted:document.querySelector('#player').classList.contains('mounted'),inert:document.querySelector('#dock').inert,shared:!!document.querySelector('#shared-artwork')}));
   assert.deepEqual(end,{mini:1,mounted:false,inert:false,shared:false},`Rapid toggle failed at ${delay}ms`);await p.locator('#mini-player .mini-meta').click();await p.waitForTimeout(80);
 }
 await p.evaluate(()=>auralisBack());await p.waitForTimeout(850);console.log('PASS: closing-tail reversal and 9 rapid open/exit cycles retain one visible usable player surface.');
}finally{await b.close()}})().catch(e=>{console.error(e);process.exitCode=1});
