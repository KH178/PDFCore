/**
 * PropertyInspector — Context-aware property panel
 *
 * - PageRoot: page padding, margin, background color (non-deletable)
 * - Text: font, size, weight, style, alignment, line-height, letter-spacing, colors
 * - Image: dimensions
 * - Table: header colors, cell padding, font size, border color
 * - Shapes: dimensions, fill, stroke
 * - Layout: display, flex-direction, gap
 * - EVERY component: display mode, padding, margin, background, border, opacity, rotation, size
 */
import { useState, useEffect, useCallback, useRef } from 'react';
import type grapesjs from 'grapesjs';
type Editor = grapesjs.Editor;

interface PropertyInspectorProps {
  editor: Editor | null;
}

interface CompState {
  pdfType: string | null;
  style: Record<string, string>;
  attrs: Record<string, string>;
  comp: any;
}

const EMPTY: CompState = { pdfType: null, style: {}, attrs: {}, comp: null };

export default function PropertyInspector({ editor }: PropertyInspectorProps) {
  const [state, setState] = useState<CompState>(EMPTY);
  const stateRef = useRef<CompState>(EMPTY);

  const readFromComp = useCallback((comp: any): CompState => {
    if (!comp) return EMPTY;
    const attrs = comp.getAttributes?.() || {};
    const pdfType = attrs['data-pdf-type'] || null;
    const rawStyle = comp.getStyle?.() || {};
    const style: Record<string, string> = {};
    for (const k of Object.keys(rawStyle)) {
      if (typeof rawStyle[k] === 'string') style[k] = rawStyle[k];
    }
    return { pdfType, style, attrs, comp };
  }, []);

  const refresh = useCallback((comp?: any) => {
    const c = comp || (editor as any)?.getSelected?.() || null;
    const s = readFromComp(c);
    stateRef.current = s;
    setState(s);
  }, [editor, readFromComp]);

  useEffect(() => {
    if (!editor) return;
    const onSelect = (comp: any) => refresh(comp);
    const onDeselect = () => { stateRef.current = EMPTY; setState(EMPTY); };
    const onUpdate = () => {
      const sel = (editor as any).getSelected?.();
      if (sel) refresh(sel);
    };

    editor.on('component:selected', onSelect);
    editor.on('component:deselected', onDeselect);
    (editor as any).on('component:styleUpdate', onUpdate);
    (editor as any).on('component:update', onUpdate);

    return () => {
      editor.off('component:selected', onSelect);
      editor.off('component:deselected', onDeselect);
      (editor as any).off('component:styleUpdate', onUpdate);
      (editor as any).off('component:update', onUpdate);
    };
  }, [editor, refresh]);

  // ─── Style/Attr writers ───────────────────────────────────────────────

  const setStyle = (key: string, value: string) => {
    const { comp } = stateRef.current;
    if (!comp) return;
    const current = { ...(comp.getStyle() || {}) };
    if (value === '' || value === 'unset') delete current[key];
    else current[key] = value;
    comp.setStyle(current);
    refresh(comp);
  };

  const setAttr = (key: string, value: string) => {
    const { comp } = stateRef.current;
    if (!comp) return;
    comp.addAttributes({ [key]: value });
    refresh(comp);
  };

  const { pdfType, style, attrs, comp } = state;

  // ═══ No selection ═══
  if (!pdfType) {
    return (
      <div className="p-4 text-xs text-gray-400 space-y-3">
        <div className="text-gray-300 font-bold text-sm">📄 No Selection</div>
        <p className="text-[11px] leading-relaxed text-gray-500">Click an element on the canvas to see its properties.</p>
        <div className="mt-4 p-3 bg-gray-800/50 rounded border border-gray-700/50">
          <p className="text-[10px] text-gray-500 font-medium mb-1">💡 Tips</p>
          <ul className="text-[10px] text-gray-600 space-y-0.5">
            <li>• Double-click text to edit inline</li>
            <li>• Drag blocks from the Blocks tab</li>
            <li>• Use Row for side-by-side layout</li>
            <li>• Press Delete to remove selected</li>
          </ul>
        </div>
      </div>
    );
  }

  const isPageRoot = pdfType === 'PageRoot';
  const isText = pdfType === 'Text';
  const isImage = pdfType === 'Image';
  const isTable = pdfType === 'Table';
  const isShape = pdfType === 'Rectangle' || pdfType === 'Circle';
  const isLine = pdfType === 'Line';
  const isLayout = pdfType === 'Column' || pdfType === 'Row' || pdfType === 'Container';
  const isDynamicText = pdfType === 'DynamicText';
  const isHyperlink = pdfType === 'Hyperlink';
  const isPageNumber = pdfType === 'PageNumber';
  // All text-like types get typography controls
  const isTextLike = isText || isDynamicText || isHyperlink || isPageNumber;

  return (
    <div className="text-xs text-gray-200 overflow-y-auto max-h-full">
      {/* Element header */}
      <div className="sticky top-0 z-10 bg-gray-900 px-3 py-2 border-b border-gray-700 flex items-center gap-2">
        <span className="text-sm">{typeIcon(pdfType)}</span>
        <span className="font-bold text-indigo-400 text-[11px]">{pdfType}</span>
        {isPageRoot && <span className="text-[9px] text-gray-500 ml-auto">🔒 locked</span>}
      </div>

      <div className="p-3 space-y-3">

        {/* ═══ PAGE ROOT ═══ */}
        {isPageRoot && (
          <>
            <Section label="Page Settings">
              <Row>
                <NumInput label="Pad Top" value={pf(style['padding-top'] || style['padding'])} unit="px" onChange={(v) => setStyle('padding-top', v + 'px')} />
                <NumInput label="Pad Bot" value={pf(style['padding-bottom'] || style['padding'])} unit="px" onChange={(v) => setStyle('padding-bottom', v + 'px')} />
              </Row>
              <Row>
                <NumInput label="Pad Left" value={pf(style['padding-left'] || style['padding'])} unit="px" onChange={(v) => setStyle('padding-left', v + 'px')} />
                <NumInput label="Pad Right" value={pf(style['padding-right'] || style['padding'])} unit="px" onChange={(v) => setStyle('padding-right', v + 'px')} />
              </Row>
            </Section>
            <Section label="Page Margins">
              <Row>
                <NumInput label="Top" value={pf(style['margin-top'])} unit="px" onChange={(v) => setStyle('margin-top', v + 'px')} />
                <NumInput label="Bottom" value={pf(style['margin-bottom'])} unit="px" onChange={(v) => setStyle('margin-bottom', v + 'px')} />
              </Row>
              <Row>
                <NumInput label="Left" value={pf(style['margin-left'])} unit="px" onChange={(v) => setStyle('margin-left', v + 'px')} />
                <NumInput label="Right" value={pf(style['margin-right'])} unit="px" onChange={(v) => setStyle('margin-right', v + 'px')} />
              </Row>
            </Section>
            <Section label="Page Background">
              <Row>
                <ColorInput label="Color" value={style['background-color'] || style['background'] || '#ffffff'} onChange={(v) => setStyle('background-color', v)} />
              </Row>
            </Section>
          </>
        )}

        {/* ═══ TEXT / DYNAMIC TEXT / HYPERLINK / PAGE NUMBER ═══ */}
        {isTextLike && (
          <>
            <Section label="Typography">
              <SelectInput label="Font" value={(style['font-family'] || 'Inter').replace(/['"]/g, '').split(',')[0].trim()} options={['Inter', 'Helvetica', 'Times New Roman', 'Courier', 'Georgia', 'Arial', 'Verdana']} onChange={(v) => setStyle('font-family', `'${v}', sans-serif`)} />
              <Row>
                <NumInput label="Size" value={pf(style['font-size']) || 14} unit="px" onChange={(v) => setStyle('font-size', v + 'px')} />
                <SelectInput label="Weight" value={style['font-weight'] || 'normal'} options={['normal', 'bold', '300', '500', '600', '700', '800', '900']} onChange={(v) => setStyle('font-weight', v)} />
              </Row>
              <Row>
                <SelectInput label="Style" value={style['font-style'] || 'normal'} options={['normal', 'italic']} onChange={(v) => setStyle('font-style', v)} />
                <SelectInput label="Align" value={style['text-align'] || 'left'} options={['left', 'center', 'right', 'justify']} onChange={(v) => setStyle('text-align', v)} />
              </Row>
              <Row>
                <NumInput label="Line H" value={pf(style['line-height']) || 1.5} unit="" step={0.1} onChange={(v) => setStyle('line-height', String(v))} />
                <NumInput label="Letter" value={pf(style['letter-spacing'])} unit="px" onChange={(v) => setStyle('letter-spacing', v + 'px')} />
              </Row>
              <Row>
                <SelectInput label="Transform" value={style['text-transform'] || 'none'} options={['none', 'uppercase', 'lowercase', 'capitalize']} onChange={(v) => setStyle('text-transform', v)} />
              </Row>
            </Section>
            <Section label="Colors">
              <Row>
                <ColorInput label="Text" value={style['color'] || '#1e293b'} onChange={(v) => setStyle('color', v)} />
                <ColorInput label="BG" value={style['background-color'] || 'transparent'} onChange={(v) => setStyle('background-color', v)} />
              </Row>
            </Section>
          </>
        )}

        {/* ═══ HYPERLINK SPECIAL ═══ */}
        {isHyperlink && (
          <Section label="Link">
            <Row>
              <TextInput label="URL" value={attrs['href'] || '#'} onChange={(v) => setAttr('href', v)} />
            </Row>
            <Row>
              <TextInput label="Text" value={comp.components().models[0]?.get('content') || ''} onChange={(v) => comp.components(v)} />
            </Row>
          </Section>
        )}

        {/* ═══ DYNAMIC TEXT SPECIAL ═══ */}
        {isDynamicText && (
          <Section label="Data Binding">
            <Row>
              <TextInput label="Field" value={attrs['data-binding'] || ''} onChange={(v) => { setAttr('data-binding', v); comp.components(`{{${v}}}`); }} />
            </Row>
          </Section>
        )}

        {/* ═══ IMAGE ═══ */}
        {isImage && (
          <Section label="Dimensions">
            <Row>
              <NumInput label="Width" value={pf(style['width']) || 200} unit="px" onChange={(v) => setStyle('width', v + 'px')} />
              <NumInput label="Height" value={pf(style['height']) || 150} unit="px" onChange={(v) => setStyle('height', v + 'px')} />
            </Row>
          </Section>
        )}

        {/* ═══ TABLE ═══ */}
        {isTable && (
          <>
            <Section label="Header Style">
              <Row>
                <ColorInput label="BG" value={attrs['data-header-bg'] || '#1e293b'} onChange={(v) => setAttr('data-header-bg', v)} />
                <ColorInput label="Text" value={attrs['data-header-color'] || '#ffffff'} onChange={(v) => setAttr('data-header-color', v)} />
              </Row>
            </Section>
            <Section label="Cell Style">
              <Row>
                <NumInput label="Pad" value={pf(attrs['data-cell-padding']) || 10} unit="px" onChange={(v) => setAttr('data-cell-padding', String(v))} />
                <NumInput label="Font" value={pf(attrs['data-font-size']) || 12} unit="px" onChange={(v) => setAttr('data-font-size', String(v))} />
              </Row>
              <Row>
                <ColorInput label="Border" value={attrs['data-border-color'] || '#e2e8f0'} onChange={(v) => setAttr('data-border-color', v)} />
              </Row>
            </Section>
            <Section label="Column Widths">
              <TableColumnWidths comp={state.comp} onChanged={() => refresh()} />
            </Section>
          </>
        )}

        {/* ═══ SHAPES ═══ */}
        {isShape && (
          <Section label="Shape">
            <Row>
              <NumInput label="W" value={pf(style['width'])} unit="px" onChange={(v) => setStyle('width', v + 'px')} />
              <NumInput label="H" value={pf(style['height'])} unit="px" onChange={(v) => setStyle('height', v + 'px')} />
            </Row>
            <Row>
              <ColorInput label="Fill" value={style['background-color'] || '#3b82f6'} onChange={(v) => setStyle('background-color', v)} />
            </Row>
            {pdfType === 'Rectangle' && (
              <Row>
                <NumInput label="Radius" value={pf(style['border-radius'])} unit="px" onChange={(v) => setStyle('border-radius', v + 'px')} />
              </Row>
            )}
          </Section>
        )}

        {isLine && (
          <Section label="Line">
            <Row>
              <NumInput label="Width" value={pf(style['width']) || 200} unit="px" onChange={(v) => setStyle('width', v + 'px')} />
              <NumInput label="Thick" value={pf(style['border-top-width']) || 2} unit="px" onChange={(v) => setStyle('border-top-width', v + 'px')} />
            </Row>
            <Row>
              <ColorInput label="Color" value={style['border-top-color'] || style['border-color'] || '#334155'} onChange={(v) => setStyle('border-top-color', v)} />
            </Row>
          </Section>
        )}

        {/* ═══ LAYOUT-SPECIFIC ═══ */}
        {isLayout && (
          <Section label="Container">
            <Row>
              <SelectInput label="Direction" value={style['flex-direction'] || 'column'} options={['column', 'row', 'column-reverse', 'row-reverse']} onChange={(v) => setStyle('flex-direction', v)} />
              <SelectInput label="Wrap" value={style['flex-wrap'] || 'nowrap'} options={['nowrap', 'wrap', 'wrap-reverse']} onChange={(v) => setStyle('flex-wrap', v)} />
            </Row>
            <Row>
              <NumInput label="Gap" value={pf(style['gap'])} unit="px" onChange={(v) => setStyle('gap', v + 'px')} />
              <SelectInput label="Align" value={style['align-items'] || 'stretch'} options={['stretch', 'flex-start', 'center', 'flex-end', 'baseline']} onChange={(v) => setStyle('align-items', v)} />
            </Row>
            <Row>
              <SelectInput label="Justify" value={style['justify-content'] || 'flex-start'} options={['flex-start', 'center', 'flex-end', 'space-between', 'space-around', 'space-evenly']} onChange={(v) => setStyle('justify-content', v)} />
            </Row>
          </Section>
        )}

        {/* ═══ COMMON PROPERTIES (for non-PageRoot) ═══ */}
        {!isPageRoot && (
          <>
            <Divider />

            <Section label="Layout">
              <Row>
                <SelectInput label="Display" value={style['display'] || 'block'} options={['block', 'inline-block', 'inline', 'flex', 'inline-flex', 'none']} onChange={(v) => setStyle('display', v)} />
              </Row>
            </Section>

            <Section label="Spacing">
              <Row>
                <NumInput label="Pad" value={pf(style['padding'])} unit="px" onChange={(v) => setStyle('padding', v + 'px')} />
                <NumInput label="Margin" value={pf(style['margin'])} unit="px" onChange={(v) => setStyle('margin', v + 'px')} />
              </Row>
            </Section>

            <Section label="Background">
              <Row>
                <ColorInput label="Color" value={style['background-color'] || 'transparent'} onChange={(v) => setStyle('background-color', v)} />
              </Row>
            </Section>

            <Section label="Border">
              <Row>
                <NumInput label="Width" value={pf(style['border-width'])} unit="px" onChange={(v) => { setStyle('border-width', v + 'px'); if (v > 0 && !style['border-style']) setStyle('border-style', 'solid'); }} />
                <ColorInput label="Color" value={style['border-color'] || '#d1d5db'} onChange={(v) => setStyle('border-color', v)} />
              </Row>
              <Row>
                <NumInput label="Radius" value={pf(style['border-radius'])} unit="px" onChange={(v) => setStyle('border-radius', v + 'px')} />
                <SelectInput label="Style" value={style['border-style'] || 'solid'} options={['none', 'solid', 'dashed', 'dotted', 'double']} onChange={(v) => setStyle('border-style', v)} />
              </Row>
            </Section>

            <Section label="Effects">
              <Row>
                <NumInput label="Opacity" value={Math.round(parseFloat(style['opacity'] || '1') * 100)} unit="%" onChange={(v) => setStyle('opacity', String(v / 100))} />
                <NumInput label="Rotate" value={extractRotation(style['transform'])} unit="°" onChange={(v) => setStyle('transform', v ? `rotate(${v}deg)` : '')} />
              </Row>
            </Section>

            <Section label="Size">
              <Row>
                <NumInput label="W" value={pf(style['width'])} unit="px" onChange={(v) => setStyle('width', v ? v + 'px' : '')} />
                <NumInput label="H" value={pf(style['height'])} unit="px" onChange={(v) => setStyle('height', v ? v + 'px' : '')} />
              </Row>
              <Row>
                <NumInput label="Min H" value={pf(style['min-height'])} unit="px" onChange={(v) => setStyle('min-height', v ? v + 'px' : '')} />
                <NumInput label="Max W" value={pf(style['max-width'])} unit="px" onChange={(v) => setStyle('max-width', v ? v + 'px' : '')} />
              </Row>
            </Section>
          </>
        )}
      </div>
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════════════════
//  Sub-components
// ═══════════════════════════════════════════════════════════════════════════

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <div className="text-[10px] uppercase tracking-wider text-gray-500 font-bold mb-1.5">{label}</div>
      <div className="space-y-1.5">{children}</div>
    </div>
  );
}

function Row({ children }: { children: React.ReactNode }) {
  return <div className="flex gap-2 items-center">{children}</div>;
}

function TableColumnWidths({ comp, onChanged }: { comp: any; onChanged: () => void }) {
  const el = comp?.view?.el as HTMLTableElement | null;
  if (!el) return <div className="text-gray-600 text-[10px]">No table DOM</div>;
  const headers = Array.from(el.querySelectorAll('thead th')) as HTMLElement[];
  if (headers.length === 0) return <div className="text-gray-600 text-[10px]">No headers</div>;

  const widths = headers.map(th => {
    const w = th.style.width;
    if (w && w.endsWith('px')) return parseInt(w);
    return th.getBoundingClientRect().width || 100;
  });

  const setColWidth = (colIdx: number, width: number) => {
    // Traverse GrapesJS model: Table -> thead -> tr -> th
    const thead = comp.components().models.find((c: any) => c.get('tagName') === 'thead');
    if (!thead) { console.warn('TableColumnWidths: No thead found'); return; }
    const tr = thead.components().models.find((c: any) => c.get('tagName') === 'tr');
    if (!tr) { console.warn('TableColumnWidths: No tr header found'); return; }
    const ths = tr.components().models;
    const th = ths[colIdx];
    
    if (th) {
      console.log(`Setting col ${colIdx} width to ${width}`);
      if (width > 0) th.addStyle({ width: `${width}px` });
      else {
        // Remove width by setting empty string or reverting to auto
        const style = { ...th.getStyle() };
        delete style.width;
        th.setStyle(style);
      }
      // Force refresh to update the inputs
      onChanged();
    } else {
      console.warn(`TableColumnWidths: No th found at index ${colIdx}`);
    }
  };

  return (
    <div className="space-y-1">
      {widths.map((w, i) => (
        <Row key={i}>
          <NumInput
            label={`Col ${i + 1}`}
            value={Math.round(w)}
            unit="px"
            onChange={(v) => setColWidth(i, v)}
          />
        </Row>
      ))}
      <div className="text-[9px] text-gray-600 mt-1">
        Set to 0 for auto width
      </div>
    </div>
  );
}

function Divider() {
  return <div className="border-t border-gray-700/50 my-1" />;
}

const stopKeys = (e: React.KeyboardEvent) => e.stopPropagation();

function NumInput({ label, value, unit, step, onChange }: {
  label: string; value: number; unit: string; step?: number; onChange: (v: number) => void;
}) {
  return (
    <div className="flex items-center gap-1 flex-1 min-w-0">
      <span className="text-gray-500 text-[10px] w-10 flex-shrink-0 truncate">{label}</span>
      <input
        type="number"
        step={step || 1}
        value={Math.round(value * 100) / 100 || ''}
        onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
        onKeyDown={stopKeys}
        onKeyUp={stopKeys}
        className="flex-1 min-w-0 bg-gray-800 border border-gray-700 rounded px-1.5 py-0.5 text-gray-200 text-[11px] focus:border-indigo-500 focus:outline-none w-12"
      />
      {unit && <span className="text-gray-600 text-[10px] w-4 flex-shrink-0">{unit}</span>}
    </div>
  );
}

function ColorInput({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div className="flex items-center gap-1 flex-1 min-w-0">
      <span className="text-gray-500 text-[10px] w-10 flex-shrink-0 truncate">{label}</span>
      <div className="flex items-center gap-1 flex-1 min-w-0 bg-gray-800 border border-gray-700 rounded px-1.5 py-0.5">
        <input
          type="color"
          value={safeHex(value)}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={stopKeys}
          onKeyUp={stopKeys}
          className="w-4 h-4 rounded border-none p-0 bg-transparent cursor-pointer"
        />
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={stopKeys}
          onKeyUp={stopKeys}
          className="flex-1 min-w-0 bg-transparent text-gray-200 text-[11px] focus:outline-none"
        />
      </div>
    </div>
  );
}

function SelectInput({ label, value, options, onChange }: {
  label: string; value: string; options: string[]; onChange: (v: string) => void;
}) {
  return (
    <div className="flex items-center gap-1 flex-1 min-w-0">
      <span className="text-gray-500 text-[10px] w-10 flex-shrink-0 truncate">{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={stopKeys}
        onKeyUp={stopKeys}
        className="flex-1 min-w-0 bg-gray-800 border border-gray-700 rounded px-1 py-0.5 text-gray-200 text-[11px] focus:border-indigo-500 focus:outline-none cursor-pointer"
      >
        {options.map(o => <option key={o} value={o}>{o}</option>)}
      </select>
    </div>
  );
}

function TextInput({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div className="flex items-center gap-1 flex-1 min-w-0">
      <span className="text-gray-500 text-[10px] w-10 flex-shrink-0 truncate">{label}</span>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={stopKeys}
        onKeyUp={stopKeys}
        className="flex-1 min-w-0 bg-gray-800 border border-gray-700 rounded px-1.5 py-0.5 text-gray-200 text-[11px] focus:border-indigo-500 focus:outline-none"
      />
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════════════════
//  Utils
// ═══════════════════════════════════════════════════════════════════════════

function pf(v: string | undefined | null): number {
  if (!v) return 0;
  return parseFloat(String(v).replace(/[^\d.-]/g, '')) || 0;
}

function extractRotation(t: string | undefined): number {
  if (!t) return 0;
  const m = t.match(/rotate\(([-\d.]+)deg\)/);
  return m ? parseFloat(m[1]) : 0;
}

function safeHex(color: string): string {
  if (!color || color === 'transparent' || color === 'inherit' || color === 'none') return '#000000';
  if (/^#[0-9a-f]{6}$/i.test(color)) return color;
  if (/^#[0-9a-f]{3}$/i.test(color)) return '#' + color[1] + color[1] + color[2] + color[2] + color[3] + color[3];
  const m = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
  if (m) return '#' + [m[1], m[2], m[3]].map(n => parseInt(n).toString(16).padStart(2, '0')).join('');
  return '#000000';
}

function typeIcon(t: string): string {
  const icons: Record<string, string> = {
    Text: '✏', Image: '🖼', Table: '☰', Rectangle: '■', Circle: '●', Line: '—',
    Column: '📐', Row: '↔', Container: '☐', Header: '▬', Footer: '▬',
    PageNumber: '#', DynamicText: '⚡', Hyperlink: '🔗', PageRoot: '📄',
  };
  return icons[t] || '🔹';
}
