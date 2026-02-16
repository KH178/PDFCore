/**
 * PDFCore Visual Editor — Main Shell
 *
 * Key design decisions:
 * - Initial content uses GrapesJS component JSON (not HTML strings) so
 *   type defaults like editable:true are reliably applied.
 * - Page dimensions are explicit pixels matching the selected page size.
 * - Text editing: native iframe dblclick listener + GrapesJS editable:true.
 * - Canvas iframe auto-refreshes after load for proper alignment.
 */
import { useEffect, useRef, useState } from 'react';
import grapesjs from 'grapesjs';
type EditorType = grapesjs.Editor;
import 'grapesjs/dist/css/grapes.min.css';
import { registerPDFBlocks } from '../editor/pdfBlocks';
import { registerComponentTypes, registerEditorCommands } from '../editor/componentTypes';
import PropertyInspector from './PropertyInspector';
import Toolbar from './Toolbar';

// Page sizes in points (1pt = 1px at 72dpi, the PDF standard)
const PAGE_SIZES: Record<string, { w: number; h: number }> = {
  'A4 Portrait':    { w: 595, h: 842 },
  'A4 Landscape':   { w: 842, h: 595 },
  'Letter':         { w: 612, h: 792 },
};

export default function PDFEditor() {
  const canvasRef = useRef<HTMLDivElement>(null);
  const [editor, setEditor] = useState<EditorType | null>(null);
  const [activeTab, setActiveTab] = useState<'blocks' | 'properties'>('blocks');

  useEffect(() => {
    if (!canvasRef.current) return;

    const defaultDevice = 'A4 Portrait';
    const defaultSize = PAGE_SIZES[defaultDevice];

    const gjsEditor = grapesjs.init({
      container: canvasRef.current,
      height: '100%',
      width: '100%',
      fromElement: false,
      storageManager: false,

      canvas: {
        styles: [
          'https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap',
        ],
      },

      // Pixel-based page widths
      deviceManager: {
        devices: Object.entries(PAGE_SIZES).map(([name, sz]) => ({
          name,
          width: `${sz.w}px`,
        })),
      },

      panels: { defaults: [] },

      layerManager: { appendTo: '.panel-layers' },
      blockManager: { appendTo: '.panel-blocks', blocks: [] },
      assetManager: { upload: false, dropzone: true },
    });

    // ═══ Register types FIRST ═══
    registerComponentTypes(gjsEditor);
    registerEditorCommands(gjsEditor);
    registerPDFBlocks(gjsEditor);

    // ═══ Set initial content as component definitions (NOT HTML strings) ═══
    // This ensures editable: true is applied from the type defaults.
    const wrapper = gjsEditor.DomComponents.getWrapper();
    wrapper?.components().reset();
    wrapper?.append({
      type: 'pdf-page-root',
      style: {
        width: '100%',
        'min-height': `${defaultSize.h}px`,
        background: 'white',
        padding: '40px',
        'font-family': "'Inter', sans-serif",
        'box-sizing': 'border-box',
      },
      components: [
        {
          type: 'pdf-text-comp',
          content: 'Untitled Document',
          attributes: { 'data-pdf-type': 'Text', 'data-pdf-size': '28' },
          style: {
            'font-size': '28px',
            'font-weight': 'bold',
            color: '#0f172a',
            'font-family': 'Inter, sans-serif',
            padding: '4px',
            'margin-bottom': '8px',
          },
        },
        {
          type: 'pdf-text-comp',
          content: 'Drag blocks from the right panel. Double-click text to edit.',
          attributes: { 'data-pdf-type': 'Text' },
          style: {
            'font-size': '13px',
            color: '#64748b',
            'font-family': 'Inter, sans-serif',
            padding: '4px',
          },
        },
      ],
    } as any);

    // ═══ Keyboard shortcuts ═══
    const keyHandler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'z') { e.preventDefault(); gjsEditor.UndoManager.undo(); }
      if (e.ctrlKey && e.key === 'y') { e.preventDefault(); gjsEditor.UndoManager.redo(); }
      if (e.key === 'Delete' || e.key === 'Backspace') {
        const sel = gjsEditor.getSelected();
        if (!sel) return;
        if ((sel.getAttributes() as any)['data-pdf-type'] === 'PageRoot') return;
        // Don't delete while editing text
        const iframe = gjsEditor.Canvas.getFrameEl();
        const active = iframe?.contentDocument?.activeElement;
        if (active?.getAttribute('contenteditable') === 'true') return;
        e.preventDefault();
        sel.remove();
      }
    };
    document.addEventListener('keydown', keyHandler);

    // ═══ Auto-switch to Properties ═══
    gjsEditor.on('component:selected', () => setActiveTab('properties'));

    // ═══ Update PageRoot when device/page-size changes ═══
    (gjsEditor as any).on('device:select', () => {
      const deviceName = (gjsEditor as any).getDevice();
      const size = PAGE_SIZES[deviceName];
      if (!size) return;
      // PageRoot width is 100% — it auto-fills the device frame.
      // Only update min-height for the new page dimensions.
      const root = wrapper?.components().models.find(
        (m: any) => m.getAttributes()['data-pdf-type'] === 'PageRoot'
      );
      if (root) {
        root.setStyle({
          ...root.getStyle(),
          'min-height': `${size.h}px`,
        });
      }
    });

    // ═══ Canvas styling + TEXT EDITING via native iframe dblclick ═══
    gjsEditor.on('load', () => {
      const frame = gjsEditor.Canvas.getFrameEl();
      if (!frame?.contentDocument) return;

      // Inject iframe styles
      const style = frame.contentDocument.createElement('style');
      style.textContent = `
        * { box-sizing: border-box; }
        body {
          margin: 0 !important;
          padding: 0 !important;
          background: white !important;
          overflow-x: hidden;
        }
        [data-pdf-type="PageRoot"] {
          /* PageRoot IS the page — width:100% fills the device frame exactly */
        }
        [data-pdf-type="Text"],
        [data-pdf-type="PageNumber"],
        [data-pdf-type="Hyperlink"] {
          cursor: text;
          min-height: 1em;
        }
        [contenteditable="true"] {
          outline: 2px solid #6366f1 !important;
          outline-offset: 2px;
        }
        [data-pdf-type="Image"] { cursor: move; transition: box-shadow 0.15s; }
        [data-pdf-type="Image"]:hover { box-shadow: 0 0 0 2px #6366f1; }
        [data-pdf-type="Image"] img { pointer-events: none; }
        [data-pdf-type="Rectangle"], [data-pdf-type="Circle"] { cursor: move; }
        [data-pdf-type="Column"], [data-pdf-type="Row"],
        [data-pdf-type="Container"], [data-pdf-type="Header"],
        [data-pdf-type="Footer"] { min-height: 30px; }
        [data-pdf-type="Column"]:hover, [data-pdf-type="Row"]:hover,
        [data-pdf-type="Container"]:hover {
          outline: 1px dashed rgba(99,102,241,0.4);
          outline-offset: -1px;
        }
        table td, table th { cursor: text; }
      `;
      frame.contentDocument.head.appendChild(style);

      // ══ NATIVE DBLCLICK on iframe for text editing ══
      frame.contentDocument.addEventListener('dblclick', (e: Event) => {
        const target = e.target as HTMLElement;
        if (!target) return;

        // Direct table cell editing — td/th get contentEditable directly
        const tag = target.tagName?.toLowerCase();
        if (tag === 'td' || tag === 'th') {
          e.preventDefault();
          e.stopPropagation();
          target.contentEditable = 'true';
          target.focus();
          const sel = frame.contentDocument!.getSelection();
          if (sel) {
            const range = frame.contentDocument!.createRange();
            range.selectNodeContents(target);
            sel.removeAllRanges();
            sel.addRange(range);
          }
          return;
        }

        // Walk up to find the nearest data-pdf-type element
        let el: HTMLElement | null = target;
        while (el && !el.getAttribute('data-pdf-type')) {
          el = el.parentElement;
        }
        if (!el) return;
        const pdfType = el.getAttribute('data-pdf-type');
        if (pdfType === 'Text' || pdfType === 'PageNumber' || pdfType === 'Hyperlink' || pdfType === 'DynamicText') {
          e.preventDefault();
          e.stopPropagation();
          el.contentEditable = 'true';
          el.focus();
          const sel = frame.contentDocument!.getSelection();
          if (sel) {
            const range = frame.contentDocument!.createRange();
            range.selectNodeContents(el);
            sel.removeAllRanges();
            sel.addRange(range);
          }
        }
      });

      // Click outside editable → save and exit edit mode
      frame.contentDocument.addEventListener('click', (e: Event) => {
        const target = e.target as HTMLElement;
        const editables = frame.contentDocument!.querySelectorAll('[contenteditable="true"]');
        editables.forEach((editable: Element) => {
          if (editable !== target && !editable.contains(target)) {
            (editable as HTMLElement).contentEditable = 'false';
          }
        });
      });

      gjsEditor.refresh();
    });

    (window as any).editor = gjsEditor;
    setEditor(gjsEditor);

    return () => {
      document.removeEventListener('keydown', keyHandler);
      gjsEditor.destroy();
    };
  }, []);

  return (
    <div className="h-screen w-full flex flex-col bg-gray-950 text-white overflow-hidden">
      <Toolbar editor={editor} />
      <div className="flex-1 flex overflow-hidden">
        {/* Layers */}
        <div className="w-44 bg-gray-900 flex-shrink-0 border-r border-gray-800 flex flex-col">
          <div className="px-3 py-2 text-[10px] font-bold uppercase tracking-widest text-gray-500 border-b border-gray-800">Layers</div>
          <div className="panel-layers flex-1 overflow-y-auto text-xs" />
        </div>

        {/* Canvas */}
        <div className="flex-1 overflow-hidden relative">
          <div ref={canvasRef} className="h-full w-full" />
        </div>

        {/* Blocks + Properties */}
        <div className="w-64 bg-gray-900 flex-shrink-0 border-l border-gray-800 flex flex-col">
          <div className="flex border-b border-gray-800">
            <button onClick={() => setActiveTab('blocks')}
              className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-widest transition cursor-pointer ${activeTab === 'blocks' ? 'text-indigo-400 border-b-2 border-indigo-400 bg-gray-800/50' : 'text-gray-500 hover:text-gray-300'}`}>
              Blocks
            </button>
            <button onClick={() => setActiveTab('properties')}
              className={`flex-1 py-2 text-[10px] font-bold uppercase tracking-widest transition cursor-pointer ${activeTab === 'properties' ? 'text-indigo-400 border-b-2 border-indigo-400 bg-gray-800/50' : 'text-gray-500 hover:text-gray-300'}`}>
              Properties
            </button>
          </div>
          <div className="flex-1 overflow-y-auto">
            <div style={{ display: activeTab === 'blocks' ? 'block' : 'none' }}>
              <div className="panel-blocks" />
            </div>
            <div style={{ display: activeTab === 'properties' ? 'block' : 'none' }}>
              <PropertyInspector editor={editor} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
