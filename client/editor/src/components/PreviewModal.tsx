import { useEffect, useState } from 'react';
// Use wasm_v8 to bust the browser cache!
import init, { WasmTemplate } from '../wasm_v8/ai_pdf_writer';

interface PreviewModalProps {
  template: any;
  assets: Map<string, Blob>;
  onClose: () => void;
}

export default function PreviewModal({ template, assets, onClose }: PreviewModalProps) {
  const [pdfUrl, setPdfUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let active = true;

    async function render() {
      try {
        setLoading(true);
        // Initialize WASM
        await init();

        // Create Template
        const json = JSON.stringify(template, null, 2);
        // DEBUG: print full template to console for inspection
        console.log('[PDFCore] Template JSON:', json);
        // Highlight Image nodes specifically
        const findImages = (node: any): void => {
          if (!node) return;
          if (node.type === 'Image') console.log('[PDFCore] Image node:', JSON.stringify(node));
          if (node.child) findImages(node.child);
          if (node.children) node.children.forEach(findImages);
        };
        findImages(template.root);
        const wasmTemplate = WasmTemplate.from_json(json);

        // Add Assets
        for (const [name, blob] of assets) {
          const buffer = await blob.arrayBuffer();
          const bytes = new Uint8Array(buffer);
          wasmTemplate.add_asset(name, bytes);
        }

        // Render PDF (empty data for now)
        // TODO: Add data binding support later
        const data = "{}";
        const pdfBytes = wasmTemplate.render_to_pdf(data);
        
        if (!active) return;
        
        // Create Blob URL
        const pdfBlob = new Blob([new Uint8Array(pdfBytes)], { type: 'application/pdf' });
        const url = URL.createObjectURL(pdfBlob);
        setPdfUrl(url);
        setLoading(false);
      } catch (err) {
        if (!active) return;
        console.error("WASM Render Error:", err);
        setError((err as Error).message || "Unknown error during rendering");
        setLoading(false);
      }
    }

    render();

    return () => {
      active = false;
      if (pdfUrl) URL.revokeObjectURL(pdfUrl);
    };
  }, [template, assets]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4">
      <div className="bg-gray-900 rounded-lg shadow-2xl w-full h-full max-w-6xl max-h-[90vh] flex flex-col border border-gray-700">
        
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-gray-800">
          <h2 className="text-sm font-bold text-white flex items-center gap-2">
            <i className="fa fa-eye text-indigo-400" />
            PDF Preview (WASM)
          </h2>
          <button 
            onClick={onClose}
            className="text-gray-400 hover:text-white transition"
          >
            <i className="fa fa-times text-lg" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 bg-gray-800 relative overflow-hidden">
          {loading && (
            <div className="absolute inset-0 flex items-center justify-center flex-col gap-3 text-gray-400">
              <i className="fa fa-circle-notch fa-spin text-3xl text-indigo-500" />
              <span>Rendering PDF in browser...</span>
            </div>
          )}

          {error && (
            <div className="absolute inset-0 flex items-center justify-center bg-red-900/20">
              <div className="bg-red-950 border border-red-800 text-red-200 p-6 rounded max-w-md text-center">
                <i className="fa fa-exclamation-triangle text-2xl mb-2 text-red-500" />
                <h3 className="font-bold mb-2">Rendering Failed</h3>
                <p className="text-sm opacity-80">{error}</p>
                <p className="text-xs mt-4 text-gray-500">
                  Ensure wasm-pack is installed and built successfully.
                </p>
              </div>
            </div>
          )}

          {!loading && !error && pdfUrl && (
            <iframe 
              src={pdfUrl} 
              className="w-full h-full border-none" 
              title="PDF Preview"
            />
          )}
        </div>
      </div>
    </div>
  );
}
