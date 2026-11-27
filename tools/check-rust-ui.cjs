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
  page: 'home', loggedIn: true, homeReady: true, nickname: 'Auralis Listener', uin: '000000', avatar: 'https://example.invalid/avatar',
  dailyCover: rows[1].coverUrl, discovery: rows.slice(0, 18), tracks: rows, recent: rows.slice(0, 4), favorites: rows.slice(0, 2), likes: rows.slice(0, 24),
  search: [], playlists, recommendations: playlists, queue: rows.slice(0, 8), history: ['周末', '慢慢听'], artists: ['Auralis Studio', '音乐时光'],
  detailTitle: '晚风里的旋律', detailCover: playlists[1].cover, detailLoading: false, detailError: '', query: '', searchedQuery: '', busy: false, status: '',
  discoveryLoading: false, discoveryMore: false, discoveryError: '', colorMode: 'light', monet: false, monetSource: 'wallpaper', monetStyle: 'TonalSpot', monetSpec: 'Spec2021', monetColor: '3482FF',
  quality: 'auto', qualityFallback: true, version: '0.4.0-preview', qrOpen: false, lyrics: '[00:00.00]让旋律慢慢靠近\n[00:03.00]把今天交给音乐\n[00:06.00]下一句，落在心里\n[00:09.00]和你一起慢慢听\n[00:12.00]每一刻都有回响\n[00:15.00]走向更远的地方',
  translation: '', currentSong: rows[1], collectionBusy: false, collectionMessage: '', safeTop: 24, safeBottom: 16,
  colors: { bg: '#f6f6f6', card: '#ffffff', fg: '#191919', muted: '#797b80', primary: '#3482ff', 'on-primary': '#ffffff', border: '#e7e8eb' },
};
data.counts = Object.fromEntries(['discovery', 'tracks', 'recent', 'favorites', 'likes', 'search', 'playlists', 'recommendations', 'queue', 'history', 'artists'].map(key => [key, data[key].length]));
const live = { mid: rows[1].mid, title: rows[1].title, artist: rows[1].artist, cover: rows[1].coverUrl, position: 1000, duration: 180000, playing: false, mode: 'ORDER', actualQuality: 'hq', favorite: false, sleepRemaining: 0, sleepAfterTrack: false, sleepMinutes: 0 };

(async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 411, height: 914 }, deviceScaleFactor: 1, isMobile: true, hasTouch: true });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.addInitScript(() => { window.__actions = []; window.AuralisNative = { postMessage: value => window.__actions.push(JSON.parse(value)) }; });
    await page.route('https://auralis.local/**', async route => {
      const uri = new URL(route.request().url());
      if (uri.pathname === '/artwork') {
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
    await content(); await pulse();
    assert.equal(await page.locator('#navigation button').count(), 3);
    assert.equal(await page.locator('[data-action=daily]').count(), 1);
    assert.equal(await page.locator('[data-action="song:discovery:6"]').count(), 1, 'Shelf songs repeat in discovery rows');
    assert.equal(await page.evaluate(() => document.activeElement.tagName), 'BODY');
    const bounds = await page.evaluate(() => ({ header: document.querySelector('#header').getBoundingClientRect().bottom, content: document.querySelector('#content').getBoundingClientRect().top, shelf: document.querySelector('.shelf-card').getBoundingClientRect().x }));
    assert.ok(bounds.content >= bounds.header - 1, 'Content overlaps header');
    assert.ok(bounds.shelf >= 20, 'Shelf snaps away its leading gutter');
    await page.screenshot({ path: path.join(output, 'home-light.png') });

    await page.locator('#mini-player .mini-identity>span:not(.artwork)').click();
    await page.waitForTimeout(750);
    assert.ok(await page.locator('#player').evaluate(e => e.classList.contains('visible')));
    const opened = await page.locator('#player').boundingBox();
    assert.ok(Math.abs(opened.y) < 1 && Math.abs(opened.width - 411) < 1, 'Player did not expand to window');
    await page.screenshot({ path: path.join(output, 'player-light.png') });
    await page.locator('#player-pages').dispatchEvent('pointerdown', { clientX: 330, clientY: 350 });
    await page.locator('#player-pages').dispatchEvent('pointerup', { clientX: 60, clientY: 352 });
    await page.waitForTimeout(600);
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
    data.detailError = ''; data.tracks = rows; data.counts.tracks = rows.length; await content();
    await page.locator('#content').evaluate(e => { e.scrollTop = e.scrollHeight; });
    await page.locator('[data-action=top]').waitFor({ state: 'visible' });
    assert.ok(await page.locator('[data-action=top]').isVisible());
    assert.ok(await page.locator('[data-action=locate]').isVisible());

    data.page = 'settings'; data.colors = { bg: '#141414', card: '#232323', fg: '#f2f2f2', muted: '#96979b', primary: '#91b6ff', 'on-primary': '#182c4b', border: '#3d3e42' }; await content();
    assert.equal(await page.locator('[data-action=logout]').count(), 1);
    assert.ok(await page.locator('[data-action="navigate:appearance"]').isVisible(), 'New settings page inherited playlist scroll');
    assert.ok(await page.locator('[data-action="navigate:monet"]').isDisabled());
    await page.screenshot({ path: path.join(output, 'settings-dark.png') });
    data.page = 'ui-customize'; await content();
    await page.locator('[data-action="ui:bar:floating"]').click();
    assert.equal(await page.locator('#app').getAttribute('data-ui-bar'), 'floating');
    await page.locator('[data-action="ui:compact:on"]').click();
    await page.locator('[data-action="ui:shape:circle"]').click();
    const saved = await page.evaluate(() => window.__actions.filter(a => a.type === 'ui_preferences').at(-1));
    assert.equal(saved.value.bar, 'floating'); assert.equal(saved.value.compact, 'on');
    data.uiPreferences = saved.value;
    await content(); // A native state round-trip must retain the selection.
    assert.equal(await page.locator('#app').getAttribute('data-ui-shape'), 'circle');
    await page.waitForTimeout(450); await page.screenshot({ path: path.join(output, 'customization-dark.png') });
    data.page = 'lyrics-style'; await content();
    await page.locator('[data-action="ui:lyricSize:36"]').click();
    await page.locator('[data-action="ui:lyricAlign:center"]').click();
    await page.locator('[data-action="ui:lyricBlur:off"]').click();
    assert.equal(await page.locator('#app').getAttribute('data-ui-lyricblur'), 'off');
    await page.waitForTimeout(450); await page.screenshot({ path: path.join(output, 'lyrics-settings-dark.png') });
    data.uiPreferences = await page.evaluate(() => window.__actions.filter(a => a.type === 'ui_preferences').at(-1).value);
    data.page = 'settings'; await content();
    await page.locator('#mini-player .mini-identity>span:not(.artwork)').click(); await page.waitForTimeout(750);
    await page.locator('#player [data-action=song-actions]').first().click(); await page.waitForTimeout(500);
    const sheet = await page.locator('.half-sheet').boundingBox();
    assert.ok(sheet.height <= 458 && sheet.y >= 455, 'Song actions exceed half screen');
    await page.locator('.sheet [data-action=quality]').click();
    assert.ok(await page.locator('.half-sheet [data-action=sheet_back]').isVisible());
    await page.locator('[data-action=sheet_back]').click();
    assert.ok(await page.locator('.sheet [data-action=add-playlist]').isVisible());
    await page.screenshot({ path: path.join(output, 'song-actions-dark.png') });
    await page.locator('.sheet [data-action=close_sheet]').click(); await page.locator('[data-action=close_player]').click();
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
    assert.equal(errors.length, 0, errors.join('\n'));
    console.log('PASS: real Rust/WASM boot, safe layout, shelf gutter, one daily entry, player expansion, left-swipe lyrics, exact active line, manual-follow recovery, automatic mini collapse, search/IME focus, playlist filters/loading/retry, top/locate, settings dependencies, half-height sheet and wide layout.');
    fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify({ result: 'passed', engine: 'real Rust/WASM; mock native service bridge', focal, bounds, screenshots: fs.readdirSync(output).filter(n => n.endsWith('.png')) }, null, 2));
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
