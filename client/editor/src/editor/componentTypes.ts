/**
 * PDFCore — Custom GrapesJS Component Types
 *
 * Every type sets default attributes { 'data-pdf-type': '...' } so components
 * created via block definitions (JSON) or HTML both get the right attrs.
 *
 * isComponent detects from HTML → { type: 'registeredName' }
 * Block definitions can reference types directly.
 */
import type grapesjs from 'grapesjs';
type Editor = grapesjs.Editor;

export function registerComponentTypes(editor: Editor) {
  const dc = editor.DomComponents;

  const moveDeleteToolbar = [
    { attributes: { class: 'fa fa-arrows', title: 'Move' }, command: 'tlb-move' },
    { attributes: { class: 'fa fa-trash-o', title: 'Delete' }, command: 'tlb-delete' },
  ];

  const layoutResizable = {
    tl: 0, tc: 0, tr: 0,
    bl: 0, bc: 1, br: 1,
    cl: 0, cr: 1,
    minDim: 30,
  };

  // ═══ PAGE ROOT ═══════════════════════════════════════════════════════
  dc.addType('pdf-page-root', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'PageRoot') return { type: 'pdf-page-root' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'PageRoot' },
        removable: false,
        draggable: false,
        copyable: false,
        droppable: true,
        selectable: true,
        hoverable: true,
        resizable: false,
        toolbar: [],
      },
    },
  });

  // ═══ TEXT ═════════════════════════════════════════════════════════════
  dc.addType('pdf-text-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Text') return { type: 'pdf-text-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Text' },
        editable: true,
        droppable: false,
        draggable: true,
        removable: true,
        resizable: {
          tl: 0, tc: 0, tr: 0,
          bl: 0, bc: 0, br: 1,
          cl: 0, cr: 1,
          minDim: 20,
        } as any,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  // ═══ IMAGE ═══════════════════════════════════════════════════════════
  dc.addType('pdf-image-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Image') return { type: 'pdf-image-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Image' },
        droppable: false,
        resizable: {
          tl: 0, tc: 0, tr: 0,
          bl: 0, bc: 1, br: 1,
          cl: 0, cr: 1,
          ratioDefault: false,
          minDim: 40,
        } as any,
        toolbar: [
          { attributes: { class: 'fa fa-arrows', title: 'Move' }, command: 'tlb-move' },
          { attributes: { class: 'fa fa-upload', title: 'Upload Image' }, command: 'pdf-upload-image' },
          { attributes: { class: 'fa fa-trash-o', title: 'Delete' }, command: 'tlb-delete' },
        ],
      },
    },
  });

  // ═══ TABLE ═══════════════════════════════════════════════════════════
  dc.addType('pdf-table-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Table') return { type: 'pdf-table-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'table',
        attributes: { 'data-pdf-type': 'Table' },
        droppable: false,
        editable: true,
        toolbar: [
          { attributes: { class: 'fa fa-arrows', title: 'Move' }, command: 'tlb-move' },
          { attributes: { class: 'fa fa-plus-square', title: 'Add Column' }, command: 'table-add-col' },
          { attributes: { class: 'fa fa-minus-square', title: 'Remove Column' }, command: 'table-remove-col' },
          { attributes: { class: 'fa fa-plus-circle', title: 'Add Row' }, command: 'table-add-row' },
          { attributes: { class: 'fa fa-minus-circle', title: 'Remove Row' }, command: 'table-remove-row' },
          { attributes: { class: 'fa fa-trash-o', title: 'Delete' }, command: 'tlb-delete' },
        ],
      },
    },
  });

  // ═══ SHAPES ══════════════════════════════════════════════════════════
  dc.addType('pdf-rect-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Rectangle') return { type: 'pdf-rect-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Rectangle' },
        droppable: false,
        resizable: { tl: 0, tc: 0, tr: 0, bl: 0, bc: 1, br: 1, cl: 0, cr: 1, minDim: 20 } as any,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  dc.addType('pdf-circle-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Circle') return { type: 'pdf-circle-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Circle' },
        droppable: false,
        resizable: { tl: 0, tc: 0, tr: 0, bl: 0, bc: 1, br: 1, cl: 0, cr: 1, ratioDefault: true, minDim: 20 } as any,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  dc.addType('pdf-line-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Line') return { type: 'pdf-line-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'hr',
        attributes: { 'data-pdf-type': 'Line' },
        droppable: false,
        resizable: { tl: 0, tc: 0, tr: 0, bl: 0, bc: 0, br: 0, cl: 0, cr: 1, minDim: 20 } as any,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  // ═══ LAYOUT ══════════════════════════════════════════════════════════
  dc.addType('pdf-column-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Column') return { type: 'pdf-column-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Column' },
        droppable: true,
        removable: true,
        resizable: layoutResizable as any,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  dc.addType('pdf-row-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Row') return { type: 'pdf-row-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Row' },
        droppable: true,
        removable: true,
        resizable: layoutResizable as any,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  dc.addType('pdf-container-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Container') return { type: 'pdf-container-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Container' },
        droppable: true,
        removable: true,
        resizable: layoutResizable as any,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  // ═══ HEADER / FOOTER ═════════════════════════════════════════════════
  dc.addType('pdf-header-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Header') return { type: 'pdf-header-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Header' },
        droppable: true,
        removable: true,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  dc.addType('pdf-footer-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Footer') return { type: 'pdf-footer-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'Footer' },
        droppable: true,
        removable: true,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  // ═══ PAGE NUMBER / DYNAMIC TEXT / HYPERLINK ═══════════════════════════
  dc.addType('pdf-pagenumber-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'PageNumber') return { type: 'pdf-pagenumber-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'div',
        attributes: { 'data-pdf-type': 'PageNumber' },
        editable: true,
        droppable: false,
        toolbar: moveDeleteToolbar,
      },
    },
  });

  dc.addType('pdf-dynamic-text-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'DynamicText') return { type: 'pdf-dynamic-text-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'span',
        attributes: { 'data-pdf-type': 'DynamicText' },
        editable: false,
        droppable: false,
        resizable: { tl: 0, tc: 0, tr: 0, bl: 0, bc: 0, br: 1, cl: 0, cr: 1, minDim: 30 } as any,
        toolbar: [
          ...moveDeleteToolbar,
          { attributes: { class: 'fa fa-database', title: 'Edit Binding' }, command: 'pdf-edit-binding' },
        ],
      },
    },
  });

  dc.addType('pdf-hyperlink-comp', {
    isComponent: (el: any) => {
      if (el.getAttribute?.('data-pdf-type') === 'Hyperlink') return { type: 'pdf-hyperlink-comp' };
      return false;
    },
    model: {
      defaults: {
        tagName: 'a',
        attributes: { 'data-pdf-type': 'Hyperlink' },
        editable: true,
        droppable: false,
        toolbar: moveDeleteToolbar,
      },
    },
  });
}

// ═══════════════════════════════════════════════════════════════════════════
//  COMMANDS
// ═══════════════════════════════════════════════════════════════════════════

export function registerEditorCommands(editor: Editor) {
  const cmds = editor.Commands as any;

  cmds.add('pdf-upload-image', {
    run(ed: any) {
      const selected = ed.getSelected();
      if (!selected) return;
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = 'image/jpeg,image/png,image/tiff,image/svg+xml';
      input.onchange = () => {
        const file = input.files?.[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = () => {
          const dataUrl = reader.result as string;
          selected.addAttributes({ 'data-pdf-src': file.name });
          selected.components(`<img src="${dataUrl}" style="width:100%;height:100%;object-fit:contain;pointer-events:none;" />`);
          selected.setStyle({
            ...selected.getStyle(),
            background: 'transparent',
            display: 'block',
          });
        };
        reader.readAsDataURL(file);
      };
      input.click();
    },
  });

  cmds.add('table-add-col', {
    run(ed: any) {
      const sel = ed.getSelected();
      const el = sel?.view?.el;
      if (!el || el.tagName !== 'TABLE') return;
      el.querySelectorAll('tr').forEach((tr: HTMLElement) => {
        const isHeader = tr.parentElement?.tagName === 'THEAD';
        const cell = document.createElement(isHeader ? 'th' : 'td');
        cell.textContent = isHeader ? `Col ${tr.children.length + 1}` : 'Data';
        tr.appendChild(cell);
      });
      restyleAndSync(sel, el, ed);
    },
  });

  cmds.add('table-remove-col', {
    run(ed: any) {
      const sel = ed.getSelected();
      const el = sel?.view?.el;
      if (!el || el.tagName !== 'TABLE') return;
      let removed = false;
      el.querySelectorAll('tr').forEach((tr: HTMLElement) => {
        if (tr.children.length > 1) { tr.removeChild(tr.lastElementChild!); removed = true; }
      });
      if (removed) restyleAndSync(sel, el, ed);
    },
  });

  cmds.add('table-add-row', {
    run(ed: any) {
      const sel = ed.getSelected();
      const el = sel?.view?.el;
      if (!el || el.tagName !== 'TABLE') return;
      const tbody = el.querySelector('tbody');
      if (!tbody) return;
      const firstRow = tbody.querySelector('tr');
      const colCount = firstRow ? firstRow.children.length : 3;
      const tr = document.createElement('tr');
      for (let i = 0; i < colCount; i++) {
        const td = document.createElement('td');
        td.textContent = 'Data';
        tr.appendChild(td);
      }
      tbody.appendChild(tr);
      restyleAndSync(sel, el, ed);
    },
  });

  cmds.add('table-remove-row', {
    run(ed: any) {
      const sel = ed.getSelected();
      const el = sel?.view?.el;
      if (!el || el.tagName !== 'TABLE') return;
      const tbody = el.querySelector('tbody');
      if (!tbody) return;
      const rows = tbody.querySelectorAll('tr');
      if (rows.length > 1) {
        tbody.removeChild(rows[rows.length - 1]);
        restyleAndSync(sel, el, ed);
      }
    },
  });
}

/**
 * Re-apply all cell styles uniformly, then sync HTML back to GrapesJS.
 * This ensures consistent styling after ANY add/remove operation:
 * - Header cells: correct bg, color, font-weight
 * - Body cells: correct border, padding, font-size, striped backgrounds
 */
function restyleAndSync(sel: any, el: HTMLElement, ed: any) {
  const attrs = sel.getAttributes();
  const borderColor = attrs['data-border-color'] || '#e2e8f0';
  const cellPad = attrs['data-cell-padding'] || '10';
  const fontSize = attrs['data-font-size'] || '12';
  const headerBg = attrs['data-header-bg'] || '#1e293b';
  const headerColor = attrs['data-header-color'] || '#ffffff';
  const striped = attrs['data-striped'] !== 'false';

  // Style ALL header cells
  el.querySelectorAll('thead th').forEach((th: Element) => {
    (th as HTMLElement).style.cssText =
      `border:1px solid ${borderColor};padding:${cellPad}px;background:${headerBg};color:${headerColor};font-weight:600;text-align:left;font-size:${fontSize}px;`;
  });

  // Style ALL body cells with stripe alternation
  const bodyRows = el.querySelectorAll('tbody tr');
  bodyRows.forEach((tr: Element, rowIdx: number) => {
    const stripeBg = striped && rowIdx % 2 === 1 ? 'background:#f8fafc;' : '';
    tr.querySelectorAll('td').forEach((td: Element) => {
      (td as HTMLElement).style.cssText =
        `border:1px solid ${borderColor};padding:${cellPad}px;font-size:${fontSize}px;${stripeBg}`;
    });
  });

  // Sync: replace GrapesJS component with updated HTML
  const html = el.outerHTML;
  const parent = sel.parent();
  const idx = parent.components().indexOf(sel);
  sel.remove();
  const newComps = parent.components().add(html, { at: idx });
  // Re-select the new table component
  if (newComps) {
    const newComp = Array.isArray(newComps) ? newComps[0] : newComps;
    if (newComp) setTimeout(() => ed.select(newComp), 50);
  }
}
