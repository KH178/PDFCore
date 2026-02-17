/**
 * Converter — bridges GrapesJS component tree ↔ PDFCore Template JSON.
 *
 * EXPORT: walks the GrapesJS component tree → produces TemplateNodes
 * IMPORT: converts TemplateNodes → HTML for GrapesJS to parse
 *
 * Supports: Text, Image, Table, Rectangle, Circle, Line, Header, Footer,
 * PageNumber, DynamicText, Hyperlink, Column, Row, Container, PageBreak
 */
import type grapesjs from 'grapesjs';
type Component = ReturnType<ReturnType<grapesjs.Editor['DomComponents']['getWrapper']>['components']>['models'][number];
type Editor = grapesjs.Editor;

import type { TemplateNode, PDFCoreTemplate } from './types';

// ─── EXPORT ─────────────────────────────────────────────────────────────────

export function editorToTemplate(editor: Editor): PDFCoreTemplate {
  const wrapper = editor.DomComponents.getWrapper();
  if (!wrapper) return { root: { type: 'Column', children: [] }, styles: {} };

  // Find the PageRoot component
  const pageRoot = wrapper.components().models.find(
    (m: Component) => m.getAttributes()['data-pdf-type'] === 'PageRoot'
  );

  let children: TemplateNode[] = [];
  let margins = { top: 40, bottom: 40, left: 40, right: 40 };

  if (pageRoot) {
    // Extract children from inside PageRoot
    children = pageRoot.components().models
      .map(componentToNode)
      .filter(Boolean) as TemplateNode[];
    // Extract margins from PageRoot padding
    const style = pageRoot.getStyle();
    margins = {
      top: pf(style['padding-top'] || style['padding']) || 40,
      bottom: pf(style['padding-bottom'] || style['padding']) || 40,
      left: pf(style['padding-left'] || style['padding']) || 40,
      right: pf(style['padding-right'] || style['padding']) || 40,
    };
  } else {
    // Fallback: no PageRoot, use wrapper children directly
    children = wrapper.components().models
      .map(componentToNode)
      .filter(Boolean) as TemplateNode[];
  }

  return {
    root: children.length === 1 ? children[0] : { type: 'Column', children },
    manifest: { name: 'Untitled Template', version: '1.2' },
    styles: {},
    settings: {
      size: 'A4',
      orientation: 'portrait',
      margins,
    },
  };
}

function componentToNode(comp: Component): TemplateNode | null {
  const attrs = comp.getAttributes();
  const pdfType = attrs['data-pdf-type'];

  if (!pdfType) {
    const children = comp.components().models;
    if (children.length === 0) return null;
    if (children.length === 1) return componentToNode(children[0]);
    return {
      type: 'Column',
      children: children.map(componentToNode).filter(Boolean) as TemplateNode[],
    };
  }

  const style = comp.getStyle();

  switch (pdfType) {
    // PageRoot is the page wrapper — extract its children, skip the root itself
    case 'PageRoot': {
      const kids = getContentChildren(comp);
      return kids.length === 1 ? kids[0] : { type: 'Column', children: kids };
    }

    case 'Column':
      return { type: 'Column', children: getContentChildren(comp), spacing: pf(style['gap']) };

    case 'Row':
      return { type: 'Row', children: getContentChildren(comp), spacing: pf(style['gap']) };

    case 'Container': {
      const real = getContentChildren(comp);
      return {
        type: 'Container',
        child: real.length === 1 ? real[0] : { type: 'Column', children: real },
        padding: pf(style['padding']),
        border: pf(style['border-width']),
      };
    }

    case 'Text': {
      const el = (comp.view as any)?.el;
      const text = el?.textContent?.trim() || String(comp.get('content') || '').trim();
      const node: TemplateNode = {
        type: 'Text',
        content: text,
        size: attrs['data-pdf-size'] ? parseFloat(attrs['data-pdf-size']) : pf(style['font-size']) || 12,
      };
      if (style['color'] && style['color'] !== 'inherit') node.color = cssToColor(style['color']);
      if (style['background-color'] && style['background-color'] !== 'transparent') node.background_color = cssToColor(style['background-color']);
      if (style['max-width']) node.width = pf(style['max-width']);
      if (style['font-weight'] === 'bold' || style['font-weight'] === '700') node.bold = true;
      if (style['font-style'] === 'italic') node.italic = true;
      if (style['text-align']) node.align = style['text-align'];
      if (style['font-family']) node.fontFamily = style['font-family'].replace(/['"]/g, '');
      if (style['line-height'] && style['line-height'] !== 'normal') node.lineHeight = parseFloat(style['line-height']);
      if (style['letter-spacing']) node.letterSpacing = pf(style['letter-spacing']);
      if (style['opacity'] && style['opacity'] !== '1') node.opacity = parseFloat(style['opacity']);
      if (style['transform']) node.rotation = parseRotation(style['transform']);
      if (style['padding']) node.padding = pf(style['padding']);
      if (style['border-width']) node.borderWidth = pf(style['border-width']);
      if (style['border-color']) node.borderColor = cssToColor(style['border-color']);
      return node;
    }

    case 'Image': {
      const imgChild = comp.components().models.find((c: Component) => c.get('tagName') === 'img');
      const src = imgChild?.getAttributes()?.src || attrs['data-pdf-src'] || '';
      return {
        type: 'Image',
        src,
        width: pf(style['width']) || 200,
        height: pf(style['height']) || 150,
        opacity: style['opacity'] ? parseFloat(style['opacity']) : undefined,
        rotation: style['transform'] ? parseRotation(style['transform']) : undefined,
        borderWidth: pf(style['border-width']),
        borderColor: style['border-color'] ? cssToColor(style['border-color']) : undefined,
      };
    }

    case 'Table': {
      const el = (comp.view as any)?.el;
      if (!el) return { type: 'Table', columns: [], rows: [], style: {} };
      return {
        type: 'Table',
        columns: Array.from(el.querySelectorAll('thead th')).map((th: any) => ({
          header: th.textContent || '',
          width: (th.style.width && parseInt(th.style.width)) || th.getBoundingClientRect().width || 100,
        })),
        rows: Array.from(el.querySelectorAll('tbody tr')).map((tr: any) =>
          Array.from(tr.querySelectorAll('td')).map((td: any) => td.textContent || '')
        ),
        style: {
          header_bg: attrs['data-header-bg'] || '#1e293b',
          header_color: attrs['data-header-color'] || '#ffffff',
          border_color: attrs['data-border-color'] || '#e2e8f0',
          cell_padding: parseInt(attrs['data-cell-padding'] || '10'),
          font_size: parseInt(attrs['data-font-size'] || '12'),
          striped: attrs['data-striped'] !== 'false',
        },
      };
    }

    case 'Rectangle':
      return {
        type: 'Rectangle',
        width: pf(style['width']) || 120,
        height: pf(style['height']) || 80,
        fill: cssToColor(style['background-color'] || '#3b82f6'),
        strokeWidth: pf(style['border-width']),
        strokeColor: style['border-color'] ? cssToColor(style['border-color']) : undefined,
        opacity: style['opacity'] ? parseFloat(style['opacity']) : undefined,
        rotation: style['transform'] ? parseRotation(style['transform']) : undefined,
        borderRadius: pf(style['border-radius']),
      };

    case 'Circle':
      return {
        type: 'Circle',
        width: pf(style['width']) || 80,
        height: pf(style['height']) || 80,
        fill: cssToColor(style['background-color'] || '#10b981'),
        strokeWidth: pf(style['border-width']),
        strokeColor: style['border-color'] ? cssToColor(style['border-color']) : undefined,
        opacity: style['opacity'] ? parseFloat(style['opacity']) : undefined,
      };

    case 'Line':
      return {
        type: 'Line',
        width: pf(style['width']) || 200,
        thickness: pf(style['border-top-width'] || style['height']) || 2,
        color: cssToColor(style['border-top-color'] || style['border-color'] || '#334155'),
      };

    case 'Header':
      return { type: 'Header', children: getContentChildren(comp) };

    case 'Footer':
      return { type: 'Footer', children: getContentChildren(comp) };

    case 'PageNumber': {
      const el = (comp.view as any)?.el;
      return {
        type: 'PageNumber',
        format: el?.textContent?.trim() || 'Page {page} of {total}',
        size: pf(style['font-size']) || 10,
        align: style['text-align'] || 'center',
      };
    }

    case 'DynamicText':
      return {
        type: 'DynamicText',
        binding: attrs['data-binding'] || '',
        size: pf(style['font-size']) || 12,
      };

    case 'Hyperlink': {
      const el = (comp.view as any)?.el;
      return {
        type: 'Hyperlink',
        text: el?.textContent?.trim() || 'Link',
        href: attrs['href'] || '#',
        size: pf(style['font-size']) || 12,
      };
    }

    case 'PageBreak':
      return null;

    default:
      return null;
  }
}

function getContentChildren(comp: Component): TemplateNode[] {
  return comp.components().models
    .filter((c: Component) => {
      const tag = c.get('tagName');
      if (tag === 'span' && !c.getAttributes()['data-pdf-type']) return false;
      return true;
    })
    .map(componentToNode)
    .filter(Boolean) as TemplateNode[];
}

// ─── IMPORT ─────────────────────────────────────────────────────────────────

export function templateToEditor(editor: Editor, template: PDFCoreTemplate, assetUrls?: Map<string, string>) {
  const html = nodeToHtml(template.root, assetUrls || new Map());
  // Use proper PageRoot wrapper matching the component type definition
  const settings = (template as any).settings || {};
  const margins = settings.margins || { top: 40, bottom: 40, left: 40, right: 40 };
  const pageHtml = `<div data-pdf-type="PageRoot" style="width:100%;min-height:842px;background:white;padding-top:${margins.top}px;padding-bottom:${margins.bottom}px;padding-left:${margins.left}px;padding-right:${margins.right}px;font-family:'Inter',sans-serif;box-sizing:border-box;">${html}</div>`;
  editor.DomComponents.getWrapper()?.components(pageHtml);
}

function nodeToHtml(node: TemplateNode, assets: Map<string, string>): string {
  switch (node.type) {
    case 'Column': {
      const kids = ((node.children as TemplateNode[]) || []).map(n => nodeToHtml(n, assets)).join('');
      return `<div data-pdf-type="Column" style="display:flex;flex-direction:column;min-height:60px;padding:12px;border:1px dashed #6366f1;border-radius:4px;${node.spacing ? `gap:${node.spacing}px;` : ''}">${kids}</div>`;
    }
    case 'Row': {
      const kids = ((node.children as TemplateNode[]) || []).map(n => nodeToHtml(n, assets)).join('');
      return `<div data-pdf-type="Row" style="display:flex;flex-direction:row;flex-wrap:wrap;min-height:40px;padding:12px;border:1px dashed #0ea5e9;border-radius:4px;${node.spacing ? `gap:${node.spacing}px;` : 'gap:8px;'}">${kids}</div>`;
    }
    case 'Container': {
      const child = node.child ? nodeToHtml(node.child as TemplateNode, assets) : '';
      return `<div data-pdf-type="Container" style="padding:${node.padding || 16}px;border:${node.border || 1}px solid #d1d5db;min-height:40px;border-radius:4px;">${child}</div>`;
    }
    case 'Text': {
      const sz = node.size || 12;
      // Always use <div> for text — matches block definitions, ensures editable works
      let s = `font-size:${sz}px;font-family:${esc(node.fontFamily as string || 'Inter')},sans-serif;padding:4px;`;
      if (node.color) s += `color:${colorToCss(node.color as any)};`;
      else s += 'color:#1e293b;';
      if (node.background_color) s += `background-color:${colorToCss(node.background_color as any)};`;
      if (node.width) s += `max-width:${node.width}px;`;
      if (node.bold || (sz as number) >= 18) s += 'font-weight:bold;';
      if (node.italic) s += 'font-style:italic;';
      if (node.align) s += `text-align:${node.align};`;
      if (node.lineHeight) s += `line-height:${node.lineHeight};`;
      if (node.letterSpacing) s += `letter-spacing:${node.letterSpacing}px;`;
      if (node.opacity !== undefined) s += `opacity:${node.opacity};`;
      if (node.rotation) s += `transform:rotate(${node.rotation}deg);`;
      if (node.borderWidth) s += `border:${node.borderWidth}px solid ${node.borderColor ? colorToCss(node.borderColor as any) : '#d1d5db'};`;
      const sizeAttr = (sz as number) >= 18 ? ` data-pdf-size="${sz}"` : '';
      return `<div data-pdf-type="Text"${sizeAttr} style="${s}">${esc(node.content as string || '')}</div>`;
    }
    case 'Image': {
      const w = node.width || 200, h = node.height || 150;
      const src = node.src as string || '';
      const resolved = assets.get(src) || src;
      const hasImg = resolved && (resolved.startsWith('blob:') || resolved.startsWith('data:') || resolved.startsWith('http'));
      let s = `width:${w}px;height:${h}px;`;
      if (node.opacity !== undefined) s += `opacity:${node.opacity};`;
      if (node.rotation) s += `transform:rotate(${node.rotation}deg);`;
      if (node.borderWidth) s += `border:${node.borderWidth}px solid ${node.borderColor ? colorToCss(node.borderColor as any) : '#d1d5db'};`;
      if (hasImg) {
        return `<div data-pdf-type="Image" data-pdf-src="${ea(src)}" style="${s}overflow:hidden;"><img src="${ea(resolved)}" style="width:100%;height:100%;object-fit:contain;pointer-events:none;" /></div>`;
      }
      return `<div data-pdf-type="Image" data-pdf-src="${ea(src)}" style="${s}background:#f1f5f9;display:flex;align-items:center;justify-content:center;border:2px dashed #cbd5e1;font-size:11px;color:#64748b;border-radius:4px;">📷 Click upload in toolbar</div>`;
    }
    case 'Table': {
      const cols = (node.columns as any[]) || [];
      const rows = (node.rows as string[][]) || [];
      const st = (node.style as any) || {};
      const hBg = st.header_bg || '#1e293b', hC = st.header_color || '#fff';
      const bC = st.border_color || '#e2e8f0', cP = st.cell_padding || 10, fS = st.font_size || 12;
      const striped = st.striped !== false;
      const thS = `border:1px solid ${bC};padding:${cP}px;background:${hBg};color:${hC};font-weight:600;text-align:left;font-size:${fS}px;`;
      const tdS = (ri: number) => `border:1px solid ${bC};padding:${cP}px;font-size:${fS}px;${striped && ri % 2 === 1 ? 'background:#f8fafc;' : ''}`;
      const ths = cols.map((c: any) => {
        const wS = c.width ? `width:${c.width}px;` : '';
        return `<th style="${thS}${wS}">${esc(c.header)}</th>`;
      }).join('');
      const trs = rows.map((r: string[], ri: number) => `<tr>${r.map(cell => `<td style="${tdS(ri)}">${esc(cell)}</td>`).join('')}</tr>`).join('');
      return `<table data-pdf-type="Table" data-header-bg="${hBg}" data-header-color="${hC}" data-border-color="${bC}" data-cell-padding="${cP}" data-font-size="${fS}" data-striped="${striped}" style="width:100%;border-collapse:collapse;font-size:${fS}px;table-layout:fixed;"><thead><tr>${ths}</tr></thead><tbody>${trs}</tbody></table>`;
    }
    case 'Rectangle': {
      let s = `width:${node.width || 120}px;height:${node.height || 80}px;background-color:${colorToCss(node.fill as any) || '#3b82f6'};`;
      if (node.strokeWidth) s += `border:${node.strokeWidth}px solid ${node.strokeColor ? colorToCss(node.strokeColor as any) : '#1e40af'};`;
      if (node.opacity !== undefined) s += `opacity:${node.opacity};`;
      if (node.rotation) s += `transform:rotate(${node.rotation}deg);`;
      if (node.borderRadius) s += `border-radius:${node.borderRadius}px;`;
      return `<div data-pdf-type="Rectangle" style="${s}"></div>`;
    }
    case 'Circle': {
      let s = `width:${node.width || 80}px;height:${node.height || 80}px;background-color:${colorToCss(node.fill as any) || '#10b981'};border-radius:50%;`;
      if (node.strokeWidth) s += `border:${node.strokeWidth}px solid ${node.strokeColor ? colorToCss(node.strokeColor as any) : '#047857'};`;
      if (node.opacity !== undefined) s += `opacity:${node.opacity};`;
      return `<div data-pdf-type="Circle" style="${s}"></div>`;
    }
    case 'Line': {
      return `<hr data-pdf-type="Line" style="width:${node.width || 200}px;border:none;border-top:${node.thickness || 2}px solid ${colorToCss(node.color as any) || '#334155'};margin:8px 0;" />`;
    }
    case 'Header': {
      const kids = ((node.children as TemplateNode[]) || []).map(n => nodeToHtml(n, assets)).join('');
      return `<div data-pdf-type="Header" style="width:100%;padding:12px;border-bottom:2px solid #e2e8f0;background:#f8fafc;min-height:40px;">${kids}</div>`;
    }
    case 'Footer': {
      const kids = ((node.children as TemplateNode[]) || []).map(n => nodeToHtml(n, assets)).join('');
      return `<div data-pdf-type="Footer" style="width:100%;padding:12px;border-top:2px solid #e2e8f0;background:#f8fafc;min-height:40px;">${kids}</div>`;
    }
    case 'PageNumber':
      return `<div data-pdf-type="PageNumber" style="font-size:${node.size || 10}px;color:#6b7280;text-align:${node.align || 'center'};">${esc(node.format as string || 'Page {page}')}</div>`;
    case 'DynamicText':
      return `<span data-pdf-type="DynamicText" data-binding="${ea(node.binding as string || '')}" style="font-size:${node.size || 'inherit'};color:inherit;font-family:inherit;font-style:italic;">{{${esc(node.binding as string || 'field')}}}</span>`;
    case 'Hyperlink':
      return `<a data-pdf-type="Hyperlink" href="${ea(node.href as string || '#')}" style="color:#2563eb;font-size:${node.size || 12}px;text-decoration:underline;">${esc(node.text as string || 'Link')}</a>`;
    default:
      return '';
  }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

function pf(v: string | undefined): number { return parseFloat(String(v || '0').replace(/[^\d.-]/g, '')) || 0; }
function esc(s: string): string { return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;'); }
function ea(s: string): string { return s.replace(/"/g, '&quot;').replace(/&/g, '&amp;'); }

function parseRotation(t: string): number {
  const m = t.match(/rotate\(([-\d.]+)deg\)/);
  return m ? parseFloat(m[1]) : 0;
}

function cssToColor(css: string): { r: number; g: number; b: number; a?: number } | undefined {
  if (!css) return undefined;
  const rgb = css.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
  if (rgb) return { r: +rgb[1] / 255, g: +rgb[2] / 255, b: +rgb[3] / 255, a: rgb[4] ? +rgb[4] : undefined };
  const hex = css.match(/^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
  if (hex) return { r: parseInt(hex[1], 16) / 255, g: parseInt(hex[2], 16) / 255, b: parseInt(hex[3], 16) / 255 };
  return undefined;
}

function colorToCss(c: any): string {
  if (!c) return '';
  const r = Math.round((c.r || 0) * 255), g = Math.round((c.g || 0) * 255), b = Math.round((c.b || 0) * 255);
  if (c.a !== undefined && c.a < 1) return `rgba(${r},${g},${b},${c.a})`;
  return `rgb(${r},${g},${b})`;
}
