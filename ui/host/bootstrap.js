// Host glue only. Application layout and event handlers are compiled from Rust.
import init, { update, system_back } from './pkg/auralis_ui.js';

await init();
window.auralisUpdate = update;
window.auralisBack = system_back;
if (window.AuralisNative) {
  window.AuralisNative.postMessage('{"type":"ready"}');
} else {
  update('{"type":"content","state":{"page":"home","counts":{},"safeTop":24,"safeBottom":16}}');
  window.dispatchEvent(new Event('auralis-ready'));
}
