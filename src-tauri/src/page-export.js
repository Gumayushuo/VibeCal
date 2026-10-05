// This script runs in each frame. No filesystem or Tauri IPC is exposed to cloud pages.
(() => {
  if (window.__vibecalExport) return;
  const trustedOrigin = origin => {
    try {
      const url = new URL(origin);
      return url.protocol === 'https:' && /(^|\.)icloud\.com(\.cn)?$/.test(url.hostname);
    } catch { return false; }
  };
  const visibleFrames = () => [...document.querySelectorAll('iframe')].filter(frame => {
    const rect = frame.getBoundingClientRect();
    const style = getComputedStyle(frame);
    return rect.width > 0 && rect.height > 0 && style.visibility !== 'hidden' && style.display !== 'none';
  });
  let active = null;
  const collect = (token, replyOrigin) => {
    const children = visibleFrames();
    const locked = !!document.querySelector('input[type="password"]');
    const text = locked ? '' : (document.body?.innerText || '').trim();
    const record = { kind: 'vibecal-text', token, children: children.length, locked: locked || text.length > 2000000,
      title: document.title.slice(0, 500), text: text.slice(0, 2000000) };
    if (window === window.top) receive(record);
    else window.top.postMessage(record, replyOrigin);
    for (const frame of children) {
      // The request carries no page data; responses are sent only to the exact cloud origin.
      frame.contentWindow?.postMessage({ kind: 'vibecal-read', token, replyOrigin }, '*');
    }
  };
  const receive = data => {
    if (!active || data.token !== active.token) return;
    active.received++;
    active.expected += data.children;
    active.locked ||= !!data.locked;
    if (data.text) active.records.push({ title: data.title, text: data.text });
  };
  window.addEventListener('message', event => {
    const data = event.data;
    if (!data || !trustedOrigin(event.origin)) return;
    if (data.kind === 'vibecal-read' && event.source === window.parent && trustedOrigin(data.replyOrigin)) {
      collect(data.token, data.replyOrigin);
    } else if (data.kind === 'vibecal-text' && window === window.top) {
      receive(data);
    }
  });
  window.__vibecalExport = {
    start() {
      if (!trustedOrigin(location.origin) || !document.body) return false;
      active = { token: crypto.randomUUID(), expected: 1, received: 0, records: [], locked: false };
      collect(active.token, location.origin);
      return true;
    },
    finish() {
      if (!active) return null;
      const result = { records: active.locked ? [] : active.records,
        missing: Math.max(0, active.expected - active.received) };
      active = null;
      return result;
    }
  };
})();
