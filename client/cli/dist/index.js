#!/usr/bin/env node
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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const commander_1 = require("commander");
const chalk_1 = __importDefault(require("chalk"));
const fs = __importStar(require("fs-extra"));
const path = __importStar(require("path"));
const archiver_1 = __importDefault(require("archiver"));
const client_1 = require("@pdfcore/client");
const program = new commander_1.Command();
program
    .name('pdfcore')
    .description('PDFCore CLI tool for rendering and managing templates')
    .version('1.0.0');
program.command('render')
    .description('Render a template to PDF')
    .argument('<template>', 'Path to template file (.pdfCoret, .zip, or .json)')
    .option('-d, --data <file>', 'Path to JSON data file')
    .option('-o, --output <file>', 'Output PDF file', 'output.pdf')
    .option('--stream', 'Read data from STDIN as NDJSON stream')
    .action(async (templatePath, options) => {
    try {
        const client = new client_1.PDFCoreClient();
        const outputPath = path.resolve(options.output);
        if (options.stream) {
            console.log(chalk_1.default.blue(`Rendering batch from STDIN to ${outputPath}...`));
            await client.renderBatch(templatePath, process.stdin, outputPath);
        }
        else {
            if (!options.data) {
                console.error(chalk_1.default.red('Error: --data <file> is required in non-stream mode.'));
                process.exit(1);
            }
            const dataPath = path.resolve(options.data);
            const data = await fs.readJson(dataPath);
            console.log(chalk_1.default.blue(`Rendering ${templatePath} to ${outputPath}...`));
            await client.render({
                template: templatePath,
                data,
                output: outputPath
            });
        }
        console.log(chalk_1.default.green('Success!'));
    }
    catch (error) {
        console.error(chalk_1.default.red('Error rendering PDF:'), error.message);
        process.exit(1);
    }
});
program.command('pack')
    .description('Pack a template directory into a .pdfCoret file')
    .argument('<directory>', 'Directory containing layout.json and assets')
    .option('-o, --output <file>', 'Output .pdfCoret file')
    .action(async (dir, options) => {
    const sourceDir = path.resolve(dir);
    const layoutPath = path.join(sourceDir, 'layout.json');
    if (!fs.existsSync(layoutPath)) {
        console.error(chalk_1.default.red(`Error: layout.json not found in ${sourceDir}`));
        process.exit(1);
    }
    const outputName = options.output || `${path.basename(sourceDir)}.pdfCoret`;
    const outputPath = path.resolve(outputName);
    const output = fs.createWriteStream(outputPath);
    const archive = (0, archiver_1.default)('zip', { zlib: { level: 9 } });
    output.on('close', () => {
        console.log(chalk_1.default.green(`Packed ${archive.pointer()} total bytes to ${outputName}`));
    });
    archive.on('error', (err) => {
        throw err;
    });
    archive.pipe(output);
    archive.directory(sourceDir, false);
    await archive.finalize();
});
program.command('validate')
    .description('Validate a template layout.json')
    .argument('<template>', 'Path to template directory or file')
    .action(async (templatePath) => {
    // Basic validation for now
    try {
        let layout;
        const stat = await fs.stat(templatePath);
        if (stat.isDirectory()) {
            layout = await fs.readJson(path.join(templatePath, 'layout.json'));
        }
        else if (templatePath.endsWith('.json')) {
            layout = await fs.readJson(templatePath);
        }
        else {
            console.log(chalk_1.default.yellow("Validation currently supports Directories or JSON files directly. for .pdfCoret, unpack first."));
            return;
        }
        if (!layout.root) {
            throw new Error("Missing 'root' node");
        }
        console.log(chalk_1.default.green("Layout appears valid (basic check passed)."));
    }
    catch (e) {
        console.error(chalk_1.default.red("Validation failed:"), e.message);
        process.exit(1);
    }
});
program.parse();
