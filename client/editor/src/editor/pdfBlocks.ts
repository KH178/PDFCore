/**
 * PDFCore Block Definitions for GrapesJS
 *
 * Simple blocks use raw HTML strings.
 * Multi-column blocks use GrapesJS component JSON with explicit `type`
 * references so nested components get the correct custom types.
 */
import type grapesjs from 'grapesjs';
type Editor = grapesjs.Editor;

export function registerPDFBlocks(editor: Editor) {
  const bm = editor.BlockManager as any;

  // ═══ LAYOUT ══════════════════════════════════════════════════════════

  bm.add('pdf-column', {
    label: '📐 Column',
    category: 'Layout',
    content: `<div data-pdf-type="Column" style="display:flex;flex-direction:column;gap:8px;min-height:60px;padding:12px;border-radius:4px;box-sizing:border-box;"></div>`,
    attributes: { class: 'fa fa-columns' },
  });

  bm.add('pdf-row', {
    label: '↔ Row (Inline)',
    category: 'Layout',
    content: `<div data-pdf-type="Row" style="display:flex;flex-direction:row;flex-wrap:nowrap;gap:8px;min-height:40px;padding:12px;border-radius:4px;box-sizing:border-box;"></div>`,
    attributes: { class: 'fa fa-arrows-h' },
  });

  bm.add('pdf-container', {
    label: '☐ Container',
    category: 'Layout',
    content: `<div data-pdf-type="Container" style="padding:16px;border:1px solid #d1d5db;min-height:40px;border-radius:4px;box-sizing:border-box;"></div>`,
    attributes: { class: 'fa fa-square-o' },
  });

  // 2 Columns — use component JSON so nested types are resolved correctly
  bm.add('pdf-twocol', {
    label: '▥ 2 Columns',
    category: 'Layout',
    content: {
      type: 'pdf-row-comp',
      style: { display: 'flex', 'flex-direction': 'row', gap: '12px', 'min-height': '60px', padding: '12px', 'border-radius': '4px', 'box-sizing': 'border-box' },
      components: [
        {
          type: 'pdf-column-comp',
          style: { flex: '1', display: 'flex', 'flex-direction': 'column', gap: '8px', 'min-height': '40px', padding: '12px', 'border-radius': '4px', 'box-sizing': 'border-box' },
        },
        {
          type: 'pdf-column-comp',
          style: { flex: '1', display: 'flex', 'flex-direction': 'column', gap: '8px', 'min-height': '40px', padding: '12px', 'border-radius': '4px', 'box-sizing': 'border-box' },
        },
      ],
    },
    attributes: { class: 'fa fa-th-large' },
  });

  // 3 Columns
  bm.add('pdf-threecol', {
    label: '▦ 3 Columns',
    category: 'Layout',
    content: {
      type: 'pdf-row-comp',
      style: { display: 'flex', 'flex-direction': 'row', gap: '12px', 'min-height': '60px', padding: '12px', 'border-radius': '4px', 'box-sizing': 'border-box' },
      components: [
        {
          type: 'pdf-column-comp',
          style: { flex: '1', display: 'flex', 'flex-direction': 'column', gap: '8px', 'min-height': '40px', padding: '12px', 'border-radius': '4px', 'box-sizing': 'border-box' },
        },
        {
          type: 'pdf-column-comp',
          style: { flex: '1', display: 'flex', 'flex-direction': 'column', gap: '8px', 'min-height': '40px', padding: '12px', 'border-radius': '4px', 'box-sizing': 'border-box' },
        },
        {
          type: 'pdf-column-comp',
          style: { flex: '1', display: 'flex', 'flex-direction': 'column', gap: '8px', 'min-height': '40px', padding: '12px', 'border-radius': '4px', 'box-sizing': 'border-box' },
        },
      ],
    },
    attributes: { class: 'fa fa-th' },
  });

  // ═══ TEXT ═════════════════════════════════════════════════════════════

  bm.add('pdf-text', {
    label: '¶ Text',
    category: 'Text',
    content: `<div data-pdf-type="Text" style="font-size:14px;font-family:inherit;padding:4px;">Type your text here...</div>`,
    attributes: { class: 'fa fa-paragraph' },
  });

  bm.add('pdf-heading', {
    label: 'H Heading',
    category: 'Text',
    content: `<div data-pdf-type="Text" data-pdf-size="24" style="font-size:24px;font-weight:bold;font-family:inherit;padding:4px;">Heading</div>`,
    attributes: { class: 'fa fa-header' },
  });

  bm.add('pdf-label', {
    label: '🏷 Label',
    category: 'Text',
    content: `<div data-pdf-type="Text" data-pdf-size="10" style="font-size:10px;color:#64748b;text-transform:uppercase;letter-spacing:1px;font-family:Inter,sans-serif;padding:2px;">LABEL</div>`,
    attributes: { class: 'fa fa-tag' },
  });

  // ═══ MEDIA ═══════════════════════════════════════════════════════════

  bm.add('pdf-image', {
    label: '🖼 Image',
    category: 'Media',
    content: `<div data-pdf-type="Image" style="width:200px;height:150px;background:#f1f5f9;display:flex;align-items:center;justify-content:center;border:2px dashed #cbd5e1;font-size:11px;color:#64748b;border-radius:4px;">📷 Click upload in toolbar</div>`,
    attributes: { class: 'fa fa-image' },
  });

  // ═══ DATA ════════════════════════════════════════════════════════════

  bm.add('pdf-table', {
    label: '☰ Table',
    category: 'Data',
    content: buildTableHtml(3, 2, {
      headerBg: '#1e293b', headerColor: '#ffffff',
      borderColor: '#e2e8f0', cellPadding: 10, fontSize: 12, stripedRows: true,
    }),
    attributes: { class: 'fa fa-table' },
  });

  bm.add('pdf-dynamic-text', {
    label: '⚡ Dynamic Text',
    category: 'Data',
    content: `<span data-pdf-type="DynamicText" data-binding="" style="font-size:inherit;color:inherit;font-family:inherit;font-style:italic;">{{field_name}}</span>`,
    attributes: { class: 'fa fa-bolt' },
  });

  // ═══ SHAPES ══════════════════════════════════════════════════════════

  bm.add('pdf-rect', {
    label: '■ Rectangle',
    category: 'Shapes',
    content: `<div data-pdf-type="Rectangle" style="width:120px;height:80px;background-color:#3b82f6;border:2px solid #1e40af;border-radius:4px;"></div>`,
    attributes: { class: 'fa fa-square' },
  });

  bm.add('pdf-circle', {
    label: '● Circle',
    category: 'Shapes',
    content: `<div data-pdf-type="Circle" style="width:80px;height:80px;background-color:#10b981;border:2px solid #047857;border-radius:50%;"></div>`,
    attributes: { class: 'fa fa-circle' },
  });

  bm.add('pdf-line', {
    label: '— Line',
    category: 'Shapes',
    content: `<hr data-pdf-type="Line" style="width:200px;border:none;border-top:2px solid #334155;margin:8px 0;" />`,
    attributes: { class: 'fa fa-minus' },
  });

  // ═══ UTILITY ═════════════════════════════════════════════════════════

  bm.add('pdf-page-break', {
    label: '✂ Page Break',
    category: 'Utility',
    content: `<hr data-pdf-type="PageBreak" style="border:none;border-top:2px dashed #ef4444;margin:16px 0;" />`,
    attributes: { class: 'fa fa-scissors' },
  });

  bm.add('pdf-page-number', {
    label: '# Page Number',
    category: 'Utility',
    content: `<div data-pdf-type="PageNumber" style="font-size:10px;color:#6b7280;text-align:center;font-family:Inter,sans-serif;">Page {page} of {total}</div>`,
    attributes: { class: 'fa fa-hashtag' },
  });

  bm.add('pdf-header', {
    label: '▬ Header',
    category: 'Utility',
    content: `<div data-pdf-type="Header" style="width:100%;padding:12px;border-bottom:2px solid #e2e8f0;background:#f8fafc;min-height:40px;"></div>`,
    attributes: { class: 'fa fa-window-maximize' },
  });

  bm.add('pdf-footer', {
    label: '▬ Footer',
    category: 'Utility',
    content: `<div data-pdf-type="Footer" style="width:100%;padding:12px;border-top:2px solid #e2e8f0;background:#f8fafc;min-height:40px;"></div>`,
    attributes: { class: 'fa fa-window-minimize' },
  });

  // ═══ NAVIGATION ══════════════════════════════════════════════════════

  bm.add('pdf-hyperlink', {
    label: '🔗 Hyperlink',
    category: 'Navigation',
    content: `<a data-pdf-type="Hyperlink" href="#" style="color:#2563eb;font-size:12px;text-decoration:underline;font-family:Inter,sans-serif;">Link text</a>`,
    attributes: { class: 'fa fa-link' },
  });
}

// ─── Table Builder ──────────────────────────────────────────────────────────

export function buildTableHtml(
  cols: number, rows: number,
  opts: {
    headerBg?: string; headerColor?: string; borderColor?: string;
    cellPadding?: number; fontSize?: number; stripedRows?: boolean;
    headers?: string[]; data?: string[][];
  } = {}
): string {
  const { headerBg = '#1e293b', headerColor = '#ffffff', borderColor = '#e2e8f0',
    cellPadding = 10, fontSize = 12, stripedRows = true, headers, data } = opts;
  const thS = `border:1px solid ${borderColor};padding:${cellPadding}px;background:${headerBg};color:${headerColor};font-weight:600;text-align:left;font-size:${fontSize}px;`;
  const tdS = (ri: number) => `border:1px solid ${borderColor};padding:${cellPadding}px;font-size:${fontSize}px;${stripedRows && ri % 2 === 1 ? 'background:#f8fafc;' : ''}`;
  const ths = Array.from({ length: cols }, (_, i) => `<th style="${thS}">${headers?.[i] || `Column ${i + 1}`}</th>`).join('');
  const trs = Array.from({ length: rows }, (_, ri) =>
    `<tr>${Array.from({ length: cols }, (_, ci) => `<td data-gjs-type="pdf-td" style="${tdS(ri)}">${data?.[ri]?.[ci] || 'Data'}</td>`).join('')}</tr>`
  ).join('');
  return `<table data-pdf-type="Table" data-header-bg="${headerBg}" data-header-color="${headerColor}" data-border-color="${borderColor}" data-cell-padding="${cellPadding}" data-font-size="${fontSize}" data-striped="${stripedRows}" style="width:100%;border-collapse:collapse;font-size:${fontSize}px;table-layout:fixed;"><thead><tr>${ths}</tr></thead><tbody>${trs}</tbody></table>`;
}
