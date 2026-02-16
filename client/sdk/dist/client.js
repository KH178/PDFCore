"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.PDFCoreClient = void 0;
const index_1 = require("../../../index"); // Import from local native binding for now
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
class PDFCoreClient {
    constructor() { }
    async render(options) {
        let template;
        // 1. Load Template
        if (typeof options.template === 'string') {
            if (options.template.endsWith('.pdfCoret') || options.template.endsWith('.zip')) {
                // Load from zip package
                template = index_1.Template.fromZip(options.template);
            }
            else if (options.template.endsWith('.json')) {
                // Load from JSON file
                const jsonContent = fs.readFileSync(options.template, 'utf-8');
                template = index_1.Template.fromJson(jsonContent);
            }
            else {
                throw new Error("Unsupported template format. Use .pdfCoret, .zip, or .json");
            }
        }
        else {
            template = options.template;
        }
        // 2. Prepare Document
        const doc = new index_1.Document();
        // 3. Register Assets
        doc.registerTemplateAssets(template);
        // 4. Setup Font
        // Use default font for now, TODO: Load fonts from package
        // Fallback to system font if local asset not found
        const fontPath = process.platform === 'win32'
            ? 'C:/Windows/Fonts/arial.ttf'
            : '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf';
        // Check if example asset exists, otherwise use system font
        const localFont = path.join(__dirname, '../../../examples/assets/Roboto-Regular.ttf');
        const finalFontPath = fs.existsSync(localFont) ? localFont : fontPath;
        const font = index_1.Font.fromFile(finalFontPath, "Arial");
        const fontIndex = doc.addFont(font);
        // 5. Transform template to layout node with data binding
        const data = options.data || {};
        const root = template.toLayout(JSON.stringify(data));
        // 6. Get Page Settings
        const settings = template.getSettings();
        let width = 595.28; // A4 Default
        let height = 841.89;
        if (settings.size === 'Letter') {
            width = 612.0;
            height = 792.0;
        }
        if (settings.orientation === 'Landscape') {
            const temp = width;
            width = height;
            height = temp;
        }
        const margins = settings.margins || { top: 50, bottom: 50, left: 50, right: 50 };
        // 7. Render Flow (Automatic Pagination)
        doc.renderFlow(root, width, height, font, fontIndex, undefined, // Header 
        undefined, // Footer
        {
            marginTop: margins.top,
            marginBottom: margins.bottom,
            marginLeft: margins.left,
            marginRight: margins.right
        });
        // 8. Write Output
        doc.writeTo(options.output);
    }
    async renderBatch(templatePath, dataStream, outputPath) {
        // 1. Load Template
        let template;
        if (templatePath.endsWith('.pdfCoret') || templatePath.endsWith('.zip')) {
            template = index_1.Template.fromZip(templatePath);
        }
        else if (templatePath.endsWith('.json')) {
            const jsonContent = fs.readFileSync(templatePath, 'utf-8');
            template = index_1.Template.fromJson(jsonContent);
        }
        else {
            throw new Error("Unsupported template format. Use .pdfCoret, .zip, or .json");
        }
        // 2. Prepare Streaming Document
        const doc = index_1.Document.streaming(outputPath);
        // 3. Register Assets ONCE
        // Note: For streaming, we should register assets before adding pages if possible, 
        // to ensure they are available. The current Rust implementation handles this.
        doc.registerTemplateAssets(template);
        // 4. Setup Font
        const fontPath = process.platform === 'win32'
            ? 'C:/Windows/Fonts/arial.ttf'
            : '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf';
        const localFont = path.join(__dirname, '../../../examples/assets/Roboto-Regular.ttf');
        const finalFontPath = fs.existsSync(localFont) ? localFont : fontPath;
        const font = index_1.Font.fromFile(finalFontPath, "Arial");
        const fontIndex = doc.addFont(font);
        // 5. Process Stream
        const rl = require('readline').createInterface({
            input: dataStream,
            crlfDelay: Infinity
        });
        for await (const line of rl) {
            if (!line.trim())
                continue;
            try {
                const data = JSON.parse(line);
                // create a page for this record
                const page = new index_1.Page(595.28, 841.89); // A4
                const root = template.toLayout(JSON.stringify(data));
                page.renderLayout(root, 0, 841.89, 595.28, font, fontIndex);
                // Write page immediately to stream
                doc.addPage(page);
            }
            catch (e) {
                console.error("Failed to process line:", line, e);
                // Continue? Or throw? For batch, maybe log and continue is better.
            }
        }
        // 6. Finalize Output
        doc.finalize();
    }
}
exports.PDFCoreClient = PDFCoreClient;
