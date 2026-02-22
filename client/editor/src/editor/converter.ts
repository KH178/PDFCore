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
  let header: TemplateNode | undefined;
  let footer: TemplateNode | undefined;
  let margins = { top: 40, bottom: 40, left: 40, right: 40 };

  if (pageRoot) {
    const rawChildren = pageRoot.components().models;
    const headerComp = rawChildren.find((c: any) => c.getAttributes()['data-pdf-type'] === 'Header');
    const footerComp = rawChildren.find((c: any) => c.getAttributes()['data-pdf-type'] === 'Footer');
    
    if (headerComp) header = componentToNode(headerComp, editor) || undefined;
    if (footerComp) footer = componentToNode(footerComp, editor) || undefined;
    
    children = rawChildren
      .filter((c: any) => {
         const t = c.getAttributes()['data-pdf-type'];
         return t !== 'Header' && t !== 'Footer';
      })
      .map((c: any) => componentToNode(c, editor))
      .filter(Boolean) as TemplateNode[];
      
    // Extract margins from PageRoot (actually uses padding as margins internally)
    const marginsDef = extractSpacing(pageRoot as Component, 'padding', pageRoot.getStyle());
    margins = marginsDef || { top: 40, right: 40, bottom: 40, left: 40 };
  } else {
    // Fallback: no PageRoot, use wrapper children directly
    children = wrapper.components().models
      .map((c: any) => componentToNode(c, editor))
      .filter(Boolean) as TemplateNode[];
  }

  const device = editor.getDevice();
  let size: 'A4' | 'Letter' | 'Legal' | 'Custom' = 'A4';
  let orientation: 'portrait' | 'landscape' = 'portrait';
  if (device === 'A4 Landscape') { size = 'A4'; orientation = 'landscape'; }
  else if (device === 'Letter') { size = 'Letter'; orientation = 'portrait'; }

  // Header/Footer: inline into root column (Rust Template struct has no header/footer fields)
  // We put header at top, footer at bottom of the root column.
  const allChildren: TemplateNode[] = [
    ...(header ? [header] : []),
    ...children,
    ...(footer ? [footer] : []),
  ];

  return {
    root: allChildren.length === 1 ? allChildren[0] : { type: 'Column', children: allChildren },
    manifest: { name: 'Untitled Template', version: '1.2' },
    styles: {},
    settings: {
      size,
      orientation,
      margins,
    },
  };
}

function getFullStyle(comp: Component, editor: Editor): Record<string, string> {
  const style = { ...comp.getStyle() };
  try {
    comp.getClasses().forEach((cls: string) => {
      const rule = editor.Css.getRule(`.${cls}`);
      if (rule) Object.assign(style, rule.getStyle());
    });
    const idRule = editor.Css.getRule(`#${comp.getId()}`);
    if (idRule) Object.assign(style, idRule.getStyle());
    Object.assign(style, comp.getStyle());
    
    // Fallback: If font-size or color are purely inherited and missing, read computed DOM style safely
    if (comp.view && (comp.view as any).el) {
       const el = (comp.view as any).el;
       const win = el.ownerDocument?.defaultView || window;
       const computed = win.getComputedStyle(el);
       if (!style['font-size'] && computed.fontSize) style['font-size'] = computed.fontSize;
       if (!style['color'] && computed.color) style['color'] = computed.color;
       if (!style['background-color'] && computed.backgroundColor && computed.backgroundColor !== 'rgba(0, 0, 0, 0)') {
         style['background-color'] = computed.backgroundColor;
       }
       if (!style['padding'] && computed.padding && computed.padding !== '0px') {
         style['padding'] = computed.padding;
       }
    }
  } catch(e) {
    console.error('Style extraction error:', e);
  }
  return style;
}

function componentToNode(comp: Component, editor: Editor): TemplateNode | null {
  const attrs = comp.getAttributes();
  const pdfType = attrs['data-pdf-type'];

  if (!pdfType) {
    const children = comp.components().models;
    if (children.length === 0) return null;
    if (children.length === 1) return componentToNode(children[0], editor);
    return {
      type: 'Column',
      children: children.map(c => componentToNode(c, editor)).filter(Boolean) as TemplateNode[],
    };
  }

  const style = getFullStyle(comp, editor);

  switch (pdfType) {
    case 'PageRoot': {
      const kids = getContentChildren(comp, editor);
      return kids.length === 1 ? kids[0] : { type: 'Column', children: kids };
    }

    case 'Column': {
      const el = (comp.view as any)?.el;
      const compStyle = el ? (el.ownerDocument?.defaultView || window).getComputedStyle(el) : null;
      
      const padding = extractSpacing(comp, 'padding', style);
      const margin = extractSpacing(comp, 'margin', style);
      const bg = cssToColor(style['background-color']);
      const border = extractSpacing(comp, 'border-width', style);
      const borderColor = style['border-color'] ? cssToColor(style['border-color']) : (compStyle?.borderColor ? cssToColor(compStyle.borderColor) : undefined);
      const borderRadius = compStyle?.borderRadius ? pf(compStyle.borderRadius) : pf(style['border-radius']);
      const computedWidth = el ? el.getBoundingClientRect().width : undefined;
      
      const col = {
        type: 'Column',
        children: getContentChildren(comp, editor),
        spacing: pf(style['gap']),
      };
      if (!isZero(padding) || !isZero(margin) || bg || !isZero(border) || computedWidth) {
        return { type: 'Container', child: col, padding: spacingMapToStr(padding), margin: spacingMapToStr(margin), background_color: bg, border: spacingMapToStr(border), border_color: borderColor, border_radius: borderRadius, width: computedWidth };
      }
      return col;
    }

    case 'Row': {
      const el = (comp.view as any)?.el;
      const compStyle = el ? (el.ownerDocument?.defaultView || window).getComputedStyle(el) : null;
      
      const padding = extractSpacing(comp, 'padding', style);
      const margin = extractSpacing(comp, 'margin', style);
      const bg = cssToColor(style['background-color']);
      const border = extractSpacing(comp, 'border-width', style);
      const borderColor = style['border-color'] ? cssToColor(style['border-color']) : (compStyle?.borderColor ? cssToColor(compStyle.borderColor) : undefined);
      const borderRadius = compStyle?.borderRadius ? pf(compStyle.borderRadius) : pf(style['border-radius']);
      const computedHeight = el ? el.getBoundingClientRect().height : undefined;
      
      const row = {
        type: 'Row',
        children: getContentChildren(comp, editor),
        spacing: pf(style['gap']),
        justifyContent: style['justify-content'],
        alignItems: style['align-items'],
      };
      if (!isZero(padding) || !isZero(margin) || bg || !isZero(border) || computedHeight) {
        return { type: 'Container', child: row, padding: spacingMapToStr(padding), margin: spacingMapToStr(margin), background_color: bg, border: spacingMapToStr(border), border_color: borderColor, border_radius: borderRadius, height: computedHeight };
      }
      return row;
    }

    case 'Container': {
      const real = getContentChildren(comp, editor);
      const f = parseFlex(style['flex']);
      return {
        type: 'Container',
        child: real.length === 1 ? real[0] : { type: 'Column', children: real },
        padding: spacingMapToStr(extractSpacing(comp, 'padding', style)),
        margin: spacingMapToStr(extractSpacing(comp, 'margin', style)),
        border: spacingMapToStr(extractSpacing(comp, 'border-width', style)),
        borderColor: style['border-color'] ? cssToColor(style['border-color']) : undefined,
        background_color: cssToColor(style['background-color']),
        width: style['width'],
        height: style['height'],
        radius: pf(style['border-radius']),
        flexGrow: style['flex-grow'] ? parseFloat(style['flex-grow']) : f?.grow,
        flexShrink: style['flex-shrink'] ? parseFloat(style['flex-shrink']) : f?.shrink,
        flexBasis: style['flex-basis'] || f?.basis,
      };
    }

    case 'Text': {
      const el = (comp.view as any)?.el;
      const text = el?.textContent?.trim() || String(comp.get('content') || '').trim();
      const node: any = {
        type: 'Text',
        content: text,
        size: attrs['data-pdf-size'] ? parseFloat(attrs['data-pdf-size']) : pf(style['font-size']) || 12,
      };
      if (style['color'] && style['color'] !== 'inherit') node.color = cssToColor(style['color']);
      if (style['font-weight'] === 'bold' || style['font-weight'] === '700') node.bold = true;
      if (style['font-style'] === 'italic') node.italic = true;
      if (style['text-align']) node.align = style['text-align'];
      if (style['font-family']) node.fontFamily = style['font-family'].replace(/['"]/g, '');
      if (style['line-height'] && style['line-height'] !== 'normal') node.lineHeight = parseFloat(style['line-height']);
      if (style['letter-spacing']) node.letterSpacing = pf(style['letter-spacing']);
      
      const padding = extractSpacing(comp, 'padding', style);
      const margin = extractSpacing(comp, 'margin', style);
      const border = extractSpacing(comp, 'border-width', style);
      const bg = cssToColor(style['background-color']);
      
      if (!isZero(padding) || !isZero(margin) || !isZero(border) || bg) {
        return { type: 'Container', child: node, padding: spacingMapToStr(padding), margin: spacingMapToStr(margin), border: spacingMapToStr(border), background_color: bg, width: style['max-width'] ? pf(style['max-width']) : undefined };
      }
      if (style['max-width']) node.width = pf(style['max-width']);
      return node as TemplateNode;
    }

    case 'Image': {
      // IMPORTANT: src must match the asset key used in add_asset (data-pdf-src), NOT the raw data URL
      const src = attrs['data-pdf-src'] || '';
      const el = (comp.view as any)?.el;
      // Use explicit style width/height set by the user, fallback to element bounds, then defaults
      const w = pf(style['width']) || (el ? el.getBoundingClientRect().width : 0) || 200;
      const h = pf(style['height']) || (el ? el.getBoundingClientRect().height : 0) || 150;
      const node = {
        type: 'Image',
        src,
        width: w,
        height: h,
      };
      
      const margin = extractSpacing(comp, 'margin', style);
      if (!isZero(margin)) {
        return { type: 'Container', child: node as TemplateNode, padding: spacingMapToStr(margin) };
      }
      return node as TemplateNode;
    }

    case 'Table': {
      const el = (comp.view as any)?.el;
      if (!el) return { type: 'Table', columns: [], rows: [] };
      const rawCols = Array.from(el.querySelectorAll('thead th')).map((th: any) => ({
        header: th.textContent || '',
        width: (th.style.width && parseInt(th.style.width)) || th.getBoundingClientRect().width || 100,
      }));
      // Standard A4 width is 595. With default margins (40*2), max width is ~515.
      const totalW = rawCols.reduce((sum, col) => sum + col.width, 0);
      const scale = totalW > 515 ? 515 / totalW : 1;
      
      const node = {
        type: 'Table',
        columns: rawCols.map(c => ({ header: c.header, width: c.width * scale })),
        rows: Array.from(el.querySelectorAll('tbody tr')).map((tr: any) =>
          Array.from(tr.querySelectorAll('td')).map((td: any) => ({
             content: td.textContent || '',
             colspan: parseInt(td.getAttribute('colspan') || '1', 10),
             rowspan: parseInt(td.getAttribute('rowspan') || '1', 10),
          }))
        ),
        settings: {
          padding: parseInt(attrs['data-cell-padding'] || '10'),
          font_size: parseInt(attrs['data-font-size'] || '12'),
          header_height: 30,
          cell_height: 20,
          border_width: 1,
          header_bg: cssToColor(attrs['data-header-bg']) || { r: 30/255, g: 41/255, b: 59/255 },
          header_color: cssToColor(attrs['data-header-color']) || { r: 1, g: 1, b: 1 },
          border_color: cssToColor(attrs['data-border-color']) || { r: 226/255, g: 232/255, b: 240/255 },
          striped: attrs['data-striped'] !== 'false',
          alternate_row_color: { r: 248/255, g: 250/255, b: 252/255 },
        },
      };
      const margin = extractSpacing(comp, 'margin', style);
      if (!isZero(margin)) {
         return { type: 'Container', child: node as TemplateNode, padding: spacingMapToStr(margin) };
      }
      return node as TemplateNode;
    }

    case 'Rectangle':
    case 'Circle': {
      const el = (comp.view as any)?.el;
      const w = el ? el.getBoundingClientRect().width : (pf(style['width']) || 120);
      const h = el ? el.getBoundingClientRect().height : (pf(style['height']) || 80);
      const compStyle = el ? (el.ownerDocument?.defaultView || window).getComputedStyle(el) : null;
      let br = compStyle?.borderRadius ? pf(compStyle.borderRadius) : pf(style['border-radius']);
      
      const borderColor = style['border-color'] ? cssToColor(style['border-color']) : (compStyle?.borderColor ? cssToColor(compStyle.borderColor) : undefined);
      
      return {
        type: 'Container',
        child: { type: 'Column', children: [] },
        width: w,
        height: h,
        background_color: cssToColor(style['background-color'] || '#3b82f6'),
        border: spacingMapToStr(extractSpacing(comp, 'border-width', style)),
        border_color: borderColor,
        border_radius: br,
        margin: spacingMapToStr(extractSpacing(comp, 'margin', style)),
        padding: spacingMapToStr(extractSpacing(comp, 'padding', style)),
      };
    }

    case 'Line': {
      const el = (comp.view as any)?.el;
      const w = el ? el.getBoundingClientRect().width : (pf(style['width']) || 200);
      
      return {
        type: 'Container',
        child: { type: 'Column', children: [] },
        width: w,
        height: pf(style['border-top-width'] || style['height']) || 2,
        background_color: cssToColor(style['border-top-color'] || style['border-color'] || '#334155'),
      };
    }

    case 'Header':
    case 'Footer': {
      const el = (comp.view as any)?.el;
      const h = el ? el.getBoundingClientRect().height : 0;
      const bg = cssToColor(style['background-color']);
      const borderW = extractSpacing(comp, 'border-width', style);
      const borderColor = style['border-color'] ? cssToColor(style['border-color']) : undefined;
      const children = getContentChildren(comp, editor);
      return {
        type: 'Container',
        child: { type: 'Column', children },
        padding: spacingMapToStr(extractSpacing(comp, 'padding', style)) ?? 12,
        height: h > 0 ? h : undefined,
        background_color: bg,
        border: spacingMapToStr(borderW),
        border_color: borderColor,
      };
    }


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
        type: 'Text',
        content: `{{${attrs['data-binding'] || 'field'}}}`,
        size: pf(style['font-size']) || 12,
      };

    case 'Hyperlink': {
      const el = (comp.view as any)?.el;
      const linkText = el?.textContent?.trim() || 'Link';
      const url = attrs['href'] || '#';
      return {
        type: 'Text',
        content: `${linkText} (${url})`,
        size: pf(style['font-size']) || 12,
        color: { r: 37/255, g: 99/255, b: 235/255 }
      };
    }

    case 'PageBreak':
      // Emit a large transparent spacer that forces the layout engine to split onto next page
      return {
        type: 'Container',
        child: { type: 'Column', children: [] },
        height: 9999,
        padding: 0,
      };

    default:
      return null;
  }
}

function getContentChildren(comp: Component, editor: Editor): TemplateNode[] {
  return comp.components().models
    .filter((c: Component) => {
      const tag = c.get('tagName');
      if (tag === 'span' && !c.getAttributes()['data-pdf-type']) return false;
      return true;
    })
    .map(c => componentToNode(c, editor))
    .filter(Boolean) as TemplateNode[];
}

// ─── IMPORT ─────────────────────────────────────────────────────────────────

export function templateToEditor(editor: Editor, template: PDFCoreTemplate, assetUrls?: Map<string, string>) {
  const html = nodeToHtml(template.root, assetUrls || new Map());
  // Use proper PageRoot wrapper matching the component type definition
  const settings = (template as any).settings || {};
  const margins = settings.margins || { top: 40, bottom: 40, left: 40, right: 40 };
  let width = settings.size === 'Letter' ? 612 : 595;
  let height = settings.size === 'Letter' ? 792 : 842;
  if (settings.orientation === 'landscape') {
      [width, height] = [height, width];
  }
  // We lock the PageRoot to the exact dimensions of the PDF page, scaled, with overflow hidden
  const pageHtml = `<div data-pdf-type="PageRoot" style="width:${width}px;min-height:${height}px;margin:0 auto;background:white;padding-top:${margins.top}px;padding-bottom:${margins.bottom}px;padding-left:${margins.left}px;padding-right:${margins.right}px;font-family:'Inter',sans-serif;box-sizing:border-box;box-shadow:0 10px 15px -3px rgb(0 0 0 / 0.1); overflow:hidden;">${html}</div>`;
  editor.DomComponents.getWrapper()?.components(pageHtml);
}

function nodeToHtml(node: TemplateNode, assets: Map<string, string>): string {
  switch (node.type) {
    case 'Column': {
      const kids = ((node.children as TemplateNode[]) || []).map(n => nodeToHtml(n, assets)).join('');
      let s = `display:flex;flex-direction:column;box-sizing:border-box;min-height:60px;padding:${node.padding || 12}px;border-radius:4px;${node.spacing ? `gap:${node.spacing}px;` : ''}`;
      // Restore flex/layout props
      if (node.width) s += `width:${px(node.width as any)};`; 
      else if (node.flexGrow === undefined && node.flexBasis === undefined) s += 'flex-grow:1;'; // Fallback for legacy
      
      if (node.flexGrow !== undefined) s += `flex-grow:${node.flexGrow};`;
      if (node.flexShrink !== undefined) s += `flex-shrink:${node.flexShrink};`;
      if (node.flexBasis) s += `flex-basis:${node.flexBasis};`;
      
      if (node.backgroundColor) s += `background-color:${colorToCss(node.backgroundColor as any)};`;
      if (node.borderWidth) s += `border:${node.borderWidth}px solid ${node.borderColor ? colorToCss(node.borderColor as any) : '#6366f1'};`;
      else s += 'outline:1px dashed #6366f1;outline-offset:-1px;'; // Fallback visual, uses outline so it's not exported
      
      return `<div data-pdf-type="Column" style="${s}">${kids}</div>`;
    }
    case 'Row': {
      const kids = ((node.children as TemplateNode[]) || []).map(n => nodeToHtml(n, assets)).join('');
      let s = `display:flex;flex-direction:row;flex-wrap:nowrap;box-sizing:border-box;padding:${node.padding || 12}px;border-radius:4px;${node.spacing ? `gap:${node.spacing}px;` : 'gap:8px;'}`;
      
      if (node.height) s += `min-height:${px(node.height as any)};`;
      else s += 'min-height:40px;';
      
      if (node.width) s += `width:${px(node.width as any)};`;
      else s += 'width:100%;';
      
      if (node.justifyContent) s += `justify-content:${node.justifyContent};`;
      if (node.alignItems) s += `align-items:${node.alignItems};`;
      if (node.backgroundColor) s += `background-color:${colorToCss(node.backgroundColor as any)};`;
      s += 'outline:1px dashed #0ea5e9;outline-offset:-1px;'; // visual helper, uses outline so it's not exported
      
      return `<div data-pdf-type="Row" style="${s}">${kids}</div>`;
    }
    case 'Container': {
      const child = node.child ? nodeToHtml(node.child as TemplateNode, assets) : '';
      let s = `box-sizing:border-box;padding:${node.padding || 16}px;border-radius:${node.radius || 4}px;`;
      
      if (node.border !== undefined) {
         if ((node.border as number) > 0) s += `border:${node.border}px solid ${node.borderColor ? colorToCss(node.borderColor as any) : '#d1d5db'};`;
         else s += 'border:none;';
      } else {
         s += `border:1px solid #d1d5db;`;
      }
      
      if (node.backgroundColor) s += `background-color:${colorToCss(node.backgroundColor as any)};`;
      if (node.width) s += `width:${px(node.width as any)};`;
      if (node.height) s += `height:${px(node.height as any)};`;
      else s += 'min-height:40px;';
      
      if (node.flexGrow !== undefined) s += `flex-grow:${node.flexGrow};`;
      if (node.flexShrink !== undefined) s += `flex-shrink:${node.flexShrink};`;
      if (node.flexBasis) s += `flex-basis:${node.flexBasis};`;
      
      return `<div data-pdf-type="Container" style="${s}">${child}</div>`;
    }
    case 'Text': {
      const sz = node.size || 12;
      // Always use <div> for text — matches block definitions, ensures editable works
      let s = `box-sizing:border-box;font-size:${sz}px;font-family:${esc(node.fontFamily as string || 'Inter')},sans-serif;padding:4px;`;
      if (node.color) s += `color:${colorToCss(node.color as any)};`;
      else s += 'color:#1e293b;';
      if (node.backgroundColor) s += `background-color:${colorToCss(node.backgroundColor as any)};`;
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
      let s = `box-sizing:border-box;width:${w}px;height:${h}px;`;
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
      const rows = node.rows as Array<Array<{content: string, colspan?: number, rowspan?: number}>> || [];
      const st = (node.style as any) || {};
      const hBg = st.header_bg || '#1e293b', hC = st.header_color || '#fff';
      const bC = st.border_color || '#e2e8f0', cP = st.cell_padding || 10, fS = st.font_size || 12;
      const striped = st.striped !== false;
      const thS = `border:1px solid ${bC};padding:${cP}px;background:${hBg};color:${hC};font-weight:600;text-align:left;font-size:${fS}px;`;
      const ths = cols.map((c: any) => {
        const wS = c.width ? `width:${c.width}px;` : '';
        return `<th style="${thS}${wS}">${esc(c.header)}</th>`;
      }).join('');
      const rowsStr = rows.map((row, ri) => {
        const cellsStr = row.map(cell => {
           let attrStr = '';
           if (cell.colspan && cell.colspan > 1) attrStr += ` colspan="${cell.colspan}"`;
           if (cell.rowspan && cell.rowspan > 1) attrStr += ` rowspan="${cell.rowspan}"`;
           const tdS = `border:1px solid ${bC};padding:${cP}px;font-size:${fS}px;${striped && ri % 2 === 1 ? 'background:#f8fafc;' : ''}`;
           return `<td style="${tdS}"${attrStr}>${esc(cell.content)}</td>`;
        }).join('');
        return `<tr>${cellsStr}</tr>`;
      }).join('');
      return `<table data-pdf-type="Table" data-header-bg="${hBg}" data-header-color="${hC}" data-border-color="${bC}" data-cell-padding="${cP}" data-font-size="${fS}" data-striped="${striped}" style="box-sizing:border-box;width:100%;border-collapse:collapse;font-size:${fS}px;table-layout:fixed;"><thead><tr>${ths}</tr></thead><tbody>${rowsStr}</tbody></table>`;
    }
    case 'Rectangle': {
      let s = `box-sizing:border-box;width:${node.width || 120}px;height:${node.height || 80}px;background-color:${colorToCss(node.fill as any) || '#3b82f6'};`;
      if (node.strokeWidth) s += `border:${node.strokeWidth}px solid ${node.strokeColor ? colorToCss(node.strokeColor as any) : '#1e40af'};`;
      if (node.opacity !== undefined) s += `opacity:${node.opacity};`;
      if (node.rotation) s += `transform:rotate(${node.rotation}deg);`;
      if (node.borderRadius) s += `border-radius:${node.borderRadius}px;`;
      return `<div data-pdf-type="Rectangle" style="${s}"></div>`;
    }
    case 'Circle': {
      let s = `box-sizing:border-box;width:${node.width || 80}px;height:${node.height || 80}px;background-color:${colorToCss(node.fill as any) || '#10b981'};border-radius:50%;`;
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

function px(val: string | number): string { return typeof val === 'number' ? `${val}px` : val; }
function pf(v: string | undefined): number { return parseFloat(String(v || '0').replace(/[^\d.-]/g, '')) || 0; }
function esc(s: string): string { return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;'); }
function ea(s: string): string { return s.replace(/"/g, '&quot;').replace(/&/g, '&amp;'); }

function isZero(sp: {top: number, right: number, bottom: number, left: number} | undefined): boolean {
   if (!sp) return true;
   return sp.top === 0 && sp.right === 0 && sp.bottom === 0 && sp.left === 0;
}

function spacingMapToStr(sp: {top: number, right: number, bottom: number, left: number} | undefined): number | undefined {
   if (!sp) return undefined;
   return sp.top; // Temporary mapping: core layout strictly expects numeric `f64` (Uniform padding fallback)
}

function extractSpacing(comp: Component, prop: 'margin'|'padding'|'border-width', style: Record<string, string>): { top: number, right: number, bottom: number, left: number } | undefined {
  let top = undefined, right = undefined, bottom = undefined, left = undefined;
  let hasAny = false;

  // 1. Try computed style first (most accurate 1:1 WYSIWYG pixels)
  if (comp.view && (comp.view as any).el) {
     const el = (comp.view as any).el;
     const win = el.ownerDocument?.defaultView || window;
     const computed = win.getComputedStyle(el);
     const prefix = prop === 'border-width' ? 'border' : prop;
     const suffix = prop === 'border-width' ? 'width' : '';
     
     top = pf(computed.getPropertyValue(`${prefix}-top${suffix ? '-' + suffix : ''}`));
     right = pf(computed.getPropertyValue(`${prefix}-right${suffix ? '-' + suffix : ''}`));
     bottom = pf(computed.getPropertyValue(`${prefix}-bottom${suffix ? '-' + suffix : ''}`));
     left = pf(computed.getPropertyValue(`${prefix}-left${suffix ? '-' + suffix : ''}`));
     
     return { top, right, bottom, left };
  }

  // 2. Fallback to style object
  const prefix = prop;
  if (style[prefix]) {
      const p = style[prefix].split(' ').map(pf);
      if (p.length === 1) { top = right = bottom = left = p[0]; hasAny = true; }
      else if (p.length === 2) { top = bottom = p[0]; right = left = p[1]; hasAny = true; }
      else if (p.length === 3) { top = p[0]; right = left = p[1]; bottom = p[2]; hasAny = true; }
      else if (p.length >= 4) { top = p[0]; right = p[1]; bottom = p[2]; left = p[3]; hasAny = true; }
  } else {
      top = pf(style[`${prefix}-top`]);
      right = pf(style[`${prefix}-right`]);
      bottom = pf(style[`${prefix}-bottom`]);
      left = pf(style[`${prefix}-left`]);
      if (top || right || bottom || left) hasAny = true;
  }

  return hasAny ? { top: top||0, right: right||0, bottom: bottom||0, left: left||0 } : undefined;
}

function cssToColor(css: string): { r: number; g: number; b: number; a?: number } | undefined {
  if (!css) return undefined;
  css = css.trim().toLowerCase();
  
  // Handle rgb/rgba
  const rgb = css.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*([\d.]+))?\s*\)/);
  if (rgb) return { r: +rgb[1] / 255, g: +rgb[2] / 255, b: +rgb[3] / 255, a: rgb[4] ? +rgb[4] : undefined };
  
  // Handle 6-digit hex
  const hex6 = css.match(/^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
  if (hex6) return { r: parseInt(hex6[1], 16) / 255, g: parseInt(hex6[2], 16) / 255, b: parseInt(hex6[3], 16) / 255 };
  
  // Handle 3-digit hex
  const hex3 = css.match(/^#([0-9a-f])([0-9a-f])([0-9a-f])$/i);
  if (hex3) return { r: parseInt(hex3[1]+hex3[1], 16) / 255, g: parseInt(hex3[2]+hex3[2], 16) / 255, b: parseInt(hex3[3]+hex3[3], 16) / 255 };
  
  // Handle named primary colors
  const named: Record<string, {r:number; g:number; b:number; a?:number}> = {
    'black': {r:0, g:0, b:0}, 'white': {r:1, g:1, b:1},
    'red': {r:1, g:0, b:0}, 'green': {r:0, g:0.5, b:0}, 'blue': {r:0, g:0, b:1},
    'transparent': {r:0, g:0, b:0, a:0}
  };
  if (named[css]) return named[css];
  
  return undefined;
}

function colorToCss(c: any): string {
  if (!c) return '';
  const r = Math.round((c.r || 0) * 255), g = Math.round((c.g || 0) * 255), b = Math.round((c.b || 0) * 255);
  if (c.a !== undefined && c.a < 1) return `rgba(${r},${g},${b},${c.a})`;
  return `rgb(${r},${g},${b})`;
}

function parseFlex(flex: string | undefined): { grow?: number; shrink?: number; basis?: string } | undefined {
  if (!flex) return undefined;
  const parts = flex.trim().split(/\s+/);
  if (parts.length === 1) {
    if (!isNaN(parseFloat(parts[0])) && !parts[0].includes('px') && !parts[0].includes('%')) {
       return { grow: parseFloat(parts[0]), shrink: 1, basis: '0%' }; // flex: 1 -> 1 1 0% (or 0%)
    }
     return { grow: 1, shrink: 1, basis: parts[0] };
  }
  return {
    grow: parts[0] ? parseFloat(parts[0]) : undefined,
    shrink: parts[1] ? parseFloat(parts[1]) : undefined,
    basis: parts[2]
  };
}
