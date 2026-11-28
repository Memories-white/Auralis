// Reads the player sheet's ACTUAL visible card rectangle inside the page.
//
// The morph may express the card either as a resized layout box (v1) or as a fixed
// window box with a `clip-path: inset(... round ...)` (v2). Both are reduced to the
// same visible rect so the geometry assertions stay implementation-agnostic and are
// never loosened.
function installCardReader() {
  window.__auralisCard = () => {
    const sheet = document.querySelector('#player-sheet');
    if (!sheet) return null;
    const base = sheet.getBoundingClientRect();
    const cp = getComputedStyle(sheet).clipPath;
    if (cp && cp.startsWith('inset(')) {
      const inner = cp.slice('inset('.length, cp.lastIndexOf(')'));
      const split = inner.split(' round ');
      const v = split[0].trim().split(/[\s]+/).map(parseFloat);
      const t = v[0];
      const r = v.length > 1 ? v[1] : v[0];
      const b = v.length > 2 ? v[2] : v[0];
      const l = v.length > 3 ? v[3] : r;
      return {
        x: base.x + l,
        y: base.y + t,
        w: base.width - l - r,
        h: base.height - t - b,
        right: base.right - r,
        bottom: base.bottom - b,
        radius: split[1] ? parseFloat(split[1]) : 0,
        clipped: true,
      };
    }
    // Legacy engine: the card is the surface's own layout box and its radius lives on
    // the container's border-radius.
    const surface = document.querySelector('#player-surface');
    const r = surface ? surface.getBoundingClientRect() : base;
    const radius = parseFloat(getComputedStyle(sheet).borderTopLeftRadius) || 0;
    return { x: r.x, y: r.y, w: r.width, h: r.height, right: r.right, bottom: r.bottom, radius, clipped: false };
  };
}

// Source string that installs `window.__auralisCard()`; evaluate it once per page.
exports.CARD_RECT_SOURCE = `(${installCardReader.toString()})()`;

// Progress of the sheet between the song bar and the window, derived from the real
// visible card rect (not from an implementation-specific box).
exports.PROGRESS_SOURCE = `
window.__auralisProgress = (barHeight) => {
  const card = window.__auralisCard();
  if (!card) return 0;
  return (card.h - barHeight) / (innerHeight - barHeight);
};`;
