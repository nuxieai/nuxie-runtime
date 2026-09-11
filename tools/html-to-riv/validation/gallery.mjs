import fs from 'node:fs';

/** A portable artifact: embeds screenshots and data, with no server or CDN. */
export function renderGallery(report) {
  const data = JSON.stringify(report).replaceAll('<','\\u003c').replaceAll('>','\\u003e').replaceAll('&','\\u0026');
  return fs.readFileSync(new URL('gallery.html',import.meta.url),'utf8').replace('/* REPORT_DATA */',data);
}
