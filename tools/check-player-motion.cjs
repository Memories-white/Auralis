// Real WASM motion cost probe. Browser callbacks/CPU metrics are not device GPU FPS.
const fs=require('node:fs'),path=require('node:path');
const assert=require('node:assert/strict');
const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
const root=path.resolve(__dirname,'..'),assets=path.join(root,'android/app/src/main/assets/ui');
const output=path.join(root,'device-debug/rust-ui/motion');fs.mkdirSync(output,{recursive:true});
const tag=process.argv[2]||'current';
const data={page:'home',loggedIn:false,homeReady:false,currentSong:{mid:'motion',title:'Motion study',artist:'Auralis',coverUrl:'https://example.invalid/motion'},
 discovery:[],tracks:[],recent:[],favorites:[],likes:[],search:[],playlists:[],recommendations:[],queue:[],history:[],artists:[],counts:{},
 colorMode:'light',monet:false,uiPreferences:{bar:'glass'},safeTop:24,safeBottom:16,lyrics:Array.from({length:400},(_,i)=>`[${String(Math.floor(i*2/60)).padStart(2,'0')}:${String(i*2%60).padStart(2,'0')}.000]A quiet melody for motion study ${i}`).join('\n'),translation:'',
 colors:{bg:'#f6f6f6',card:'#fff',fg:'#191919',muted:'#73757a',primary:'#3482ff','on-primary':'#fff',border:'#e7e8eb'}};
const live={mid:'motion',title:'Motion study',artist:'Auralis',cover:'https://example.invalid/motion',playing:false,position:12000,duration:240000};
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE});
 try{
  const page=await browser.newPage({viewport:{width:411,height:914},deviceScaleFactor:1,isMobile:true,hasTouch:true});
  await page.addInitScript(()=>{window.AuralisNative={postMessage:()=>{}};});
  await page.route('https://auralis.local/**',async route=>{
   const u=new URL(route.request().url());
   if(u.pathname==='/artwork')return route.fulfill({contentType:'image/svg+xml',body:'<svg xmlns="http://www.w3.org/2000/svg" width="500" height="500"><defs><linearGradient id="a" x2="1" y2="1"><stop stop-color="#637e92"/><stop offset="1" stop-color="#c9b6a5"/></linearGradient></defs><path fill="url(#a)" d="M0 0h500v500H0z"/><circle cx="250" cy="250" r="150" fill="none" stroke="#ffffff88"/></svg>'});
   const file=path.resolve(assets,'.'+u.pathname);if(!file.startsWith(assets+path.sep))throw Error('Invalid asset');
   return route.fulfill({contentType:file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':'text/html',body:fs.readFileSync(file)});
  });
  await page.goto('https://auralis.local/index.html');await page.waitForFunction(()=>typeof window.auralisUpdate==='function');
  await page.evaluate(({data,live})=>{auralisUpdate(JSON.stringify({type:'content',state:data}));auralisUpdate(JSON.stringify({type:'playback',state:live}));},{data,live});
  await page.waitForTimeout(600);
  await page.screenshot({path:path.join(output,tag+'-dock-light.png')});
  const cdp=await page.context().newCDPSession(page);await cdp.send('Performance.enable');
  const metrics=async()=>Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map(m=>[m.name,m.value]));
  const before=await metrics();
  const frames=await page.evaluate(async()=>{
   const samples=[];let mutations=0;const observer=new MutationObserver(ms=>{mutations+=ms.length;});
   observer.observe(document.querySelector('#player'),{subtree:true,attributes:true,attributeFilter:['style']});
   const start=performance.now();let firstCallbackMs;document.querySelector('#mini-player .mini-identity').click();
   await new Promise(resolve=>{const tick=t=>{firstCallbackMs??=performance.now()-start;samples.push(t);if(t-start<1100)requestAnimationFrame(tick);else resolve();};requestAnimationFrame(tick);});observer.disconnect();
   const gaps=samples.slice(1).map((v,i)=>v-samples[i]).sort((a,b)=>a-b);
   return{callbacks:samples.length,firstCallbackMs,medianMs:gaps[Math.floor(gaps.length*.5)],p95Ms:gaps[Math.floor(gaps.length*.95)],maximumMs:Math.max(...gaps),styleMutations:mutations};
  });
  const after=await metrics();const delta={};for(const key of ['LayoutCount','LayoutDuration','RecalcStyleCount','RecalcStyleDuration','ScriptDuration','TaskDuration'])delta[key]=after[key]-before[key];
  const geometry=await page.evaluate(()=>({miniHeight:document.querySelector('#mini-player').getBoundingClientRect().height,navHeight:document.querySelector('#navigation').getBoundingClientRect().height,lyricsNodes:document.querySelectorAll('.lyric-line').length,clipPath:getComputedStyle(document.querySelector('#player')).clipPath}));
  assert.deepEqual(geometry,{miniHeight:52,navHeight:54,lyricsNodes:0,clipPath:'none'});
  const result={tag,scope:'Chromium real WASM, 400 line fixture; callbacks and CPU, not phone GPU FPS',frames,delta,geometry};
  fs.writeFileSync(path.join(output,tag+'.json'),JSON.stringify(result,null,2));console.log(JSON.stringify(result));
  await page.screenshot({path:path.join(output,tag+'-player.png')});
  await page.locator('[data-action=close_player]').click(); await page.waitForTimeout(800);
  data.colors={bg:'#141414',card:'#232323',fg:'#f2f2f2',muted:'#96979b',primary:'#91b6ff','on-primary':'#182c4b',border:'#3d3e42'};
  data.colorMode='dark';
  await page.evaluate(data=>auralisUpdate(JSON.stringify({type:'content',state:data})),data);
  await page.waitForTimeout(400);await page.screenshot({path:path.join(output,tag+'-dock-dark.png')});
  for(const [bar,mini,nav,width] of [['floating',64,64,308],['normal',68,64,411]]){
   data.uiPreferences={bar};await page.evaluate(data=>auralisUpdate(JSON.stringify({type:'content',state:data})),data);
   const size=await page.evaluate(()=>({mini:document.querySelector('#mini-player').getBoundingClientRect().height,nav:document.querySelector('#navigation').getBoundingClientRect().height,width:document.querySelector('#navigation').getBoundingClientRect().width}));
   assert.deepEqual(size,{mini,nav,width});
  }
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
