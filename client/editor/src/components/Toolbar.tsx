/**
 * Toolbar — top bar with document actions, undo/redo, zoom, and export/import.
 */
import { useRef, useState, useEffect } from 'react';
import type grapesjs from 'grapesjs';
import { editorToTemplate, templateToEditor } from '../editor/converter';
import { exportTemplate, importTemplate, downloadBlob, collectAssetsFromEditor } from '../editor/templateIO';

type Editor = grapesjs.Editor;

interface ToolbarProps {
  editor: Editor | null;
}

export default function Toolbar({ editor }: ToolbarProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);

  useEffect(() => {
    if (!editor) return;
    const update = () => {
      setCanUndo(editor.UndoManager.hasUndo());
      setCanRedo(editor.UndoManager.hasRedo());
    };
    (editor as any).on('change:changesCount', update);
    editor.on('undo', update);
    editor.on('redo', update);
    return () => {
      (editor as any).off('change:changesCount', update);
      editor.off('undo', update);
      editor.off('redo', update);
    };
  }, [editor]);

  const handleUndo = () => editor?.UndoManager.undo();
  const handleRedo = () => editor?.UndoManager.redo();

  const handleExport = async () => {
    if (!editor) return;
    const template = editorToTemplate(editor);
    const assets = collectAssetsFromEditor(editor);
    const blob = await exportTemplate(template, assets);
    const name = template.manifest?.name || 'template';
    downloadBlob(blob, `${name.replace(/\s+/g, '_')}.pdfCoret`);
  };

  const handleImportClick = () => fileInputRef.current?.click();

  const handleImportFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!editor || !e.target.files?.length) return;
    const file = e.target.files[0];
    try {
      const { template, assets } = await importTemplate(file);
      const assetUrls = new Map<string, string>();
      for (const [name, blob] of assets) {
        assetUrls.set(name, URL.createObjectURL(blob));
      }
      templateToEditor(editor, template, assetUrls);
    } catch (err) {
      console.error('Import failed:', err);
      alert('Failed to import template:\n' + (err as Error).message);
    }
    e.target.value = '';
  };

  const handlePageSize = (device: string) => {
    if (!editor) return;
    editor.setDevice(device);
  };

  return (
    <div className="flex items-center gap-1 bg-gray-900 text-white px-3 py-1.5 text-xs border-b border-gray-800 select-none">
      {/* Brand */}
      <span className="font-bold text-sm mr-3 tracking-tight text-indigo-400">
        📄 PDFCore
      </span>

      <div className="w-px h-5 bg-gray-700 mx-1" />

      {/* Undo / Redo */}
      <ToolBtn icon="fa-undo" title="Undo (Ctrl+Z)" onClick={handleUndo} disabled={!canUndo} />
      <ToolBtn icon="fa-repeat" title="Redo (Ctrl+Y)" onClick={handleRedo} disabled={!canRedo} />

      <div className="w-px h-5 bg-gray-700 mx-1" />

      {/* Page Size */}
      <select
        onChange={(e) => handlePageSize(e.target.value)}
        className="bg-gray-800 border border-gray-700 rounded px-2 py-1 text-[11px] text-gray-300 cursor-pointer"
        defaultValue="A4 Portrait"
      >
        <option value="A4 Portrait">A4 Portrait</option>
        <option value="A4 Landscape">A4 Landscape</option>
        <option value="Letter">Letter</option>
      </select>

      <div className="flex-1" />

      {/* Export / Import */}
      <button
        onClick={handleExport}
        className="px-3 py-1 rounded bg-indigo-600 hover:bg-indigo-500 transition text-[11px] font-medium cursor-pointer"
      >
        📦 Export .pdfCoret
      </button>
      <button
        onClick={handleImportClick}
        className="px-3 py-1 rounded bg-emerald-600 hover:bg-emerald-500 transition text-[11px] font-medium cursor-pointer"
      >
        📂 Import
      </button>

      <input
        ref={fileInputRef}
        type="file"
        accept=".pdfCoret,.zip"
        className="hidden"
        onChange={handleImportFile}
      />
    </div>
  );
}

function ToolBtn({ icon, title, onClick, disabled }: { icon: string; title: string; onClick: () => void; disabled?: boolean }) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={`w-7 h-7 flex items-center justify-center rounded transition cursor-pointer
        ${disabled ? 'text-gray-600 cursor-not-allowed' : 'text-gray-300 hover:bg-gray-800 hover:text-white'}`}
    >
      <i className={`fa ${icon} text-xs`} />
    </button>
  );
}
