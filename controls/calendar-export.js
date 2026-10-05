const pad = number => String(number).padStart(2, '0');
const dateString = date => `${pad(date.getMonth() + 1)}/${pad(date.getDate())}/${date.getFullYear()}`;
const timeString = date => `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
const field = value => {
  const text = String(value ?? '');
  const prefix = /^[=+\-@]/.test(text.trimStart()) ? "'" : '';
  return `"${prefix}${text.replaceAll('"', '""')}"`;
};
const parseBoundary = (value, end = false) => {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) throw new Error('请选择有效的日期范围。');
  const [year, month, day] = value.split('-').map(Number);
  const date = new Date(year, month - 1, day);
  if (date.getFullYear() !== year || date.getMonth() !== month - 1 || date.getDate() !== day) throw new Error('无效日期。');
  if (end) date.setDate(date.getDate() + 1);
  return date;
};

export function convertCalendar(ICAL, text, from, through, format = 'csv') {
  const lower = parseBoundary(from);
  const upper = parseBoundary(through, true);
  if (upper <= lower || upper - lower > 366 * 10 * 86400000) throw new Error('日期范围应不超过十年，且结束日期不能早于开始日期。');
  if (text.length > 5_000_000) throw new Error('ICS 文件过大，请分日历导出后再转换。');
  ICAL.TimezoneService.reset();
  const calendar = new ICAL.Component(ICAL.parse(text.replace(/^\uFEFF/, '')));
  if (calendar.name !== 'vcalendar') throw new Error('文件不是有效的 ICS 日历。');
  calendar.getAllSubcomponents('vtimezone').forEach(zone => ICAL.TimezoneService.register(zone));
  const components = calendar.getAllSubcomponents('vevent');
  if (!components.length) throw new Error('ICS 文件里没有日历事件。');
  if (components.length > 10000) throw new Error('ICS 事件过多，请分日历导出。');
  const exceptions = new Map();
  for (const component of components) {
    if (!component.hasProperty('recurrence-id')) continue;
    const uid = component.getFirstPropertyValue('uid');
    if (!exceptions.has(uid)) exceptions.set(uid, []);
    exceptions.get(uid).push(component);
  }
  for (const component of components) {
    for (const name of ['dtstart', 'dtend', 'recurrence-id', 'rdate', 'exdate']) {
      for (const property of component.getAllProperties(name)) {
        const zone = property.getParameter('tzid');
        if (zone && !ICAL.TimezoneService.has(zone)) throw new Error(`ICS 缺少 ${zone} 的时区定义，请保留原始 ICS 导入 Outlook，避免时间偏差。`);
      }
    }
  }
  const rows = [];
  const seen = new Set();
  let iterations = 0;
  const add = (event, start, end, key) => {
    if (String(event.component.getFirstPropertyValue('status')).toUpperCase() === 'CANCELLED') return;
    const startDate = start.toJSDate();
    const endDate = end.toJSDate();
    if (!Number.isFinite(startDate.getTime()) || !Number.isFinite(endDate.getTime()) || endDate < startDate) throw new Error('有事件的时间无效，无法可靠转换。');
    if (startDate >= upper || (endDate > startDate ? endDate <= lower : startDate < lower) || seen.has(key)) return;
    seen.add(key);
    if (rows.length >= 10000) throw new Error('事件超过 10,000 条，请缩小日期范围。');
    if (start.isDate && endDate > startDate) endDate.setDate(endDate.getDate() - 1);
    rows.push({ subject: event.summary || '(untitled)', startDate: dateString(startDate),
      startTime: start.isDate ? '' : timeString(startDate), endDate: dateString(endDate),
      endTime: start.isDate ? '' : timeString(endDate), allDay: start.isDate ? 'True' : 'False',
      description: event.description || '', location: event.location || '',
      private: String(event.component.getFirstPropertyValue('class')).toUpperCase() === 'PRIVATE' ? 'True' : 'False',
      sort: startDate.getTime() });
  };
  for (const [index, component] of components.entries()) {
    const event = new ICAL.Event(component, { strictExceptions: true,
      exceptions: component.hasProperty('recurrence-id') ? [] : (exceptions.get(component.getFirstPropertyValue('uid')) || []) });
    if (!event.startDate) throw new Error('有事件缺少开始时间，无法可靠转换。');
    const uid = event.uid || `event-${index}`;
    if (event.isRecurrenceException()) {
      add(event, event.startDate, event.endDate, `${uid}:${event.recurrenceId.toString()}`);
    } else if (event.isRecurring()) {
      const iterator = event.iterator();
      let occurrence;
      while ((occurrence = iterator.next())) {
        if (++iterations > 100000) throw new Error('重复事件展开过多，请使用原始 ICS 文件。');
        if (occurrence.toJSDate() >= upper) break;
        const detail = event.getOccurrenceDetails(occurrence);
        add(detail.item, detail.startDate, detail.endDate, `${uid}:${occurrence.toString()}`);
      }
    } else {
      add(event, event.startDate, event.endDate, uid);
    }
  }
  rows.sort((a, b) => a.sort - b.sort);
  if (!rows.length) throw new Error('所选日期范围内没有事件。');
  const columns = ['subject', 'startDate', 'startTime', 'endDate', 'endTime', 'allDay', 'description', 'location', 'private'];
  const content = format === 'csv' ? '\uFEFFSubject,Start Date,Start Time,End Date,End Time,All day event,Description,Location,Private\r\n' +
    rows.map(row => columns.map(column => field(row[column])).join(',')).join('\r\n') + '\r\n' :
    rows.map(row => `${row.subject}\r\n${row.startDate} ${row.startTime} – ${row.endDate} ${row.endTime}${row.allDay === 'True' ? ' (all day)' : ''}\r\n${row.location}\r\n${row.description}`).join('\r\n\r\n');
  return { content, count: rows.length };
}
