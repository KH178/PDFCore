/**
 * Import/Export for .pdfCoret template packages (ZIP files).
 *
 * .pdfCoret structure:
 *   layout.json   – The PDFCore Template (root, settings)
 *   styles.json   – Named style definitions
 *   manifest.json – Name, version, author
 *   assets/       – Embedded images and fonts
 */
import JSZip from 'jszip';
import type { PDFCoreTemplate } from './types';

/**
 * Collect all embedded images (data URLs) from the editor canvas
 * and return them as named blobs.
 */
export function collectAssetsFromEditor(editor: any): Map<string, Blob> {
  const assets = new Map<string, Blob>();
  const wrapper = editor.DomComponents.getWrapper();
  if (!wrapper) return assets;

  function walk(comp: any) {
    const attrs = comp.getAttributes?.() || {};
    const pdfType = attrs['data-pdf-type'];

    if (pdfType === 'Image') {
      const pdfSrc = attrs['data-pdf-src'] || '';
      // Look for an embedded <img> with a data URL
      const imgEl = comp.view?.el?.querySelector('img');
      const src = imgEl?.getAttribute('src') || '';

      if (src.startsWith('data:') && pdfSrc) {
        // Convert data URL to blob
        const blob = dataUrlToBlob(src);
        if (blob) {
          assets.set(pdfSrc, blob);
        }
      }
    }

    comp.components().each((child: any) => walk(child));
  }

  walk(wrapper);
  return assets;
}

function dataUrlToBlob(dataUrl: string): Blob | null {
  try {
    const [header, data] = dataUrl.split(',');
    const mimeMatch = header.match(/:(.*?);/);
    const mime = mimeMatch ? mimeMatch[1] : 'application/octet-stream';
    const binary = atob(data);
    const array = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      array[i] = binary.charCodeAt(i);
    }
    return new Blob([array], { type: mime });
  } catch {
    return null;
  }
}

/**
 * Export a PDFCoreTemplate to a .pdfCoret ZIP blob.
 */
export async function exportTemplate(
  template: PDFCoreTemplate,
  assets: Map<string, Blob> = new Map()
): Promise<Blob> {
  const zip = new JSZip();

  // layout.json
  zip.file('layout.json', JSON.stringify({
    root: template.root,
    settings: template.settings,
  }, null, 2));

  // styles.json
  zip.file('styles.json', JSON.stringify(template.styles || {}, null, 2));

  // manifest.json
  zip.file('manifest.json', JSON.stringify(template.manifest || {
    name: 'Untitled',
    version: '1.2',
  }, null, 2));

  // assets/
  for (const [name, blob] of assets) {
    const arrayBuf = await blob.arrayBuffer();
    zip.file(`assets/${name}`, arrayBuf);
  }

  return zip.generateAsync({ type: 'blob' });
}

/**
 * Import a .pdfCoret ZIP blob into a PDFCoreTemplate + assets.
 */
export async function importTemplate(
  file: Blob
): Promise<{ template: PDFCoreTemplate; assets: Map<string, Blob> }> {
  const zip = await JSZip.loadAsync(file);

  // Read layout.json (required)
  const layoutFile = zip.file('layout.json');
  if (!layoutFile) throw new Error('Invalid .pdfCoret: layout.json not found');
  const layoutJson = await layoutFile.async('string');
  const layout = JSON.parse(layoutJson);

  // Read styles.json (optional)
  let styles: Record<string, unknown> = {};
  const stylesFile = zip.file('styles.json');
  if (stylesFile) {
    const stylesJson = await stylesFile.async('string');
    styles = JSON.parse(stylesJson);
  }

  // Read manifest.json (optional)
  let manifest: PDFCoreTemplate['manifest'] = undefined;
  const manifestFile = zip.file('manifest.json');
  if (manifestFile) {
    const manifestJson = await manifestFile.async('string');
    manifest = JSON.parse(manifestJson);
  }

  // Read assets — look for any files under assets/ or any non-JSON files
  const assets = new Map<string, Blob>();
  const assetPaths: string[] = [];
  zip.forEach((relativePath, zipEntry) => {
    if (zipEntry.dir) return;
    if (relativePath === 'layout.json' || relativePath === 'styles.json' || relativePath === 'manifest.json') return;
    assetPaths.push(relativePath);
  });

  // Actually load asset blobs
  for (const path of assetPaths) {
    const name = path.startsWith('assets/') ? path.slice(7) : path;
    if (!name) continue;
    const zipEntry = zip.file(path);
    if (zipEntry) {
      const data = await zipEntry.async('blob');
      assets.set(name, data);
    }
  }

  const template: PDFCoreTemplate = {
    root: layout.root,
    manifest,
    styles,
    settings: layout.settings,
  };

  return { template, assets };
}

/**
 * Trigger a browser download of a Blob.
 */
export function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
