// Read-only profile of the cost of one playback pulse while the player is open.
//
// The sheet must never re-measure or re-mount on a pulse: before this change every
// control change re-queried the sheet and read five `offsetTop` values (a forced
// layout per pulse). This probe measures that path with Chromium's own counters.
// Chromium callbacks/CPU metrics are not device GPU frames or power.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');

const root = path.resolve(__dirname, '..');
const assets = path.join(root, 'android/app/src/main/assets/ui');
const output = path.join(root, 'device-debug/rust-ui/motion');
fs.mkdirSync(output, { recursive: true });
const tag = process.argv[2] || 'current';

const song = { mid: 'pulse-fixture', title: 'Pulse study', artist: 'Auralis', coverUrl: 'https://example.invalid/cover', duration: 180 };
const data = {
  page: 'home', currentSong: song, queue: [song], likes: [], recent: [], tracks: [], discovery: [],
  recommendations: [], playlists: [], history: [], artists: [], counts: {},
  safeTop: 24, safeBottom: 16, uiPreferences: {}, lyrics: '[00:00.000]Stay in the light',
  translation: '', wordLyrics: '[]', lyricState: 'ready',
  colors: { bg: '#141414', card: '#242424', fg: '#f2f2f2', muted: '#96979b', primary: '#91b6ff', 'on-primary': '#182c4b', border: '#3d3e42' },
};
const live = { mid: song.mid, title: song.title, artist: song.artist, cover: song.coverUrl, playing: false, position: 6000, duration: 180000 };

(async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 411, height: 914 }, isMobile: true, hasTouch: true });
    await page.addInitScript(() => { window.AuralisNative = { postMessage: () => {} }; });
    await page.route('**/*', (route) => {
      const u = new URL(route.request().url());
      if (u.hostname === 'example.invalid' || u.pathname === '/artwork') {
        return route.fulfill({ contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="500" height="500"><path fill="#8899aa" d="M0 0h500v500H0z"/></svg>' });
      }
      const file = path.resolve(assets, '.' + u.pathname);
      assert.ok(file.startsWith(assets + path.sep));
      return route.fulfill({ contentType: file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.js') ? 'text/javascript' : 'text/html', body: fs.readFileSync(file) });
    });
    await page.goto('https://auralis.local/index.html');
    await page.waitForFunction(() => window.auralisUpdate);
    await page.evaluate(({ data, live }) => {
      auralisUpdate(JSON.stringify({ type: 'content', state: data }));
      auralisUpdate(JSON.stringify({ type: 'playback', state: live }));
    }, { data, live });
    await page.waitForTimeout(320);
    await page.locator('#mini-player .mini-meta').click();
    await page.waitForTimeout(900);
    assert.equal(await page.locator('#player').evaluate((e) => e.classList.contains('mounted')), true);

    const cdp = await page.context().newCDPSession(page);
    await cdp.send('Performance.enable');
    const metrics = async () => Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map((m) => [m.name, m.value]));
    const before = await metrics();
    // 30 play/pause pulses: only the transport/toggle surface changes, the sheet
    // geometry is untouched. Each pulse is processed before the next is sent.
    const pulses = 30;
    for (let i = 0; i < pulses; i++) {
      await page.evaluate((playing) => auralisUpdate(JSON.stringify({
        type: 'playback',
        state: { mid: 'pulse-fixture', title: 'Pulse study', artist: 'Auralis', cover: 'https://example.invalid/cover', playing, position: 6000 + playing * 1000, duration: 180000 },
      })), i % 2 === 0);
    }
    const after = await metrics();
    const delta = {};
    for (const key of ['LayoutCount', 'LayoutDuration', 'RecalcStyleCount', 'RecalcStyleDuration', 'ScriptDuration', 'TaskDuration']) delta[key] = after[key] - before[key];
    const report = {
      tag,
      scope: 'Chromium real WASM, player settled and open; CPU counters around 30 playback pulses. Not device GPU frames or power.',
      pulses,
      delta,
      perPulse: { LayoutCount: delta.LayoutCount / pulses, RecalcStyleCount: delta.RecalcStyleCount / pulses, ScriptDurationMs: (delta.ScriptDuration * 1000) / pulses },
      mountedAfter: await page.locator('#player').evaluate((e) => e.classList.contains('mounted')),
      sheetHeightAfter: await page.locator('#player-surface').evaluate((e) => e.getBoundingClientRect().height),
    };
    fs.writeFileSync(path.join(output, `pulse-${tag}.json`), JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report));
  } finally {
    await browser.close();
  }
})().catch((e) => { console.error(e); process.exitCode = 1; });
