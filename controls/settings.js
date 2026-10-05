import ICAL from './vendor/ical.js';
import { convertCalendar } from './calendar-export.js';

const invoke = (command, args = {}) => window.__TAURI_INTERNALS__.invoke(command, args);
const names = { calendar: '日历', reminders: '提醒事项', notes: '备忘录' };
let preferences;
let busy = false;
let exporting = false;
const status = (message, error = false) => {
  document.querySelector('#status').textContent = message;
  document.querySelector('#status').classList.toggle('error', error);
};
function render() {
  document.querySelector('#windows').innerHTML = Object.entries(names).map(([page, name]) => {
    const pref = preferences[page];
    const mode = pref.desktop_mode ? 'desktop' : pref.always_on_top ? 'top' : 'normal';
    return `<section data-page="${page}"><h2>${name}</h2>
      <label class="row">窗口模式<select data-field="mode">
        <option value="normal" ${mode === 'normal' ? 'selected' : ''}>正常窗口</option>
        <option value="desktop" ${mode === 'desktop' ? 'selected' : ''}>固定到桌面</option>
        <option value="top" ${mode === 'top' ? 'selected' : ''}>窗口置顶</option></select></label>
      <label class="row">内容缩放<span class="slider"><input aria-label="${name}内容缩放" data-field="zoom" type="range" min="25" max="150" step="5" value="${Math.round(pref.zoom * 100)}"><output>${Math.round(pref.zoom * 100)}%</output></span></label>
      <label class="fit"><input data-field="auto_fit" type="checkbox" ${pref.auto_fit ? 'checked' : ''}>随窗口自动缩放，让内容适合窗口</label>
      <p class="hint">自动缩放时，上方比例作为最大缩放值。小窗口里的文字也会相应变小。</p>
      <label class="row">不透明度<span class="slider"><input aria-label="${name}不透明度" data-field="opacity" type="range" min="30" max="100" step="5" value="${pref.opacity}"><output>${pref.opacity}%</output></span></label>
    </section>`;
  }).join('');
}
async function run(action) {
  if (busy) return;
  busy = true;
  document.querySelectorAll('#windows input, #windows select, .workspace-actions button').forEach(el => el.disabled = true);
  try { preferences = await action(); render(); status('设置已保存。'); }
  catch (error) { render(); status(String(error), true); }
  finally {
    busy = false;
    document.querySelectorAll('#windows input, #windows select, .workspace-actions button').forEach(el => el.disabled = false);
  }
}
document.querySelector('#windows').addEventListener('input', event => {
  if (event.target.type === 'range') event.target.nextElementSibling.textContent = `${event.target.value}%`;
});
document.querySelector('#windows').addEventListener('change', event => {
  const section = event.target.closest('[data-page]');
  if (!section) return;
  run(() => invoke('set_window_preferences', {
    page: section.dataset.page,
    zoom: Number(section.querySelector('[data-field="zoom"]').value) / 100,
    opacity: Number(section.querySelector('[data-field="opacity"]').value),
    autoFit: section.querySelector('[data-field="auto_fit"]').checked,
    mode: section.querySelector('[data-field="mode"]').value,
  }));
});
document.querySelector('#pin-all').onclick = () => run(() => invoke('set_all_window_modes', { desktop: true }));
document.querySelector('#normal-all').onclick = () => run(() => invoke('set_all_window_modes', { desktop: false }));
const exportButtons = () => document.querySelectorAll('[data-export], [data-calendar-export]');
const finishExport = message => {
  exporting = false;
  exportButtons().forEach(button => button.disabled = false);
  status(message, /失败|无法|没有读到|尚未/.test(message));
};
document.querySelectorAll('[data-export]').forEach(button => button.onclick = async () => {
  if (exporting) return;
  exporting = true;
  exportButtons().forEach(button => button.disabled = true);
  status('正在读取页面，请稍候…');
  try { await invoke('export_current_page', { page: document.querySelector('#export-page').value, format: button.dataset.export }); }
  catch (error) { finishExport(String(error)); }
});
const today = new Date();
document.querySelector('#calendar-from').value = `${today.getFullYear()}-01-01`;
document.querySelector('#calendar-through').value = `${today.getFullYear()}-12-31`;
document.querySelectorAll('[data-calendar-export]').forEach(button => button.onclick = async () => {
  if (exporting) return;
  try {
    const file = document.querySelector('#calendar-file').files[0];
    if (!file) throw new Error('请先选择 ICS 日历文件。');
    if (file.size > 5_000_000) throw new Error('ICS 文件过大，请分日历导出后再转换。');
    exporting = true;
    exportButtons().forEach(button => button.disabled = true);
    const format = button.dataset.calendarExport;
    const result = convertCalendar(ICAL, await file.text(), document.querySelector('#calendar-from').value,
      document.querySelector('#calendar-through').value, format);
    status(`已转换 ${result.count} 条事件，请选择保存位置…`);
    await invoke('save_calendar_export', { text: result.content, format });
  } catch (error) { finishExport(String(error)); }
});
async function initialize() {
  try {
    const handler = window.__TAURI_INTERNALS__.transformCallback(event => finishExport(event.payload));
    await invoke('plugin:event|listen', { event: 'export-status', target: { kind: 'WebviewWindow', label: 'settings' }, handler });
    preferences = await invoke('get_window_preferences');
    render(); status('设置会自动保存，关闭此面板即可。');
  } catch (error) { status(String(error), true); }
}
window.addEventListener('focus', async () => {
  if (busy || !preferences) return;
  try { preferences = await invoke('get_window_preferences'); render(); }
  catch (error) { status(String(error), true); }
});
initialize();

