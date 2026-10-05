import test from 'node:test';
import assert from 'node:assert/strict';
import ICAL from 'ical.js';
import { convertCalendar } from '../controls/calendar-export.js';

const calendar = (...events) => ['BEGIN:VCALENDAR', 'VERSION:2.0', ...events, 'END:VCALENDAR'].join('\r\n');
const event = (...lines) => ['BEGIN:VEVENT', ...lines, 'END:VEVENT'].join('\r\n');
const convert = text => convertCalendar(ICAL, text, '2026-10-01', '2026-10-31');

test('exports all-day end date inclusively and preserves folded Unicode text and multiline descriptions', () => {
  const result = convert(calendar(event('UID:one', 'DTSTART;VALUE=DATE:20261005', 'DTEND;VALUE=DATE:20261008',
    'SUMMARY:中文', ' 折行', 'DESCRIPTION:first\\nsecond\\,quoted', 'LOCATION:Office')));
  assert.equal(result.count, 1);
  assert.match(result.content, /"中文折行","10\/05\/2026","","10\/07\/2026","","True"/);
  assert.match(result.content, /first\nsecond,quoted/);
  assert.ok(result.content.startsWith('\uFEFFSubject,Start Date'));
});

test('expands recurrence, exclusions, moved occurrences and cancellations without duplicates', () => {
  const result = convert(calendar(
    event('UID:daily', 'DTSTART:20261005T090000', 'DTEND:20261005T100000', 'RRULE:FREQ=DAILY;COUNT=4', 'EXDATE:20261006T090000', 'SUMMARY:Daily'),
    event('UID:daily', 'RECURRENCE-ID:20261007T090000', 'DTSTART:20261007T110000', 'DTEND:20261007T120000', 'SUMMARY:Moved'),
    event('UID:daily', 'RECURRENCE-ID:20261008T090000', 'DTSTART:20261008T090000', 'DTEND:20261008T100000', 'STATUS:CANCELLED'),
    event('UID:other', 'DTSTART:20261009T090000', 'DTEND:20261009T100000', 'SUMMARY:Separate')));
  assert.equal(result.count, 3);
  assert.match(result.content, /"Moved","10\/07\/2026","11:00:00"/);
  assert.doesNotMatch(result.content, /10\/06\/2026|10\/08\/2026/);
});

test('uses explicit timezone definitions and rejects missing ones', () => {
  const timezone = ['BEGIN:VTIMEZONE', 'TZID:Asia/Shanghai', 'BEGIN:STANDARD', 'DTSTART:19700101T000000',
    'TZOFFSETFROM:+0800', 'TZOFFSETTO:+0800', 'END:STANDARD', 'END:VTIMEZONE'].join('\r\n');
  const item = event('UID:zone', 'DTSTART;TZID=Asia/Shanghai:20261005T090000', 'DTEND;TZID=Asia/Shanghai:20261005T100000', 'SUMMARY:Meeting');
  assert.throws(() => convert(calendar(item)), /缺少.*时区/);
  const result = convert(calendar(timezone, item));
  const expectedHour = String(new Date('2026-10-05T01:00:00Z').getHours()).padStart(2, '0');
  assert.ok(result.content.includes(`"${expectedHour}:00:00"`));
});

test('protects CSV cells, supports TXT and rejects empty or reversed ranges', () => {
  const file = calendar(event('UID:csv', 'DTSTART;VALUE=DATE:20261005', 'SUMMARY:  =1+1'));
  assert.match(convert(file).content, /"'  =1\+1"/);
  assert.match(convertCalendar(ICAL, file, '2026-10-01', '2026-10-31', 'txt').content, /  =1\+1/);
  assert.throws(() => convertCalendar(ICAL, file, '2026-11-01', '2026-10-01'), /结束日期/);
  assert.throws(() => convertCalendar(ICAL, file, '2026-11-01', '2026-11-30'), /没有事件/);
});

test('includes an exception moved into the range from a later original occurrence', () => {
  const result = convert(calendar(
    event('UID:monthly', 'DTSTART:20260901T090000', 'DTEND:20260901T100000', 'RRULE:FREQ=MONTHLY;COUNT=4', 'SUMMARY:Monthly'),
    event('UID:monthly', 'RECURRENCE-ID:20261101T090000', 'DTSTART:20261015T110000', 'DTEND:20261015T120000', 'SUMMARY:Early')));
  assert.equal(result.count, 2);
  assert.match(result.content, /Early/);
});
