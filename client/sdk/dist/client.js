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
        // TODO: Allow customizing page size via options
        const doc = new index_1.Document();
        const page = new index_1.Page(595.28, 841.89); // A4
        // doc.addPage(page); // Moved to end
        // 3. Register Assets
        doc.registerTemplateAssets(template);
        // 4. Render Layout
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
        // Transform template to layout node with data binding
        const data = options.data || {};
        const root = template.toLayout(JSON.stringify(data));
        // Render to page
        page.renderLayout(root, 0, 841.89, 595.28, font, fontIndex);
        // Add page to document AFTER rendering (since it copies content)
        doc.addPage(page);
        // 5. Write Output
        doc.writeTo(options.output);
    }
}
exports.PDFCoreClient = PDFCoreClient;
