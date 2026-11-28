// Integration checks for the real compiled Rust/WASM UI, with a mock native bridge.
// PLAYWRIGHT_MODULE may point to an existing Playwright installation.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const root = path.resolve(__dirname, '..');
const assets = path.join(root, 'android/app/src/main/assets/ui');
const output = path.join(root, 'device-debug/rust-ui/browser');
fs.mkdirSync(output, { recursive: true });

const song = i => ({ mid: `fixture-${i}`, title: ['Into the Light', '慢慢听', 'A Quiet Morning', 'Somewhere New'][i % 4], artist: ['Auralis Studio', '音乐时光'][i % 2], album: 'Interface Study', coverUrl: `https://example.invalid/art-${i}`, duration: 180, id: i + 1 });
const rows = Array.from({ length: 160 }, (_, i) => song(i));
const playlists = [0, 1, 2].map(i => ({ id: i + 1, dirid: i + 1, title: ['私人收藏', '晚风里的旋律', '周末散步'][i], count: 24 + i, saved: i === 2, cover: `https://example.invalid/list-${i}` }));
const data = {
  page: 'home', loggedIn: true, homeReady: true, libraryReady: true, nickname: 'Auralis Listener', uin: '000000', avatar: 'https://example.invalid/avatar',
  dailyCover: rows[1].coverUrl, discovery: rows.slice(0, 18), tracks: rows, recent: rows.slice(0, 4), favorites: rows.slice(0, 2), likes: rows.slice(0, 24),
  search: [], playlists, recommendations: playlists, queue: rows.slice(0, 8), history: ['周末', '慢慢听'], artists: ['Auralis Studio', '音乐时光'],
  detailTitle: '晚风里的旋律', detailCover: playlists[1].cover, detailLoading: false, detailError: '', query: '', searchedQuery: '', busy: false, status: '',
  discoveryLoading: false, discoveryMore: true, discoveryError: '', colorMode: 'light', monet: false, monetSource: 'wallpaper', monetStyle: 'TonalSpot', monetSpec: 'Spec2021', monetColor: '3482FF',
  quality: 'auto', qualityFallback: true, version: '0.5.17-preview', qrOpen: false, lyrics: '[00:00.00]让旋律慢慢靠近\n[00:03.00]把今天交给音乐\n[00:06.00]下一句，落在心里\n[00:09.00]和你一起慢慢听\n[00:12.00]每一刻都有回响\n[00:15.00]走向更远的地方',
  translation: '', currentSong: rows[1], collectionBusy: false, collectionMessage: '', safeTop: 24, safeBottom: 16,
  colors: { bg: '#f6f6f6', card: '#ffffff', fg: '#191919', muted: '#797b80', primary: '#3482ff', 'on-primary': '#ffffff', border: '#e7e8eb' },
};
data.counts = Object.fromEntries(['discovery', 'tracks', 'recent', 'favorites', 'likes', 'search', 'playlists', 'recommendations', 'queue', 'history', 'artists'].map(key => [key, data[key].length]));
const live = { mid: rows[1].mid, title: rows[1].title, artist: rows[1].artist, cover: rows[1].coverUrl, position: 1000, duration: 180000, playing: false, mode: 'ORDER', actualQuality: 'hq', favorite: false, sleepRemaining: 0, sleepAfterTrack: false, sleepMinutes: 0 };

(async () => {
  const browser = await chromium.launch({ headless: true, executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE });
  try {
    const page = await browser.newPage({ viewport: { width: 411, height: 914 }, deviceScaleFactor: 1, isMobile: true, hasTouch: true });
    const errors = [];
    const artworkRequests = new Map();
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.addInitScript(() => { window.__actions = []; window.AuralisNative = { postMessage: value => window.__actions.push(JSON.parse(value)) }; });
    let qrRequests = 0;
    await page.route('https://auralis.local/**', async route => {
      const uri = new URL(route.request().url());
      if (uri.pathname === '/qr') {
        qrRequests++;
        return route.fulfill({ status: 404, contentType: 'text/plain', body: '' });
      }
      if (uri.pathname === '/artwork') {
        const key = uri.searchParams.get('url');
        artworkRequests.set(key, (artworkRequests.get(key) || 0) + 1);
        await new Promise(resolve => setTimeout(resolve, 30)); // exercise concurrent views
        const seed = [...(uri.searchParams.get('url') || '')].reduce((n, c) => n + c.charCodeAt(0), 0) % 360;
        const body = `<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><defs><linearGradient id="a" x2="1" y2="1"><stop stop-color="hsl(${seed} 28% 64%)"/><stop offset="1" stop-color="hsl(${seed + 25} 36% 30%)"/></linearGradient></defs><path fill="url(#a)" d="M0 0h512v512H0z"/><circle cx="256" cy="256" r="145" fill="none" stroke="#ffffff40" stroke-width="1"/><circle cx="256" cy="256" r="105" fill="none" stroke="#ffffff30"/><path d="M210 303V186l110-22v116" fill="none" stroke="#fff9" stroke-width="10"/><circle cx="188" cy="305" r="25" fill="#fff9"/><circle cx="298" cy="282" r="25" fill="#fff9"/></svg>`;
        return route.fulfill({ contentType: 'image/svg+xml', body });
      }
      const file = path.resolve(assets, '.' + uri.pathname);
      assert.ok(file.startsWith(assets + path.sep));
      const type = file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.js') ? 'text/javascript' : 'text/html';
      return route.fulfill({ contentType: type, body: fs.readFileSync(file) });
    });
    await page.goto('https://auralis.local/index.html');
    await page.waitForFunction(() => typeof window.auralisUpdate === 'function');
    const content = async () => page.evaluate(input => window.auralisUpdate(JSON.stringify({ type: 'content', state: input })), data);
    const pulse = async () => page.evaluate(input => window.auralisUpdate(JSON.stringify({ type: 'playback', state: input })), live);
    const qrFixture = await page.evaluate(() => {
      const canvas = document.createElement('canvas');
      canvas.width = canvas.height = 296;
      const ctx = canvas.getContext('2d');
      ctx.fillStyle = 'white';
      ctx.fillRect(0, 0, 296, 296);
      ctx.fillStyle = 'black';
      ctx.fillRect(32, 32, 232, 232);
      return canvas.toDataURL('image/png').split(',')[1];
    });
    Object.assign(data, { qrOpen: true, qrStatus: '等待扫码', qrImage: qrFixture });
    await content();
    await page.waitForFunction(() => document.querySelector('.qr-content img')?.complete, null, { timeout: 2500 });
    assert.equal(await page.locator('.qr-content img').evaluate(img => img.naturalWidth), 296, 'Waiting for scan must display a readable image');
    const qrSize = await page.locator('.qr-content img').boundingBox();
    assert.ok(qrSize && qrSize.width >= 100 && Math.abs(qrSize.width - qrSize.height) < 1,
      `The QR image must occupy a visible square, got ${JSON.stringify(qrSize)}`);
    assert.equal(qrRequests, 0, 'The login QR image must load even when the WebView image route is unavailable');
    Object.assign(data, { qrStatus: '正在获取二维码', qrImage: '' });
    await content();
    assert.equal(await page.locator('.qr-content img').count(), 0, 'Refreshing must remove the previous QR image');
    data.qrOpen = false;
    await content(); await pulse();
    await page.waitForFunction(() => [...document.querySelectorAll('#content img[data-art-src]')].every(img => img.complete && img.naturalWidth > 0));
    const coverRequestsAtBoot = new Map(artworkRequests);
    assert.equal(artworkRequests.keys().next().value, data.currentSong.coverUrl, 'Restored current-song cover was queued behind the feed');
    assert.ok([...artworkRequests.values()].every(n => n === 1), 'Duplicate cover fetches during boot');
    await page.locator('[data-action="song:discovery:17"] img').evaluate(img => { window.__retainedCover = img; });
    data.status = '已缓存'; await content();
    assert.ok(await page.locator('[data-action="song:discovery:17"] img').evaluate(img => img === window.__retainedCover && img.complete && img.naturalWidth > 0), 'Content refresh discarded decoded cover');
    data.status = ''; data.page = 'mine'; await content(); data.page = 'home'; await content();
    await page.waitForFunction(() => [...document.querySelectorAll('#content img[data-art-src]')].every(img => img.complete && img.naturalWidth > 0));
    for (const [url, count] of coverRequestsAtBoot) assert.equal(artworkRequests.get(url), count, 'Changing pages refetched cover ' + url);
    assert.ok(await page.locator('#mini-player img').evaluate(img => img.complete && img.naturalWidth > 0));
    assert.equal(await page.locator('#navigation button').count(), 4);
    data.page='collection';await content();
    assert.equal(await page.locator('#navigation [aria-selected=true]').innerText(),'歌曲');
    assert.equal(await page.locator('[id^="song-likes-"]').count(),24);
    data.page='home';await content();
    assert.equal(await page.locator('[data-action=daily]').count(), 1);
    assert.equal(await page.locator('[data-action="song:discovery:6"]').count(), 1, 'Shelf songs repeat in discovery rows');
    assert.equal(await page.locator('.feed-status [data-action=load_more]').count(), 1, 'Recommendation paging needs a manual control');
    assert.equal(await page.evaluate(() => document.activeElement.tagName), 'BODY');
    const bounds = await page.evaluate(() => ({ header: document.querySelector('#header').getBoundingClientRect().bottom, content: document.querySelector('#content').getBoundingClientRect().top, shelf: document.querySelector('.shelf-card').getBoundingClientRect().x }));
    assert.ok(bounds.content >= bounds.header - 1, 'Content overlaps header');
    assert.ok(bounds.shelf >= 20, 'Shelf snaps away its leading gutter');
    await page.waitForTimeout(450); await page.screenshot({ path: path.join(output, 'home-light.png') });

    // Navigation emphasis is Rust-sampled, so every reading must be a valid
    // intermediate slot position that ends exactly on the selected item.
    data.page = 'mine'; await content();
    const dockTrack = [];
    for (let i = 0; i < 14; i++) {
      dockTrack.push(await page.evaluate(() => parseFloat(getComputedStyle(document.querySelector('#navigation')).getPropertyValue('--nav-index')) || 0));
      await page.waitForTimeout(16);
    }
    await page.waitForTimeout(600);
    const dockSettled = await page.evaluate(() => {
      const nav = document.querySelector('#navigation');
      const item = document.querySelectorAll('#navigation .nav-item')[2].getBoundingClientRect();
      return { index: parseFloat(getComputedStyle(nav).getPropertyValue('--nav-index')), height:nav.getBoundingClientRect().height,icon:document.querySelector('.nav-glyph svg').getBoundingClientRect().width,weight:getComputedStyle(document.querySelectorAll('.nav-item')[2]).fontWeight,selected:document.querySelectorAll('.nav-item')[2].getAttribute('aria-selected') };
    });
    assert.ok(dockTrack.some(v => v > 0.02 && v < 1.98), `Dock indicator jumped instead of gliding: ${dockTrack.join(',')}`);
    assert.ok(dockTrack.every(v => v >= 0 && v < 2.09), 'Dock overshoot exceeded the spring tolerance: '+dockTrack);
    assert.ok(dockSettled.index > 1.999 && dockSettled.index < 2.001, `Dock indicator did not settle on slot 2: ${dockSettled.index}`);
    assert.equal(await page.locator('.nav-item').first().evaluate(e=>getComputedStyle(e).backgroundColor),'rgba(0, 0, 0, 0)');assert.equal(dockSettled.height,64);assert.equal(dockSettled.icon,26);assert.equal(dockSettled.weight,'700');assert.equal(dockSettled.selected,'true');assert.equal(await page.locator('.nav-pill').count(),0);
    data.page = 'home'; await content(); await page.waitForTimeout(700);

    const dockGeometry = await page.evaluate(() => ({mini:document.querySelector('#mini-player').getBoundingClientRect().height, nav:document.querySelector('#navigation').getBoundingClientRect().height}));
    assert.deepEqual(dockGeometry, {mini:68,nav:64});
    // Cover morph: the shared element must interpolate the corner radius instead of
    // switching classes at the end, and the real cover only appears once it lands.
    await page.locator('#mini-player .mini-identity>span:not(.artwork)').click();
    const morph = [];
    for (let i = 0; i < 12; i++) {
      morph.push(await page.evaluate(() => {
        const overlay = document.querySelector('#shared-artwork');
        const cover = document.querySelector('#cover-art');
        return {
          active: !!overlay,
          radius: overlay ? parseFloat(overlay.style.borderRadius) : null,
          scale: overlay ? parseFloat((overlay.style.transform.match(/scale\(([^)]+)\)/) || [])[1]) : null,
          coverOpacity: cover ? Number(getComputedStyle(cover).opacity) : null,
        };
      }));
      await page.waitForTimeout(26);
    }
    await page.waitForTimeout(750);
    assert.equal(await page.locator('.lyric-line').count(),0,'Cover page eagerly mounted the lyric document');
    assert.equal(await page.locator('#player').evaluate(e=>getComputedStyle(e).clipPath),'none');
    const landed = await page.evaluate(() => ({
      overlay: !!document.querySelector('#shared-artwork'),
      radius: getComputedStyle(document.querySelector('#cover-art')).borderRadius,
      coverOpacity: Number(getComputedStyle(document.querySelector('#cover-art')).opacity),
    }));
    assert.ok(morph.some(f => f.active), 'Shared cover overlay never mounted');
    assert.ok(morph.filter(f => f.active).length >= 3, 'Cover morph completed too fast to be continuous');
    assert.ok(morph.filter(f => f.coverOpacity === 0).length >= 3, 'Real cover was visible while the morph was in flight');
    assert.ok(!landed.overlay && Math.abs(landed.coverOpacity - 1) < 0.01, 'Shared cover did not hand over cleanly');
    assert.equal(landed.radius, '28px');
    assert.ok(await page.locator('#player').evaluate(e => e.classList.contains('visible')));
    const opened = await page.locator('#player').boundingBox();
    assert.ok(Math.abs(opened.y) < 1 && Math.abs(opened.width - 411) < 1, 'Player did not expand to window');
    await page.screenshot({ path: path.join(output, 'player-light.png') });
    const reversal=await page.evaluate(async()=>{
      document.querySelector('[data-action=close_player]').click();
      await new Promise(resolve=>setTimeout(resolve,85));
      const overlay=document.querySelector('#shared-artwork');
      const before=overlay?.style.transform;
      auralisUpdate(JSON.stringify({type:'open_player'}));
      return {exists:!!overlay,same:overlay===document.querySelector('#shared-artwork'),continuous:before===overlay?.style.transform};
    });
    assert.deepEqual(reversal,{exists:true,same:true,continuous:true},'Interrupted sheet must retain its layer and position');
    await page.waitForTimeout(800);
    assert.equal(await page.locator('#shared-artwork').count(),0);
    const touch = await page.context().newCDPSession(page);
    const swipeTo = async (direction, expected) => {
      const { x, y, width, height } = await page.locator('#player-pages').boundingBox();
      const start = x + width * (direction === 'left' ? 0.77 : 0.23);
      const end = x + width * (direction === 'left' ? 0.23 : 0.77);
      const touchY = y + height * 0.45;
      await touch.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: start, y: touchY }] });
      for (let step = 1; step <= 8; step++) {
        const touchX = start + (end - start) * step / 8;
        await touch.send('Input.dispatchTouchEvent', { type: 'touchMove', touchPoints: [{ x: touchX, y: touchY }] });
      }
      await touch.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
      await page.waitForTimeout(600);
      assert.ok(await page.locator(`#page-dot-${expected}.selected`).count(), `Swipe ${direction} did not open player page ${expected}`);
    };
    await swipeTo('left', 2);
    assert.equal(await page.locator('.lyric-line').count(),6,'Lyrics must mount on the first lyric-page visit');
    await swipeTo('right', 1);
    await swipeTo('right', 0);
    await swipeTo('left', 1);
    await swipeTo('left', 2);
    await touch.detach();
    assert.ok(await page.locator('#player').evaluate(e => e.classList.contains('lyrics-page')), 'Swipe left did not open lyrics');
    live.position = 6500; await pulse(); await page.waitForTimeout(1300);
    assert.ok(await page.locator('#lyric-2').evaluate(e => e.classList.contains('active')));
    await page.locator('#lyric-viewport').dispatchEvent('pointerdown', { clientX: 150, clientY: 350 });
    await page.locator('#lyric-viewport').evaluate(e => { e.scrollTop = 0; });
    await page.locator('#lyric-viewport').dispatchEvent('pointerup', { clientX: 150, clientY: 355 });
    await page.waitForTimeout(3100);
    const focal = await page.evaluate(() => { const line = document.querySelector('#lyric-2').getBoundingClientRect(); const viewport = document.querySelector('#lyric-viewport').getBoundingClientRect(); return (line.top - viewport.top) / viewport.height; });
    assert.ok(focal > 0.30 && focal < 0.44, `Lyric follow did not recover: ${focal}`);
    await page.screenshot({ path: path.join(output, 'lyrics-light.png') });
    await page.locator('[data-action=close_player]').click(); await page.waitForTimeout(750);

    live.playing = true; await pulse(); await page.waitForTimeout(800);
    assert.ok(await page.locator('#mini-player').evaluate(e => e.classList.contains('expanded')));
    const expandedFits=await page.locator('#mini-player').evaluate(e=>{const box=e.getBoundingClientRect();return [...e.querySelectorAll('.mini-five button')].every(b=>{const r=b.getBoundingClientRect();return r.top>=box.top&&r.bottom<=box.bottom&&r.left>=box.left&&r.right<=box.right;});});
    assert.ok(expandedFits,'Expanded mini-player clips transport/favorite/queue controls');
    await page.screenshot({ path: path.join(output, 'mini-expanded-light.png') });
    await page.waitForTimeout(4400);
    assert.ok(!(await page.locator('#mini-player').evaluate(e => e.classList.contains('expanded'))));

    data.page = 'search'; await content();
    assert.equal(await page.evaluate(() => document.activeElement.id), '');
    await page.locator('#search-input').fill('Rust 音乐');
    await page.locator('#search-input').press('Enter');
    assert.ok(await page.evaluate(() => window.__actions.some(a => a.type === 'search' && a.query === 'Rust 音乐')));
    data.page = 'mine'; await content();
    await page.locator('[data-action="filter:saved"]').click();
    assert.equal(await page.locator('.playlist-row').count(), 1);
    await page.screenshot({ path: path.join(output, 'library-light.png') });

    data.page = 'songs'; data.detailLoading = true; data.tracks = []; data.counts.tracks = 0; await content();
    assert.ok(await page.locator('.loading').isVisible());
    data.detailLoading = false; data.detailError = '连接暂时不可用'; await content();
    assert.ok(await page.locator('[data-action=retry_playlist]').isVisible());
    // A playlist longer than the first page keeps the real total in the header and
    // loads more songs while scrolling instead of stopping at 100. Covers repeat and
    // use identities not seen earlier, so the run still asserts one request per
    // artwork identity instead of tripping over the bounded decoded-image cache.
    const longPlaylist = Array.from({ length: 700 }, (_, i) => ({ ...song(i), coverUrl: `https://example.invalid/art-${700 + (i % 4)}` }));
    data.detailError = ''; data.tracks = longPlaylist.slice(0, 100); data.counts.tracks = 700;
    data.detailTotal = 700; data.detailHasMore = true; data.detailLoadingMore = false; await content();
    assert.ok(await page.locator('#content .song-row').count() === 100, 'Expected the first rendered page');
    assert.ok((await page.locator('#content').innerText()).includes('700 首'), 'Header did not show the real song count');
    assert.equal(await page.locator('.playlist-hero h2').count(),1);
    assert.equal(await page.locator('#header .large-title').evaluate(e=>getComputedStyle(e).display),'none');
    await page.locator('[data-action=detail-artists]').click();
    assert.ok(await page.locator('.artist-row').count()>0,'Playlist artist tab is empty');
    await page.locator('.artist-row').first().click();
    assert.ok(await page.evaluate(()=>window.__actions.some(a=>a.type==='search'&&a.query)));
    await page.locator('[data-action=detail-songs]').click();
    assert.ok((await page.locator('.local-more .more-note').innerText()).includes('已加载 100 / 700 首'));
    await page.waitForTimeout(450);
    await page.screenshot({ path: path.join(output, 'playlist-long.png') });
    await page.locator('[data-action=local_more]').click();
    assert.ok(await page.evaluate(() => window.__actions.some(a => a.type === 'load_more_tracks')), 'Load-more did not request the next page');
    // The mock bridge answers with the next service page.
    data.tracks = longPlaylist.slice(0, 200);
    data.detailHasMore = true; await content();
    await page.locator('#content').evaluate(e => { e.scrollTop = e.scrollHeight; });
    await page.locator('#song-tracks-100').waitFor({ state: 'attached' });
    assert.ok((await page.locator('#content .song-row').count()) > 100, 'Fetched songs were not rendered');
    await page.evaluate(() => { window.__actions.length = 0; });
    await page.locator('[data-action=play_tracks]').click();
    await page.locator('[data-action=shuffle_tracks]').click();
    const played = await page.evaluate(() => window.__actions.filter(a => a.type === 'play_all'));
    assert.equal(played.length, 2);
    assert.equal(played[0].shuffle, false); assert.equal(played[1].shuffle, true);
    // With every page fetched the list ends normally and the scroll controls stay.
    data.tracks = longPlaylist; data.counts.tracks = 700; data.detailHasMore = false; await content();
    await page.locator('#content').evaluate(e => { e.scrollTop = e.scrollHeight; });
    await page.waitForTimeout(150);
    assert.ok((await page.locator('#content .song-row').count()) >= 100);
    await page.locator('[data-action=top]').waitFor({ state: 'visible' });
    assert.ok(await page.locator('[data-action=top]').isVisible());
    assert.ok(await page.locator('[data-action=locate]').isVisible());

    data.page = 'settings'; data.colors = { bg: '#141414', card: '#232323', fg: '#f2f2f2', muted: '#96979b', primary: '#91b6ff', 'on-primary': '#182c4b', border: '#3d3e42' }; await content();
    await page.waitForTimeout(450);assert.equal(await page.locator('#content').evaluate(e=>e.scrollTop),0,'First settings visit must start at the top');await page.screenshot({path:path.join(output,'settings-overview-dark.png')});
    assert.equal(await page.locator('[data-action=logout]').count(), 1);
    assert.ok(await page.locator('[data-action="navigate:ui-customize"]').isVisible(), 'New settings page inherited playlist scroll');
    assert.equal(await page.locator('[data-action="navigate:about"]').count(),1,'Duplicate about entry');
    await page.locator('#content').evaluate(e=>{e.scrollTop=e.scrollHeight;});await page.waitForTimeout(120);
    const collapsed=await page.locator('#header').boundingBox();assert.ok(Math.abs(collapsed.height-80)<1,'Miuix settings title did not collapse');
    await page.locator('[data-action="navigate:about"]').click();const settingsScroll=await page.locator('#content').evaluate(e=>e.scrollTop);
    data.page='about';await content();await page.waitForTimeout(450);
    assert.equal(await page.locator('.about-identity').count(),1);
    assert.equal(await page.locator('.about-logo').evaluate(e=>e.getBoundingClientRect().width),112);
    await page.screenshot({path:path.join(output,'about-dark.png')});
    await page.locator('[data-action="link:project"]').click();
    assert.equal(await page.evaluate(()=>window.__actions.filter(a=>a.type==='open_external').at(-1).url),'https://github.com/Memories-white/Auralis');
    await page.locator('[data-action="copy-project"]').click();assert.ok(await page.evaluate(()=>window.__actions.some(a=>a.type==='copy_project')));
    await page.locator('[data-action="navigate:licenses"]').click();data.page='licenses';await content();await page.waitForTimeout(450);
    assert.equal(await page.locator('.license-document').count(),5);await page.locator('.license-document summary').first().click();assert.ok(await page.locator('.license-document[open] pre').first().isVisible());
    await page.locator('#header [data-action=back]').click();assert.ok(await page.evaluate(()=>window.__actions.some(a=>a.type==='back')));
    data.page='settings';await content();await page.waitForTimeout(450);assert.ok(Math.abs(await page.locator('#content').evaluate(e=>e.scrollTop)-settingsScroll)<3,'Settings scroll was not retained');
    data.page='appearance';await content();await page.waitForTimeout(450);assert.ok(await page.locator('[data-action="navigate:monet"]').isDisabled());
    data.page='settings';await content();await page.waitForTimeout(450);
    await page.screenshot({ path: path.join(output, 'settings-dark.png') });
    data.page = 'ui-customize'; await content();
    assert.ok(await page.locator('[data-action="navigate:ui-dock"]').isVisible(), 'Interface hub lost the dock page');
    await page.locator('[data-action="navigate:ui-dock"]').click();
    // The mock bridge answers navigation by publishing the page Rust asked for.
    data.page = await page.evaluate(() => window.__actions.filter(a => a.type === 'navigate').at(-1).page);
    await content();
    await page.locator('[data-action="ui:bar:floating"]').click();
    assert.equal(await page.locator('#app').getAttribute('data-ui-bar'), 'floating');
    // Mirror the native round-trip: the stored preferences travel back in the next
    // content packet, so apply them before switching pages.
    const applyPrefs = async () => {
      const value = await page.evaluate(() => window.__actions.filter(a => a.type === 'ui_preferences').at(-1)?.value);
      if (value) data.uiPreferences = value;
    };
    await applyPrefs();
    await page.locator('[data-action="ui:compact:on"]').click();
    await applyPrefs();
    data.page = 'ui-player'; await content();
    await page.locator('[data-action="ui:shape:circle"]').click();
    await applyPrefs();
    const saved = { value: data.uiPreferences };
    assert.equal(saved.value.bar, 'floating'); assert.equal(saved.value.compact, 'on');
    await content(); // A native state round-trip must retain the selection.
    assert.equal(await page.locator('#app').getAttribute('data-ui-shape'), 'circle');
    await page.waitForTimeout(450); await page.screenshot({ path: path.join(output, 'customization-dark.png') });
    data.page = 'lyrics-style'; await content();
    assert.ok(await page.locator('[data-action="navigate:lyric-type"]').isVisible(), 'Lyrics hub lost the type page');
    data.page = 'lyric-type'; await content();
    await page.locator('[data-action="ui:lyricSize:36"]').click();
    await page.locator('[data-action="ui:lyricAlign:center"]').click();
    data.page = 'lyric-effect'; await content();
    await page.locator('[data-action="ui:lyricBlur:off"]').click();
    assert.equal(await page.locator('#app').getAttribute('data-ui-lyricblur'), 'off');
    await page.waitForTimeout(450); await page.screenshot({ path: path.join(output, 'lyrics-settings-dark.png') });
    data.uiPreferences = await page.evaluate(() => window.__actions.filter(a => a.type === 'ui_preferences').at(-1).value);
    data.page = 'settings'; data.equalizerSupported=true;data.miSoundSupported=true;await content();
    await page.locator('#mini-player .mini-identity>span:not(.artwork)').click(); await page.waitForTimeout(750);
    await page.locator('#player [data-action=song-actions]').first().click(); await page.waitForTimeout(500);
    const sheet = await page.locator('.song-menu-sheet').boundingBox();
    assert.ok(sheet.height>600&&sheet.height<=714, 'Song menu must use the reference tall grouped layout');
    assert.equal(await page.locator('.sheet [data-action="enqueue-next"]').count(),1);
    assert.equal(await page.locator('.sheet [data-action="enqueue-tail"]').count(),1);
    await page.locator('.sheet [data-action="enqueue-next"]').click();
    await page.locator('.sheet [data-action="enqueue-tail"]').click();
    assert.deepEqual(await page.evaluate(()=>window.__actions.filter(a=>a.type==='enqueue').map(a=>a.next)),[true,false]);
    await page.locator('.sheet [data-action="audio-effects"]').click();
    assert.equal(await page.locator('.effects-sheet [data-action="system-equalizer"]').count(),1);
    const speed=page.locator('#playback-speed');
    await speed.evaluate(e=>{e.value='1.25';});await speed.dispatchEvent('input');await speed.dispatchEvent('change');
    assert.equal(await page.locator('#speed-value').innerText(),'×1.25');
    assert.equal(await page.evaluate(()=>window.__actions.filter(a=>a.type==='audio_settings').at(-1).speed),1.25);
    live.speed=1.25;live.pauseFade=false;await pulse();
    await page.locator('[data-action="toggle-pause-fade"]').click();
    assert.equal(await page.evaluate(()=>window.__actions.filter(a=>a.type==='audio_settings').at(-1).fade),true);
    live.pauseFade=true;await pulse();
    await page.screenshot({path:path.join(output,'audio-effects-dark.png')});
    const darkEffectsPalette=data.colors;
    data.colors={bg:'#f6f6f6',card:'#ffffff',fg:'#191919',muted:'#73757a',primary:'#3482ff','on-primary':'#ffffff',border:'#e7e8eb'};await content();
    await page.waitForTimeout(350);await page.screenshot({path:path.join(output,'audio-effects-light.png')});
    await page.setViewportSize({width:360,height:800});
    assert.ok(await page.locator('.effects-sheet').evaluate(e=>e.scrollWidth<=e.clientWidth),'Audio panel overflows narrow view');
    await page.setViewportSize({width:411,height:914});data.colors=darkEffectsPalette;await content();
    await page.locator('[data-action=sheet_back]').click();
    live.speed=1;await pulse();
    await page.locator('.sheet [data-action=quality]').click();
    assert.ok(await page.locator('.half-sheet [data-action=sheet_back]').isVisible());
    await page.locator('[data-action=sheet_back]').click();
    assert.ok(await page.locator('.sheet [data-action=add-playlist]').isVisible());
    await page.screenshot({ path: path.join(output, 'song-actions-dark.png') });
    const actionsSheet = await page.locator('.sheet').elementHandle();
    await page.locator('.sheet [data-action=add-playlist]').click();
    const playlistSheet = await page.locator('.sheet').elementHandle();
    assert.ok(await actionsSheet.evaluate(e => e.isConnected), 'The playlist chooser must reuse the song actions sheet');
    assert.equal(await page.locator('.sheet [data-action="add:3"]').count(), 0, 'Saved playlists are read-only');
    const addToPlaylist = page.locator('.sheet [data-action="add:1"]');
    await addToPlaylist.click();
    assert.deepEqual(await page.evaluate(() => window.__actions.filter(action => action.type === 'add_cloud')),
      [{ type: 'add_cloud', mid: live.mid, dirid: 1 }]);
    data.collectionMessage = '正在添加';
    data.collectionBusy = true;
    await content();
    assert.ok(await playlistSheet.evaluate(e => e.isConnected), 'Native updates must keep the playlist chooser mounted');
    assert.equal(await page.locator('.sheet .collection-message').textContent(), '正在添加');
    assert.ok(await addToPlaylist.isDisabled(), 'The chooser must prevent repeat submissions while saving');
    Object.assign(data, {collectionBusy: false, collectionMessage: ''});
    await content();
    const closingSheet=await page.locator('#sheet').elementHandle();const handle=page.locator('.sheet-handle');const handleBox=await handle.boundingBox();
    await handle.dispatchEvent('pointerdown',{clientX:handleBox.x+10,clientY:handleBox.y+2,bubbles:true});
    await handle.dispatchEvent('pointermove',{clientX:handleBox.x+10,clientY:handleBox.y+270,bubbles:true});
    await handle.dispatchEvent('pointerup',{clientX:handleBox.x+10,clientY:handleBox.y+270,bubbles:true});
    assert.ok(await closingSheet.evaluate(e=>e.isConnected),'Sheet unmounted before the exit animation');
    await page.waitForTimeout(650);assert.equal(await page.locator('#sheet').count(),0,'Drag did not dismiss the sheet');
    await page.locator('[data-action=close_player]').click();
    await page.waitForTimeout(750);
    await page.setViewportSize({ width: 360, height: 800 });
    data.page = 'ui-customize'; await content();
    assert.equal(await page.locator('#content').evaluate(e => e.scrollWidth <= e.clientWidth), true, 'Appearance page overflows narrow screen');
    data.colors = { bg: '#f6f6f6', card: '#ffffff', fg: '#191919', muted: '#66686c', primary: '#246de0', 'on-primary': '#ffffff', border: '#e0e1e4' }; await content();
    await page.waitForTimeout(450); await page.screenshot({ path: path.join(output, 'customization-light.png') });
    await page.setViewportSize({ width: 900, height: 650 });
    data.page = 'home'; await content();
    await page.screenshot({ path: path.join(output, 'wide-dark.png') });
    await page.setViewportSize({ width: 411, height: 914 });
    data.uiPreferences = { background: 'flow', coverFlip: 'on', lyricSize: '36', lyricControls: 'off' }; await content();
    await page.locator('#mini-player .mini-identity>span:not(.artwork)').click(); await page.waitForTimeout(750);
    live.mid = rows[2].mid; live.playing = true; await pulse(); await page.waitForTimeout(120);
    assert.notEqual(await page.locator('#cover-art').evaluate(e => getComputedStyle(e).transform), 'none', 'Track change has no cover motion');
    const washBefore = await page.locator('.player-wash').evaluate(e => e.style.transform);
    await page.waitForTimeout(200);
    assert.notEqual(await page.locator('.player-wash').evaluate(e => e.style.transform), washBefore, 'Flow background did not advance');
    data.uiPreferences.motion = 'reduced'; await content(); await page.waitForTimeout(100);
    const washReduced = await page.locator('.player-wash').evaluate(e => e.style.transform);
    await page.waitForTimeout(150);
    assert.equal(await page.locator('.player-wash').evaluate(e => e.style.transform), washReduced, 'Reduced motion still animates backdrop');
    await page.locator('[data-action=player_lyrics]').click(); await page.waitForTimeout(150);
    assert.equal(await page.locator('#player .transport').evaluate(e => getComputedStyle(e).display), 'none');
    assert.notEqual(await page.locator('.lyric-progress').evaluate(e => getComputedStyle(e).display), 'none');
    // Reduced motion skips the shared-element morph but must still open/close and
    // leave no mounted player or floating overlay behind.
    await page.locator('[data-action=close_player]').click(); await page.waitForTimeout(500);
    assert.ok(!(await page.locator('#player').evaluate(e => e.classList.contains('mounted'))), 'Reduced motion left the player mounted');
    assert.equal(await page.locator('#shared-artwork').count(), 0);
    await page.locator('#mini-player .mini-identity>span:not(.artwork)').click(); await page.waitForTimeout(500);
    assert.equal(await page.locator('#shared-artwork').count(), 0, 'Reduced motion still mounted the shared morph');
    assert.ok(Math.abs(Number(await page.locator('#cover-art').evaluate(e => getComputedStyle(e).opacity)) - 1) < 0.01);
    // Self-authored timed text: highlight advances within a line, pauses exactly,
    // reverses on seek and never invents timing for ordinary LRC.
    data.colors = { bg:'#151515', card:'#252525', fg:'#f5f5f5', muted:'#92929b', primary:'#91b6ff', 'on-primary':'#182c4b', border:'#3d3e42' };
    data.lyrics = '[00:00.000]Stay with the rhythm of a quieter night\n[00:05.000]Let every word shine\n[00:10.000]Hear the quiet melody';
    data.translation = '[00:00.000]跟随旋律\n[00:05.000]让每个词亮起\n[00:10.000]听见安静的旋律';
    data.wordLyrics = JSON.stringify([
      {time:0,end:5000,text:'Stay with the rhythm of a quieter night',words:[{text:'Stay ',start:0,end:1000},{text:'with ',start:1000,end:2000},{text:'the ',start:2000,end:3000},{text:'rhythm ',start:3000,end:3500},{text:'of ',start:3500,end:3800},{text:'a ',start:3800,end:4000},{text:'quieter ',start:4000,end:4500},{text:'night',start:4500,end:5000}]},
      {time:5000,end:10000,text:'Let every word shine',words:[{text:'Let ',start:5000,end:6000},{text:'every ',start:6000,end:7000},{text:'word ',start:7000,end:8000},{text:'shine',start:8000,end:9500}]}
    ]);
    data.uiPreferences = {lyricAlign:'center',lyricSize:'28',lyricBlur:'on'};
    const restoredLyrics=data.lyrics,restoredWords=data.wordLyrics;
    data.lyrics='';data.wordLyrics='[]';data.lyricState='loading';live.playing=false;
    await content();await pulse();await page.locator('[data-action=player_lyrics]').click();
    assert.ok((await page.locator('.lyric-empty').innerText()).includes('正在加载歌词'));
    data.lyricState='error';await content();
    await page.locator('[data-action=retry_lyrics]').click();
    assert.ok(await page.evaluate(()=>window.__actions.some(a=>a.type==='retry_lyrics')));
    data.lyrics=restoredLyrics;data.wordLyrics=restoredWords;data.lyricState='ready';
    live.position=1500;live.playing=false;await content();await pulse();
    await page.waitForTimeout(800);
    const fills = () => page.locator('#lyric-0 .karaoke-word').evaluateAll(ws => ws.map(w => parseFloat(w.style.getPropertyValue('--word-fill'))));
    let wordFills=await fills();
    assert.equal(await page.locator('.lyric-empty').count(),0,'Paused restored lyrics require pressing play');
    assert.ok(wordFills[0]>=100 && Math.abs(wordFills[1]-50)<1 && wordFills[2]<0, 'Completed/current/future words are not independently lit');
    const spatial=await page.evaluate(()=>[...document.querySelectorAll('#word-0-1 .karaoke-glyph')].map(e=>new DOMMatrix(getComputedStyle(e).transform)) .map(m=>({y:m.m42,scale:m.a})));
    assert.ok(spatial[0].y<-.5&&spatial.at(-1).y===0,'Reveal did not lift the sung letters independently');
    assert.ok(spatial.some(m=>m.scale>1.02),'Long syllable did not swell');
    const track=await page.locator('.lyric-progress .seek').evaluate(e=>({height:getComputedStyle(e,'::before').height,border:getComputedStyle(e,'::before').borderTopWidth}));
    assert.deepEqual(track,{height:'6px',border:'0px'});
    const chrome=await page.evaluate(()=>({progress:document.querySelector('.lyric-progress').getBoundingClientRect().bottom,transport:document.querySelector('#player .transport').getBoundingClientRect().top,tools:document.querySelector('.player-tools').getBoundingClientRect().top}));
    assert.ok(chrome.progress<=chrome.transport&&chrome.transport<chrome.tools,'Reference timeline/transport/tool order is wrong');
    await page.screenshot({path:path.join(output,'karaoke-spatial.png')});
    await page.waitForTimeout(200);assert.deepEqual(await fills(),wordFills,'Paused word highlight advanced');
    live.position=2500;await pulse();await page.waitForTimeout(50);wordFills=await fills();
    assert.ok(wordFills[1]>=100 && Math.abs(wordFills[2]-50)<1,'Highlight did not move to the next word');
    live.position=500;await pulse();await page.waitForTimeout(50);wordFills=await fills();
    assert.ok(Math.abs(wordFills[0]-50)<1 && wordFills[1]<0,'Seeking did not restore word progress');
    live.position=5500;await pulse();await page.waitForTimeout(50);
    assert.ok(await page.locator('#lyric-1').evaluate(e=>e.classList.contains('active')),'Word timeline did not follow the next line');
    live.position=1000;live.playing=true;await pulse();await page.waitForTimeout(50);
    const firstFrame=(await fills())[1];await page.waitForTimeout(70);
    assert.ok((await fills())[1]>firstFrame,'Word highlight did not advance between native playback pulses');
    live.playing=false;
    live.position=1500;await pulse();await page.waitForTimeout(80);
    await page.screenshot({path:path.join(output,'karaoke-progress.png')});
    data.wordLyrics='[]';await content();
    assert.equal(await page.locator('.karaoke-word').count(),0,'Line-only lyrics acquired fabricated word timing');
    assert.ok([...artworkRequests.values()].every(n => n === 1), 'A cached artwork was requested more than once');
    assert.equal(errors.length, 0, errors.join('\n'));
    console.log('PASS: real Rust/WASM boot, safe layout, shelf gutter, one daily entry, Miuix navigation selection, continuous cover morph with matching corner radius, player expansion, native touch swipes through information, cover and lyrics, exact active line, manual-follow recovery, spring mini collapse, search/IME focus, playlist filters/loading/retry, top/locate, settings hub/sub-pages with native round-trip, grouped song menu, audio controls, glyph lift/swell, neutral timeline, unique about/licenses, retained settings scroll, collapsing header, drag dismissal and wide layout.');
    fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify({ result: 'passed', engine: 'real Rust/WASM; mock native service bridge', settings:{uniqueAbout:true,licenses:true,scrollRestoration:true,collapsingHeader:true,dragDismiss:true},audio:{speedMessage:true,fadeMessage:true,narrow:true},karaoke:{glyphLift:true,glyphScale:true,wordFill:true,pause:true,seek:true,betweenPulses:true,lineFallback:true}, artwork: { unique: artworkRequests.size, maximumRequestsPerArtwork: Math.max(...artworkRequests.values()), decodedNodeRetained: true, pageAndPlayerReuse: true }, dock: dockSettled, cover: { frames: morph.filter(f => f.active).length, landed: landed.radius, handedOver: !landed.overlay }, focal, bounds, screenshots: fs.readdirSync(output).filter(n => n.endsWith('.png')) }, null, 2));
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
