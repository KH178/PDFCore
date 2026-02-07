const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const { Document, Template, Font } = require('../../index');

const exampleDir = __dirname;
const pkgDir = path.join(exampleDir, 'pkg');
const zipPath = path.join(exampleDir, 'invoice.pdfCoret');
const outputPath = path.join(exampleDir, 'invoice.pdf');

console.log('Generating Invoice Example...');

// Setup pkg directory
if (fs.existsSync(pkgDir)) fs.rmSync(pkgDir, { recursive: true, force: true });
fs.mkdirSync(pkgDir);

// Copy assets
fs.copyFileSync(path.join(exampleDir, 'layout.json'), path.join(pkgDir, 'layout.json'));
// Use simple 1x1 pixel PNG to avoid zip crash
fs.writeFileSync(path.join(pkgDir, 'logo.png'), Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==', 'base64'));

// Package
try {
    const parentTempZip = path.join(exampleDir, 'temp.zip');
    if (fs.existsSync(parentTempZip)) fs.unlinkSync(parentTempZip);
    
    // Use Compress-Archive like test_package_gen.js
    // Note: Compress-Archive requires the destination to end in .zip
    const cmd = `powershell -Command "Compress-Archive -Path '${pkgDir}/*' -DestinationPath '${parentTempZip}' -Force"`;
    console.log("Running:", cmd);
    execSync(cmd, { stdio: 'inherit' });
    
    if (fs.existsSync(zipPath)) fs.unlinkSync(zipPath);
    fs.renameSync(parentTempZip, zipPath);
    console.log(`Package created: ${zipPath}`);
} catch (e) {
    console.error("Packaging Failed!", e);
    process.exit(1);
}

// Render
try {
    console.log("Loading template...");
    const tmpl = Template.fromZip(zipPath);
    
    console.log("Loading data...");
    const data = fs.readFileSync(path.join(exampleDir, 'data.json'), 'utf-8');
    
    const doc = new Document();
    
    // Font
    const fontPath = 'C:/Windows/Fonts/arial.ttf';
    if (!fs.existsSync(fontPath)) {
        throw new Error("Font not found");
    }
    const font = Font.fromFile(fontPath, 'Arial');
    const fontIdx = doc.addFont(font);
    
    console.log("Registering assets...");
    doc.registerTemplateAssets(tmpl);
    
    console.log("Rendering layout...");
    const layout = tmpl.render(data);
    
    console.log("Drawing...");
    doc.renderFlow(layout, 595, 842, font, fontIdx);
    
    doc.writeTo(outputPath);
    console.log(`✓ Invoice generated: ${outputPath}`);
    
} catch (e) {
    console.error("Render Failed:", e.message);
    process.exit(1);
}
