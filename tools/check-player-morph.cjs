// Structural regression for the player sheet morph (real WASM, Chromium).
//
// The user-visible contract enforced here is geometric, not DOM-existence:
//   * one shared container grows from the song bar rect to the window rect, so the
//     card is never the whole screen while it is still small ("mask" look);
//   * the full player page keeps its final layout box and is uniformly scaled by the
//     container width, anchored at the container origin and clipped by it, so no
//     section is re-positioned, compressed or overlapped;
//   * the shared cover keeps one continuous rect/radius track and hands over exactly;
//   * the song title is never painted twice at once and never disappears;
//   * controls never flash off again once revealed;
//   * closing returns to the song bar and a reversal continues from the rendered pose.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const { CARD_RECT_SOURCE } = require('./sheet-geometry.cjs');

const root = path.resolve(__dirname, '..');
// AURALIS_UI_DIR lets the same tool measure another shipped engine (e.g. the UI assets
// extracted from a previous APK) without touching the working tree.
const assets = process.env.AURALIS_UI_DIR
  ? path.resolve(process.env.AURALIS_UI_DIR)
  : path.join(root, 'android/app/src/main/assets/ui');
const output = path.join(root, 'device-debug/rust-ui/morph');
fs.mkdirSync(output, { recursive: true });

const song = { mid: 'morph-fixture', title: 'A quiet return', artist: 'Auralis study', coverUrl: 'https://example.invalid/cover', duration: 180 };
const data = {
  page: 'home', currentSong: song, queue: [song], likes: [], recent: [], tracks: [], discovery: [],
  recommendations: [], playlists: [], history: [], artists: [], counts: {},
  safeTop: 24, safeBottom: 16, uiPreferences: {}, lyrics: '[00:00.000]Stay in the light\n[00:05.000]A quiet return',
  translation: '', wordLyrics: '[]', lyricState: 'ready',
  colors: { bg: '#141414', card: '#242424', fg: '#f2f2f2', muted: '#96979b', primary: '#91b6ff', 'on-primary': '#182c4b', border: '#3d3e42' },
};
const live = { mid: song.mid, title: song.title, artist: song.artist, cover: song.coverUrl, playing: false, position: 6000, duration: 180000 };

const W0 = 411, H0 = 914;
const SECTIONS = ['.player-toolbar', '.player-identity', '#player-pages', '.page-dots', '.player-footer'];

// One geometry frame per animation callback. Every read is batched per frame and
// nothing is written, so the sampler cannot perturb the measured motion.
const SAMPLE = `
window.__sampleMorph = (ms) => new Promise((resolve) => {
  const box = (e) => { const r = e.getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height, right: r.right, bottom: r.bottom }; };
  const num = (e, k) => Number(getComputedStyle(e)[k]);
  const alpha = (e) => {
    let a = 1, n = e;
    while (n && n !== document.documentElement) {
      const cs = getComputedStyle(n);
      if (cs.display === 'none' || cs.visibility === 'hidden') return 0;
      a *= Number(cs.opacity);
      n = n.parentElement;
    }
    return a;
  };
  const clipped = (e) => {
    const r = box(e); let l = r.x, t = r.y, rt = r.right, bt = r.bottom;
    let n = e.parentElement;
    // body/html are degenerate in this app (the shell is a fixed #app), so only real
    // clipping boxes with an area participate.
    while (n && n !== document.body && n !== document.documentElement) {
      const cs = getComputedStyle(n);
      const c = box(n);
      if ((cs.overflowX !== 'visible' || cs.overflowY !== 'visible') && c.w > 0 && c.h > 0) {
        l = Math.max(l, c.x); t = Math.max(t, c.y); rt = Math.min(rt, c.right); bt = Math.min(bt, c.bottom);
      }
      n = n.parentElement;
    }
    const w = Math.max(0, rt - l), h = Math.max(0, bt - t);
    return r.w * r.h < 1 ? 0 : (w * h) / (r.w * r.h);
  };
  const eff = (e) => (e ? alpha(e) * clipped(e) : 0);
  const frames = [];
  const start = performance.now();
  const tick = () => {
    const player = document.querySelector('#player');
    const surface = document.querySelector('#player-surface');
    const pageEl = document.querySelector('#player .player-safe');
    const overlay = document.querySelector('#shared-artwork');
    const cover = document.querySelector('#cover-art');
    const mini = document.querySelector('#mini-player');
    frames.push({
      t: performance.now() - start,
      mounted: !!player && player.classList.contains('mounted'),
      card: window.__auralisCard(),
      surfaceAlpha: surface ? num(surface, 'opacity') : null,
      page: pageEl ? box(pageEl) : null,
      pageAlpha: pageEl ? num(pageEl, 'opacity') : null,
      pageLayoutW: pageEl ? parseFloat(getComputedStyle(pageEl).width) : null,
      pageLayoutH: pageEl ? parseFloat(getComputedStyle(pageEl).height) : null,
      overlay: overlay ? { ...box(overlay), radius: parseFloat(overlay.style.borderRadius) || 0 } : null,
      cover: cover ? { ...box(cover), opacity: num(cover, 'opacity'), radius: parseFloat(getComputedStyle(cover).borderRadius) || 0 } : null,
      mini: mini ? { ...box(mini), opacity: num(mini, 'opacity'), transform: getComputedStyle(mini).transform } : null,
      titleMini: eff(document.querySelector('#mini-player .mini-meta strong')),
      titlePage: eff(document.querySelector('#player .player-identity h2')),
      transportMini: eff(document.querySelector('#mini-player .mini-compact')),
      transportPage: eff(document.querySelector('#player .player-footer .transport')),
      sections: ${JSON.stringify(SECTIONS)}.map((s) => {
        const e = document.querySelector('#player ' + s);
        return e ? { s, layoutTop: e.offsetTop, ...box(e) } : null;
      }),
    });
    if (performance.now() - start < ms) requestAnimationFrame(tick); else resolve(frames);
  };
  requestAnimationFrame(tick);
});
`;

const near = (a, b, tol) => Math.abs(a - b) <= tol;
const biggestStep = (values) => {
  let worst = 0;
  for (let i = 1; i < values.length; i++) {
    if (values[i] == null || values[i - 1] == null) continue;
    const d = values[i] - values[i - 1];
    if (Math.abs(d) > Math.abs(worst)) worst = d;
  }
  return worst;
};

(async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: W0, height: H0 }, isMobile: true, hasTouch: true });
    await page.addInitScript(() => { window.__actions = []; window.AuralisNative = { postMessage: (m) => window.__actions.push(JSON.parse(m)) }; });
    await page.route('**/*', (route) => {
      const u = new URL(route.request().url());
      if (u.hostname === 'example.invalid' || u.pathname === '/artwork') {
        return route.fulfill({ contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="500" height="500"><path fill="#e5ddd0" d="M0 0h500v500H0z"/><path fill="#253c61" d="M0 300h500v200H0z"/></svg>' });
      }
      const file = path.resolve(assets, '.' + u.pathname);
      assert.ok(file.startsWith(assets + path.sep));
      return route.fulfill({ contentType: file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.js') ? 'text/javascript' : 'text/html', body: fs.readFileSync(file) });
    });
    await page.goto('https://auralis.local/index.html');
    await page.waitForFunction(() => window.auralisUpdate);
    // The page CSP forbids injected <script>; CDP evaluation is the supported path here.
    await page.evaluate(CARD_RECT_SOURCE + '; true;');
    await page.evaluate(SAMPLE + '; true;');
    const content = () => page.evaluate((d) => auralisUpdate(JSON.stringify({ type: 'content', state: d })), data);
    const pulse = () => page.evaluate((l) => auralisUpdate(JSON.stringify({ type: 'playback', state: l })), live);
    await content(); await pulse();
    await page.waitForTimeout(320);
    const rest = await page.evaluate(() => {
      const r = (s) => { const b = document.querySelector(s).getBoundingClientRect(); return { x: b.x, y: b.y, w: b.width, h: b.height }; };
      const mini = document.querySelector('#mini-player');
      return {
        mini: r('#mini-player'),
        art: r('#mini-player .mini-art'),
        radius: parseFloat(getComputedStyle(mini).borderTopLeftRadius) || 0,
      };
    });

    // ---------- open ----------
    const open = await page.evaluate(async () => {
      document.querySelector('#mini-player .mini-meta').click();
      return window.__sampleMorph(1300);
    });
    fs.writeFileSync(path.join(output, 'open-frames.json'), JSON.stringify(open, null, 1));

    const mid = open.filter((f) => f.mounted && f.card && f.card.h > rest.mini.h + 1 && f.card.h < H0 - 1);
    assert.ok(mid.length >= 6,
      `No growing container was seen: the sheet is the whole window for the entire flight (${mid.length}/${open.length} frames between the bar and the window)`);

    // (1) Shared container: starts as the song bar, grows into the window.
    assert.ok(Math.min(...mid.map((f) => f.card.w)) <= rest.mini.w + (W0 - rest.mini.w) * 0.5,
      'Container is already window-wide early in the flight (whole-screen mask, not a card growing from the bar)');
    assert.ok(Math.max(...mid.map((f) => f.card.y)) >= rest.mini.y * 0.4,
      'Container top never starts near the song bar');
    const last = mid[mid.length - 1];
    assert.ok(near(last.card.w, W0, 1.5) && near(last.card.h, H0, 1.5), 'Container did not reach the window rect');

    // (2) Uniform width scale anchored at the container origin, clipped by it.
    for (const f of mid) {
      assert.ok(near(f.page.w, f.card.w, 1.6),
        `Page width ${f.page.w.toFixed(1)}px != container width ${f.card.w.toFixed(1)}px: the page is squashed instead of uniformly scaled`);
      assert.ok(near(f.page.x, f.card.x, 1.6), `Page left ${f.page.x.toFixed(1)} != container left ${f.card.x.toFixed(1)}`);
      assert.ok(near(f.page.y, f.card.y, 1.6), `Page top ${f.page.y.toFixed(1)} != container top ${f.card.y.toFixed(1)}`);
      assert.ok(f.page.bottom >= f.card.bottom - 1.6, 'Page detached from the container bottom');
      // The song bar must ride the card's top-left corner for the whole flight, so the
      // card reads as growing out of the bar. A CSS transition on the bar's transform
      // makes it lag behind the card even though the end poses match.
      assert.ok(near(f.mini.x, f.card.x, 2.0),
        `Song bar lagged the card horizontally by ${(f.mini.x - f.card.x).toFixed(1)}px at container ${f.card.h.toFixed(0)}px`);
      assert.ok(near(f.mini.y, f.card.y, 2.0),
        `Song bar lagged the card vertically by ${(f.mini.y - f.card.y).toFixed(1)}px at container ${f.card.h.toFixed(0)}px`);
    }

    // (3) Rigid page: every section keeps its layout offset scaled by the page scale,
    // and consecutive sections never overlap.
    const settledSections = open.filter((f) => f.mounted && f.sections.every(Boolean)).at(-1).sections;
    for (const f of mid) {
      const s = f.page.w / f.pageLayoutW;
      const secs = f.sections.filter(Boolean);
      for (let i = 0; i < secs.length - 1; i++) {
        assert.ok(secs[i].bottom <= secs[i + 1].y + 0.75,
          `Sections "${secs[i].s}" and "${secs[i + 1].s}" overlap by ${(secs[i].bottom - secs[i + 1].y).toFixed(1)}px at container ${f.card.h.toFixed(0)}px`);
        const layoutGap = settledSections[i + 1].layoutTop - settledSections[i].layoutTop;
        const renderedGap = secs[i + 1].y - secs[i].y;
        assert.ok(near(renderedGap, layoutGap * s, Math.max(1.2, layoutGap * 0.02)),
          `Gap "${secs[i].s}"->"${secs[i + 1].s}" is ${renderedGap.toFixed(1)}px, expected ${(layoutGap * s).toFixed(1)}px from page scale ${s.toFixed(4)} (section spacing compressed/expanded)`);
      }
    }

    // (4) No teleport, no reversal, continuous cover.
    assert.ok(near(mid[0].card.radius, rest.radius, 2.5),
      `Card corner radius ${mid[0].card.radius.toFixed(1)} does not start on the song bar radius ${rest.radius.toFixed(1)}`);
    assert.ok(mid[mid.length - 1].card.radius <= 2.5, `Card corner radius ${mid[mid.length - 1].card.radius.toFixed(1)} did not reach the window`);
    assert.ok(mid.every((f, i) => i === 0 || f.card.radius <= mid[i - 1].card.radius + 0.35), 'Card corner radius grew while opening');
    assert.ok(Math.abs(biggestStep(mid.map((f) => f.card.h))) <= 0.34 * (H0 - rest.mini.h), 'Container height teleported in one frame');
    assert.ok(mid.every((f, i) => i === 0 || f.card.h >= mid[i - 1].card.h - 0.6), 'Container height reversed while opening');
    assert.ok(mid.every((f) => f.overlay), 'Shared cover overlay was unmounted mid-flight');
    const coverArea = mid.map((f) => f.overlay.w * f.overlay.h);
    assert.ok(coverArea.every((a, i) => i === 0 || a >= coverArea[i - 1] - 1), 'Shared cover shrank while opening');
    assert.ok(Math.abs(biggestStep(mid.map((f) => f.overlay.w))) <= 0.4 * (coverArea.length ? mid[mid.length - 1].overlay.w - mid[0].overlay.w + 1 : 1),
      'Shared cover size teleported');

    // (5) Title never duplicated, never blank.
    for (const f of mid) assert.ok(Math.max(f.titleMini, f.titlePage) > 0.1, `No song title visible at container ${f.card.h.toFixed(0)}px`);
    assert.equal(mid.filter((f) => f.titleMini > 0.5 && f.titlePage > 0.5).length, 0, 'Two opaque song titles at once');

    // (6) Each control group is monotone: the song bar's controls fade out exactly
    // once, and the page's controls are revealed by the moving clip. Neither may
    // come back after leaving, and neither may pop in a single frame.
    const barControls = mid.map((f) => ({ v: f.transportMini, y: f.sections[0].y }));
    const pageControls = mid.map((f) => ({ v: f.transportPage, y: f.sections[4].y }));
    assert.ok(barControls[0].v > 0.9, 'Song bar controls are not fully visible when the card starts on the bar');
    assert.ok(pageControls.at(-1).v > 0.9, 'Page controls are not fully visible once the card has landed');
    for (const [name, track] of [['song bar', barControls], ['page', pageControls]]) {
      for (let i = 1; i < track.length; i++) {
        assert.ok(Math.abs(track[i].v - track[i - 1].v) <= 0.5, `${name} controls popped in one frame`);
      }
      if (name === 'song bar') {
        assert.ok(track.every((c, i) => i === 0 || c.v <= track[i - 1].v + 0.02), 'Song bar controls came back while handing off');
      } else {
        assert.ok(track.every((c, i) => i === 0 || c.v >= track[i - 1].v - 0.02), 'Page controls flickered back to invisible');
        assert.ok(track.some((c) => c.v > 0.05 && c.v < 0.9), 'Page controls were never partially revealed by the clip');
      }
    }
    // The footer band itself travels as part of the rigid page.
    const travel = Math.abs(pageControls.at(-1).y - pageControls[0].y) + 1;
    for (let i = 1; i < pageControls.length; i++) {
      assert.ok(Math.abs(pageControls[i].y - pageControls[i - 1].y) <= 0.4 * travel,
        `Footer control band teleported ${(pageControls[i].y - pageControls[i - 1].y).toFixed(1)}px in one frame`);
    }

    // ---------- settled ----------
    await page.waitForTimeout(700);
    const landed = await page.evaluate(() => {
      const cover = document.querySelector('#cover-art');
      const b = cover.getBoundingClientRect();
      const p = document.querySelector('#player').getBoundingClientRect();
      return {
        mounted: document.querySelector('#player').classList.contains('mounted'),
        overlay: !!document.querySelector('#shared-artwork'),
        cover: { x: b.x, y: b.y, w: b.width, h: b.height, opacity: Number(getComputedStyle(cover).opacity), radius: getComputedStyle(cover).borderRadius },
        player: { x: p.x, y: p.y, w: p.width, h: p.height },
        clipPath: getComputedStyle(document.querySelector('#player')).clipPath,
      };
    });
    assert.equal(landed.mounted, true);
    assert.equal(landed.overlay, false, 'Shared cover overlay leaked after settling');
    assert.ok(near(landed.cover.opacity, 1, 0.01), 'Real cover did not become opaque');
    assert.ok(near(landed.player.y, 0, 1) && near(landed.player.w, W0, 1) && near(landed.player.h, H0, 1), 'Settled player is not the window rect');
    assert.equal(landed.clipPath, 'none');
    assert.equal(landed.cover.radius, '28px');
    const lastOverlay = open.filter((f) => f.overlay).at(-1);
    assert.ok(near(lastOverlay.overlay.x, landed.cover.x, 2) && near(lastOverlay.overlay.y, landed.cover.y, 2) && near(lastOverlay.overlay.w, landed.cover.w, 2),
      'Cover hand-over moved the artwork');

    // ---------- a track change while open must not blank the replaced subtree ----------
    const swapped = await page.evaluate(async () => {
      auralisUpdate(JSON.stringify({
        type: 'playback',
        state: { mid: 'morph-fixture-2', title: 'Another return', artist: 'Auralis study', cover: 'https://example.invalid/cover2', playing: false, position: 0, duration: 180000 },
      }));
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
      await new Promise((r) => setTimeout(r, 260));
      const page = document.querySelector('#player .player-safe');
      const b = page.getBoundingClientRect();
      return {
        opacity: Number(getComputedStyle(page).opacity),
        transform: getComputedStyle(page).transform,
        box: { x: b.x, y: b.y, w: b.width, h: b.height },
        title: document.querySelector('#player .player-identity h2').textContent,
        sheet: (() => { const r = document.querySelector('#player-sheet').getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; })(),
      };
    });
    assert.ok(near(swapped.opacity, 1, 0.02), `Replacing the player subtree blanked the open page (opacity ${swapped.opacity})`);
    assert.equal(swapped.transform, 'none');
    assert.ok(near(swapped.box.w, W0, 1) && near(swapped.box.h, H0, 1), 'Replaced page lost its settled layout box');
    assert.ok(near(swapped.sheet.w, W0, 1) && near(swapped.sheet.h, H0, 1) && near(swapped.sheet.y, 0, 1), 'Track change collapsed the settled card');
    assert.ok(swapped.title.includes('Another return'), `Track change did not reach the title (${swapped.title})`);
    await page.waitForTimeout(120);

    // ---------- close returns to the song bar ----------
    const close = await page.evaluate(async () => {
      document.querySelector('[data-action=close_player]').click();
      return window.__sampleMorph(1300);
    });
    fs.writeFileSync(path.join(output, 'close-frames.json'), JSON.stringify(close, null, 1));
    const closing = close.filter((f) => f.mounted && f.card && f.card.h > rest.mini.h + 1 && f.card.h < H0 - 1);
    assert.ok(closing.length >= 6, 'Closing motion window not sampled');
    assert.ok(closing.every((f) => f.overlay), 'Shared cover dropped while closing');
    for (let i = 1; i < closing.length; i++) {
      assert.ok(closing[i].card.h <= closing[i - 1].card.h + 0.6, 'Container height increased while closing');
      assert.ok(closing[i].card.y >= closing[i - 1].card.y - 0.6, 'Container moved up while closing');
    }
    // The card must land on the song bar, not on the window origin, and the shared
    // cover must hand back onto the bar artwork without a last-frame jump.
    const landing = closing.at(-1);
    assert.ok(near(landing.card.x, rest.mini.x, 2) && near(landing.card.y, rest.mini.y, 2),
      `Closing card landed at (${landing.card.x.toFixed(1)},${landing.card.y.toFixed(1)}) instead of the song bar (${rest.mini.x},${rest.mini.y})`);
    assert.ok(near(landing.card.w, rest.mini.w, 2) && near(landing.card.h, rest.mini.h, 2),
      `Closing card size ${landing.card.w.toFixed(1)}x${landing.card.h.toFixed(1)} did not converge to the song bar ${rest.mini.w}x${rest.mini.h}`);
    assert.ok(near(landing.card.radius, rest.radius, 2.5),
      `Closing card corner radius ${landing.card.radius.toFixed(1)} did not return to the song bar radius ${rest.radius.toFixed(1)}`);
    const handback = closing.filter((f) => f.overlay).at(-1).overlay;
    assert.ok(near(handback.x, rest.art.x, 2.5) && near(handback.y, rest.art.y, 2.5) && near(handback.w, rest.art.w, 2.5),
      `Closing cover landed at (${handback.x.toFixed(1)},${handback.y.toFixed(1)},${handback.w.toFixed(1)}) instead of the song bar artwork (${rest.art.x},${rest.art.y},${rest.art.w})`);
    const returned = await page.evaluate(() => {
      const mini = document.querySelector('#mini-player');
      const b = mini.getBoundingClientRect();
      return {
        mounted: document.querySelector('#player').classList.contains('mounted'),
        overlay: !!document.querySelector('#shared-artwork'),
        miniOpacity: Number(getComputedStyle(mini).opacity),
        miniTransform: getComputedStyle(mini).transform,
        miniBox: { x: b.x, y: b.y, w: b.width, h: b.height },
        artVisible: getComputedStyle(document.querySelector('#mini-player .mini-art')).visibility,
        pageTransform: getComputedStyle(document.querySelector('#player .player-safe')).transform,
        overlayZ: getComputedStyle(document.querySelector('#player')).visibility,
      };
    });
    assert.equal(returned.mounted, false, 'Closing left the player mounted');
    assert.equal(returned.overlay, false, 'Closing left the shared cover mounted');
    assert.ok(near(returned.miniOpacity, 1, 0.01), `Mini player opacity ${returned.miniOpacity} did not return to 1`);
    assert.equal(returned.miniTransform, 'none', 'Mini player kept a handoff transform');
    assert.ok(near(returned.miniBox.x, rest.mini.x, 1) && near(returned.miniBox.y, rest.mini.y, 1) && near(returned.miniBox.w, rest.mini.w, 1) && near(returned.miniBox.h, rest.mini.h, 1),
      'Mini player did not return to its resting rect');
    assert.equal(returned.artVisible, 'visible', 'Mini cover stayed hidden after returning');
    assert.equal(returned.pageTransform, 'none', 'Player page kept a residual scale');

    // ---------- interruption continues from the rendered pose ----------
    const reversal = await page.evaluate(async () => {
      const card = () => { const c = window.__auralisCard(); return { top: c.y, height: c.h }; };
      document.querySelector('#mini-player .mini-meta').click();
      await new Promise((r) => setTimeout(r, 90));
      const before = card();
      auralisUpdate(JSON.stringify({ type: 'open_player' }));
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
      const after = card();
      await new Promise((r) => setTimeout(r, 900));
      return { before, after };
    });
    assert.ok(reversal.after.height >= reversal.before.height - 24,
      `Reversal restarted the sheet: ${reversal.before.height.toFixed(0)}px -> ${reversal.after.height.toFixed(0)}px`);
    await page.waitForTimeout(1200);

    await page.screenshot({ path: path.join(output, 'settled.png') });
    fs.writeFileSync(path.join(output, 'report.json'), JSON.stringify({
      result: 'passed',
      frames: { open: open.length, mid: mid.length, close: close.length, closing: closing.length },
      container: { from: rest.mini, to: { w: W0, h: H0 } },
      scale: { first: mid[0].page.w / mid[0].pageLayoutW, last: mid[mid.length - 1].page.w / mid[mid.length - 1].pageLayoutW },
      cover: { first: mid[0].overlay, last: mid[mid.length - 1].overlay, landed: landed.cover },
    }, null, 2));
    console.log(`PASS: shared container grows from the bar (${rest.mini.w.toFixed(0)}->${W0}px) while the page stays rigid, uniformly scaled and clipped; sections never overlap; cover/title/control tracks are continuous; closing returns to the song bar; a reversal keeps its pose.`);
  } finally {
    await browser.close();
  }
})().catch((e) => { console.error(e); process.exitCode = 1; });
