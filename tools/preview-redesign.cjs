// Visual harness for the redesigned dock, cover morph and settings pages.
// Writes frames to device-debug/rust-ui/preview (ignored build output).
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const root = path.resolve(__dirname, '..');
const assets = path.join(root, 'android/app/src/main/assets/ui');
const output = path.join(root, 'device-debug/rust-ui/preview');
fs.mkdirSync(output, { recursive: true });

const song = i => ({ mid: `fixture-${i}`, title: ['Into the Light', '慢慢听', 'A Quiet Morning', 'Somewhere New'][i % 4], artist: ['Auralis Studio', '音乐时光'][i % 2], album: 'Interface Study', coverUrl: `https://example.invalid/art-${i}`, duration: 180, id: i + 1 });
const rows = Array.from({ length: 60 }, (_, i) => song(i));
const playlists = [0, 1, 2].map(i => ({ id: i + 1, dirid: i + 1, title: ['私人收藏', '晚风里的旋律', '周末散步'][i], count: 24 + i, saved: i === 2, cover: `https://example.invalid/list-${i}` }));
const data = {
  page: 'home', loggedIn: true, homeReady: true, nickname: 'Auralis Listener', uin: '000000', avatar: 'https://example.invalid/avatar',
  dailyCover: rows[1].coverUrl, discovery: rows.slice(0, 18), tracks: rows, recent: rows.slice(0, 4), favorites: rows.slice(0, 2), likes: rows.slice(0, 24),
  search: [], playlists, recommendations: playlists, queue: rows.slice(0, 8), history: ['周末', '慢慢听'], artists: ['Auralis Studio', '音乐时光'],
  detailTitle: '晚风里的旋律', detailCover: playlists[1].cover, detailLoading: false, detailError: '', query: '', searchedQuery: '', busy: false, status: '',
  discoveryLoading: false, discoveryMore: false, discoveryError: '', colorMode: 'light', monet: false, monetSource: 'wallpaper', monetStyle: 'TonalSpot', monetSpec: 'Spec2021', monetColor: '3482FF',
  quality: 'auto', qualityFallback: true, version: '0.5.0-preview', qrOpen: false,
  lyrics: '[00:00.00]让旋律慢慢靠近\n[00:03.00]把今天交给音乐\n[00:06.00]下一句，落在心里\n[00:09.00]和你一起慢慢听\n[00:12.00]每一刻都有回响\n[00:15.00]走向更远的地方',
  translation: '', currentSong: rows[1], collectionBusy: false, collectionMessage: '', safeTop: 24, safeBottom: 16,
  colors: { bg: '#f6f6f6', card: '#ffffff', fg: '#191919', muted: '#797b80', primary: '#3482ff', 'on-primary': '#ffffff', border: '#e7e8eb' },
};
data.counts = Object.fromEntries(['discovery', 'tracks', 'recent', 'favorites', 'likes', 'search', 'playlists', 'recommendations', 'queue', 'history', 'artists'].map(key => [key, data[key].length]));
const live = { mid: rows[1].mid, title: rows[1].title, artist: rows[1].artist, cover: rows[1].coverUrl, position: 1000, duration: 180000, playing: false, mode: 'ORDER', actualQuality: 'hq', favorite: false, sleepRemaining: 0, sleepAfterTrack: false, sleepMinutes: 0 };

(async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 411, height: 914 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
    await page.addInitScript(() => { window.__actions = []; window.AuralisNative = { postMessage: v => window.__actions.push(JSON.parse(v)) }; });
    await page.route('https://auralis.local/**', async route => {
      const uri = new URL(route.request().url());
      if (uri.pathname === '/artwork') {
        const seed = [...(uri.searchParams.get('url') || '')].reduce((n, c) => n + c.charCodeAt(0), 0) % 360;
        const body = `<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><defs><linearGradient id="a" x2="1" y2="1"><stop stop-color="hsl(${seed} 32% 66%)"/><stop offset="1" stop-color="hsl(${seed + 24} 38% 28%)"/></linearGradient></defs><path fill="url(#a)" d="M0 0h512v512H0z"/><circle cx="256" cy="256" r="150" fill="none" stroke="#ffffff44"/><path d="M210 303V186l110-22v116" fill="none" stroke="#fff9" stroke-width="10"/><circle cx="188" cy="305" r="25" fill="#fff9"/><circle cx="298" cy="282" r="25" fill="#fff9"/></svg>`;
        return route.fulfill({ contentType: 'image/svg+xml', body });
      }
      const file = path.resolve(assets, '.' + uri.pathname);
      const type = file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.js') ? 'text/javascript' : 'text/html';
      return route.fulfill({ contentType: type, body: fs.readFileSync(file) });
    });
    await page.goto('https://auralis.local/index.html');
    await page.waitForFunction(() => typeof window.auralisUpdate === 'function');
    const content = async () => page.evaluate(input => window.auralisUpdate(JSON.stringify({ type: 'content', state: input })), data);
    const pulse = async () => page.evaluate(input => window.auralisUpdate(JSON.stringify({ type: 'playback', state: input })), live);
    await content(); await pulse();
    await page.waitForFunction(() => [...document.querySelectorAll('img[data-art-src]')].every(img => img.complete && img.naturalWidth > 0));
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'home.png') });

    // Dock indicator travel: sample the pill position while navigating.
    await page.locator('#navigation [data-action="navigate:mine"]').click();
    const track = [];
    for (let i = 0; i < 12; i++) {
      track.push(await page.evaluate(() => {
        const nav = document.querySelector('#navigation');
        const pill = document.querySelector('.nav-pill');
        return { index: getComputedStyle(nav).getPropertyValue('--nav-index').trim(), transform: getComputedStyle(pill).transform };
      }));
      await page.waitForTimeout(16);
    }
    await page.waitForTimeout(600);
    await page.screenshot({ path: path.join(output, 'library.png') });
    data.page = 'mine'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'library2.png') });

    // Player open morph: capture intermediate frames and the shared radius.
    await page.locator('#navigation [data-action="navigate:home"]').click();
    data.page = 'home'; await content(); await page.waitForTimeout(700);
    await page.locator('#mini-player .mini-identity .mini-meta').click();
    const frames = [];
    for (let i = 0; i < 10; i++) {
      frames.push(await page.evaluate(() => {
        const overlay = document.querySelector('#shared-artwork');
        const cover = document.querySelector('#cover-art');
        return {
          overlay: overlay ? { radius: overlay.style.borderRadius, transform: overlay.style.transform, w: overlay.style.width } : null,
          coverOpacity: cover ? getComputedStyle(cover).opacity : null,
          playerClip: getComputedStyle(document.querySelector('#player')).clipPath,
        };
      }));
      await page.waitForTimeout(28);
    }
    await page.waitForTimeout(700);
    await page.screenshot({ path: path.join(output, 'player-cover.png') });
    const settled = await page.evaluate(() => ({
      overlay: !!document.querySelector('#shared-artwork'),
      radius: getComputedStyle(document.querySelector('#cover-art')).borderRadius,
      coverOpacity: getComputedStyle(document.querySelector('#cover-art')).opacity,
    }));
    await page.locator('[data-action="player_lyrics"]').click();
    await page.waitForTimeout(600);
    await page.screenshot({ path: path.join(output, 'player-lyrics.png') });
    await page.locator('.identity-open').click();
    await page.waitForTimeout(700);
    await page.screenshot({ path: path.join(output, 'player-cover-return.png') });
    await page.locator('[data-action="close_player"]').click();
    await page.waitForTimeout(900);
    await page.screenshot({ path: path.join(output, 'closed.png') });

    // Mini player expansion frames.
    live.playing = true; await pulse();
    const miniFrames = [];
    for (let i = 0; i < 10; i++) {
      miniFrames.push(await page.evaluate(() => {
        const mini = document.querySelector('#mini-player');
        return { h: mini.getBoundingClientRect().height, v: getComputedStyle(mini).getPropertyValue('--mini').trim() };
      }));
      await page.waitForTimeout(30);
    }
    await page.waitForTimeout(600);
    await page.screenshot({ path: path.join(output, 'mini-expanded.png') });
    live.playing = false; await pulse(); await page.waitForTimeout(5200);

    // Settings pages.
    data.page = 'settings'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'settings.png') });
    data.page = 'ui-customize'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'settings-ui.png') });
    data.page = 'ui-dock'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'settings-dock.png') });
    data.page = 'ui-player'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'settings-player.png') });
    data.page = 'lyrics-style'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'settings-lyrics.png') });
    data.page = 'lyric-type'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'settings-lyric-type.png') });
    // Option application
    await page.locator('[data-action="ui:lyricSize:36"]').click();
    await page.waitForTimeout(400);
    const applied = await page.evaluate(() => document.querySelector('#app').style.getPropertyValue('--lyric-size'));

    // Bar variants
    for (const bar of ['normal', 'floating']) {
      data.uiPreferences = { ...(data.uiPreferences || {}), bar };
      data.page = 'home'; await content(); await page.waitForTimeout(600);
      await page.screenshot({ path: path.join(output, `home-bar-${bar}.png`) });
    }
    data.uiPreferences = { bar: 'glass' }; data.page = 'home'; await content();
    // Dark palette
    const light = data.colors;
    data.colors = { bg: '#141414', card: '#232323', fg: '#f2f2f2', muted: '#96979b', primary: '#91b6ff', 'on-primary': '#182c4b', border: '#3d3e42' };
    await content(); await page.waitForTimeout(600);
    await page.screenshot({ path: path.join(output, 'home-dark.png') });
    data.page = 'settings'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'settings-dark.png') });
    data.page = 'ui-player'; await content(); await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(output, 'settings-player-dark.png') });
    data.colors = light;
    // Narrow screen overflow check
    await page.setViewportSize({ width: 360, height: 780 });
    data.page = 'settings'; await content(); await page.waitForTimeout(500);
    const overflow = await page.evaluate(() => document.querySelector('#content').scrollWidth - document.querySelector('#content').clientWidth);
    await page.screenshot({ path: path.join(output, 'settings-narrow.png') });
    await page.setViewportSize({ width: 411, height: 914 });

    console.log(JSON.stringify({ errors, track: track.slice(0, 6), frames: frames.slice(0, 6), settled, miniFrames, applied, overflow }, null, 1));
  } finally { await browser.close(); }
})().catch(e => { console.error(e); process.exitCode = 1; });
