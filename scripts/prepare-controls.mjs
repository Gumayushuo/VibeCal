import { mkdir, copyFile } from 'node:fs/promises';
const root = new URL('../', import.meta.url);
await mkdir(new URL('controls/vendor/', root), { recursive: true });
await copyFile(new URL('node_modules/ical.js/dist/ical.min.js', root), new URL('controls/vendor/ical.js', root));
await copyFile(new URL('node_modules/ical.js/LICENSE', root), new URL('controls/vendor/ICAL-LICENSE.txt', root));
