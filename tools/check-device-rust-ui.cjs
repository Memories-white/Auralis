// Requires the debug app's WebView DevTools socket forwarded to localhost:9222.
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const path = require('node:path');
const output = path.resolve(__dirname, '../device-debug/rust-ui');
fs.mkdirSync(output, { recursive: true });
(async () => {
  const browser = await chromium.connectOverCDP(process.env.AURALIS_CDP || 'http://127.0.0.1:9222');
  const page = browser.contexts()[0].pages().find(page => page.url().includes('auralis.local'));
  if (!page) throw new Error('Auralis WebView not found');
  let original = '';
  const report = {};
  try {
    await page.locator('#navigation [data-action="navigate:settings"]').click();
    await page.locator('[data-action="navigate:appearance"]').click();
    await page.locator('[data-action="appearance:light"]').waitFor();
    original = await page.locator('.preference').filter({ has: page.locator('[role=radio][aria-checked=true]') }).getAttribute('data-action');
    await page.locator('[data-action="appearance:light"]').click();
    await page.waitForTimeout(700);
    await page.screenshot({ path: path.join(output, 'device-appearance-light.png') });
    report.appearance = 'light mode displayed and original choice restored';
    if (original !== 'appearance:light') await page.locator(`[data-action="${original}"]`).click();
    original = '';
    await page.locator('#header [data-action=back]').click();
    await page.locator('#navigation [data-action="navigate:home"]').click();
    if (!(await page.locator('[data-action="playlist:recommendations:0"]').count())) {
      await page.locator('[data-action=refresh_home]').first().click();
      await page.locator('[data-action="playlist:recommendations:0"]').waitFor({ timeout: 25000 });
    }
    await page.locator('[data-action="playlist:recommendations:0"]').click();
    await page.locator('#header [data-action=back]').waitFor();
    await page.waitForFunction(() => document.querySelectorAll('#content .song-row').length > 0 || !!document.querySelector('[data-action=retry_playlist]'), { timeout: 25000 });
    report.firstRecommendedPlaylist = await page.evaluate(() => ({ rows: document.querySelectorAll('#content .song-row').length, loading: !!document.querySelector('.loading'), retry: !!document.querySelector('[data-action=retry_playlist]'), back: !!document.querySelector('#header [data-action=back]') }));
    await page.screenshot({ path: path.join(output, 'device-first-playlist.png') });
    await page.locator('#header [data-action=back]').click();
    await page.locator('#navigation [data-action="navigate:home"]').waitFor();
    await page.locator('#mini-player .mini-identity>span:not(.artwork)').click();
    await page.waitForTimeout(750);
    await page.screenshot({ path: path.join(output, 'device-player.png') });
    await page.locator('#player-pages').dispatchEvent('pointerdown', { clientX: 330, clientY: 350 });
    await page.locator('#player-pages').dispatchEvent('pointerup', { clientX: 70, clientY: 350 });
    await page.waitForTimeout(600);
    report.player = await page.evaluate(() => ({ open: document.querySelector('#player').classList.contains('visible'), leftSwipeLyrics: document.querySelector('#player').classList.contains('lyrics-page'), framedProgress: getComputedStyle(document.querySelector('.lyric-progress')).display !== 'none' }));
    await page.screenshot({ path: path.join(output, 'device-lyrics.png') });
    await page.locator('[data-action=close_player]').click();
    await page.waitForTimeout(700);
    report.result = 'passed';
    fs.writeFileSync(path.join(output, 'device-report.json'), JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report));
  } finally {
    if (original) await page.evaluate(mode => window.AuralisNative.postMessage(JSON.stringify({ type: 'appearance', value: mode.split(':')[1] })), original).catch(() => {});
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
