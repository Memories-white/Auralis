// Performance regression for the player sheet. Real WASM in Chromium; the scenario and
// the counters are the same ones tools/profile-player-open.cjs reports on the phone, so
// the numbers are comparable in kind (Chromium main-thread work, not phone GPU frames).
//
// The defect this pins down is concrete: the transition used to resize the container's
// layout box every frame, which produced ~42 layouts / 5.8 ms of layout in the same
// 900 ms window. A fixed window-sized container plus a `clip-path` card and a
// compositor transform must produce (almost) no layout at all.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');

const root = path.resolve(__dirname, '..');
// AURALIS_UI_DIR lets the same tool measure another shipped engine (e.g. the UI assets
// extracted from a previous APK) without touching the working tree.
const assets = process.env.AURALIS_UI_DIR
  ? path.resolve(process.env.AURALIS_UI_DIR)
  : path.join(root, 'android/app/src/main/assets/ui');
const output = path.join(root, 'device-debug/rust-ui/morph');
fs.mkdirSync(output, { recursive: true });
const label = process.argv[2] || 'current';

// Budgets: a fixed container may need one layout for the first open, never one per
// frame. The old implementation was 42 layouts / ~5.8 ms; anything near that fails.
const MAX_LAYOUT_COUNT = 6;
const MAX_LAYOUT_MS = 3.5;

const song = { mid: 'perf-fixture', title: 'Pulse study', artist: 'Auralis', coverUrl: 'https://example.invalid/cover', duration: 180 };
const data = {
  page: 'home', currentSong: song, queue: [song], likes: [], recent: [], tracks: [], discovery: [],
  recommendations: [], playlists: [], history: [], artists: [], counts: {},
  safeTop: 24, safeBottom: 16, uiPreferences: {}, lyrics: '[00:00.000]Stay in the light\n[00:05.000]A quiet return',
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
    await page.waitForTimeout(350);
    // Warm the expensive case exactly like the device profiler: lyrics mounted and
    // retained before the measured opening.
    await page.locator('#mini-player .mini-meta').click();
    await page.waitForTimeout(850);
    await page.locator('[data-action=player_lyrics]').click();
    await page.waitForTimeout(750);
    await page.evaluate(() => auralisBack());
    await page.waitForTimeout(900);

    const session = await page.context().newCDPSession(page);
    await session.send('Performance.enable');
    const report = { tag: label, scenario: 'warm cover opening after lyrics; Chromium main-thread counters, not phone GPU frames', runs: [] };
    for (let i = 0; i < 3; i++) {
      const trace = [];
      const collect = (e) => trace.push(...e.value);
      session.on('Tracing.dataCollected', collect);
      await session.send('Tracing.start', { categories: 'devtools.timeline,blink.user_timing', transferMode: 'ReportEvents' });
      const before = (await session.send('Performance.getMetrics')).metrics;
      const run = await page.evaluate(async () => {
        const frames = [];
        let last = 0;
        await new Promise(requestAnimationFrame);
        const start = performance.now();
        document.querySelector('#mini-player .mini-meta').click();
        await new Promise((resolve) => {
          function frame(t) {
            if (last) frames.push(t - last);
            last = t;
            if (t - start < 900) requestAnimationFrame(frame); else resolve();
          }
          requestAnimationFrame(frame);
        });
        const sorted = [...frames].sort((a, b) => a - b);
        return {
          callbacks: frames.length,
          p50: sorted[Math.floor(sorted.length * 0.5)],
          p95: sorted[Math.floor(sorted.length * 0.95)],
          max: Math.max(...frames),
          over25ms: frames.filter((v) => v > 25).length,
        };
      });
      const after = (await session.send('Performance.getMetrics')).metrics;
      const done = new Promise((resolve) => session.once('Tracing.tracingComplete', resolve));
      await session.send('Tracing.end');
      await done;
      session.off('Tracing.dataCollected', collect);
      run.metrics = {};
      for (const name of ['LayoutCount', 'RecalcStyleCount', 'LayoutDuration', 'RecalcStyleDuration', 'ScriptDuration', 'TaskDuration']) {
        run.metrics[name] = after.find((m) => m.name === name).value - before.find((m) => m.name === name).value;
      }
      run.trace = {};
      for (const e of trace) {
        if (!['Layout', 'UpdateLayoutTree', 'Paint', 'PrePaint', 'FunctionCall'].includes(e.name) || !e.dur) continue;
        const row = (run.trace[e.name] ??= { count: 0, totalMs: 0, maxMs: 0 });
        row.count++;
        row.totalMs += e.dur / 1000;
        row.maxMs = Math.max(row.maxMs, e.dur / 1000);
      }
      report.runs.push(run);
      await page.evaluate(() => auralisBack());
      await page.waitForTimeout(900);
    }
    fs.writeFileSync(path.join(output, `perf-${label}.json`), JSON.stringify(report, null, 2));
    for (const run of report.runs) {
      assert.ok(run.metrics.LayoutCount <= MAX_LAYOUT_COUNT,
        `The sheet still triggers layout per frame: LayoutCount ${run.metrics.LayoutCount} in 900ms (budget ${MAX_LAYOUT_COUNT}); trace Layout ${run.trace.Layout ? run.trace.Layout.count : 0}`);
      assert.ok(run.metrics.LayoutDuration <= MAX_LAYOUT_MS / 1000,
        `LayoutDuration ${(run.metrics.LayoutDuration * 1000).toFixed(2)}ms in 900ms (budget ${MAX_LAYOUT_MS}ms)`);
      // The traced `Layout` events include nested/frame-level layouts, so budget the
      // time they consumed rather than their count.
      if (run.trace.Layout) {
        assert.ok(run.trace.Layout.totalMs <= MAX_LAYOUT_MS,
          `Traced layout work ${run.trace.Layout.totalMs.toFixed(2)}ms in 900ms (budget ${MAX_LAYOUT_MS}ms)`);
      }
    }
    const summary = report.runs.map((r) => ({
      LayoutCount: r.metrics.LayoutCount,
      LayoutMs: +(r.metrics.LayoutDuration * 1000).toFixed(2),
      RecalcStyle: r.metrics.RecalcStyleCount,
      PaintMs: r.trace.Paint ? +r.trace.Paint.totalMs.toFixed(2) : null,
      PaintCount: r.trace.Paint ? r.trace.Paint.count : null,
      TaskMs: +(r.metrics.TaskDuration * 1000).toFixed(1),
      p95: +r.p95.toFixed(1),
      over25ms: r.over25ms,
    }));
    console.log('PASS: warm opening keeps layout at the fixed-container budget.', JSON.stringify(summary));
  } finally {
    await browser.close();
  }
})().catch((e) => { console.error(e); process.exitCode = 1; });
