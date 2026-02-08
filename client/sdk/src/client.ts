
import { Document, Font, Page, Template } from '../../../index'; // Import from local native binding for now
import * as fs from 'fs';
import * as path from 'path';

export interface RenderOptions {
    template: string | Template;
    data?: any;
    output: string;
}

export class PDFCoreClient {
    constructor() {}

    async render(options: RenderOptions): Promise<void> {
        let template: Template;

        // 1. Load Template
        if (typeof options.template === 'string') {
            if (options.template.endsWith('.pdfCoret') || options.template.endsWith('.zip')) {
                // Load from zip package
                template = Template.fromZip(options.template);
            } else if (options.template.endsWith('.json')) {
                 // Load from JSON file
                 const jsonContent = fs.readFileSync(options.template, 'utf-8');
                 template = Template.fromJson(jsonContent);
            } else {
                throw new Error("Unsupported template format. Use .pdfCoret, .zip, or .json");
            }
        } else {
            template = options.template;
        }

        // 2. Prepare Document
        // TODO: Allow customizing page size via options
        const doc = new Document(); 
        const page = new Page(595.28, 841.89); // A4
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

        const font = Font.fromFile(finalFontPath, "Arial");
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
