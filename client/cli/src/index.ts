#!/usr/bin/env node
import { Command } from 'commander';
import chalk from 'chalk';
import * as fs from 'fs-extra';
import * as path from 'path';
import archiver from 'archiver';
import { PDFCoreClient } from '@pdfcore/client';

const program = new Command();

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
      const client = new PDFCoreClient();
      const outputPath = path.resolve(options.output);

      if (options.stream) {
        console.log(chalk.blue(`Rendering batch from STDIN to ${outputPath}...`));
        await client.renderBatch(templatePath, process.stdin, outputPath);
      } else {
        if (!options.data) {
          console.error(chalk.red('Error: --data <file> is required in non-stream mode.'));
          process.exit(1);
        }
        const dataPath = path.resolve(options.data);
        const data = await fs.readJson(dataPath);
        
        console.log(chalk.blue(`Rendering ${templatePath} to ${outputPath}...`));
        await client.render({
          template: templatePath,
          data,
          output: outputPath
        });
      }
      console.log(chalk.green('Success!'));
    } catch (error: any) {
      console.error(chalk.red('Error rendering PDF:'), error.message);
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
      console.error(chalk.red(`Error: layout.json not found in ${sourceDir}`));
      process.exit(1);
    }

    const outputName = options.output || `${path.basename(sourceDir)}.pdfCoret`;
    const outputPath = path.resolve(outputName);
    const output = fs.createWriteStream(outputPath);
    const archive = archiver('zip', { zlib: { level: 9 } });

    output.on('close', () => {
      console.log(chalk.green(`Packed ${archive.pointer()} total bytes to ${outputName}`));
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
        let layout: any;
        const stat = await fs.stat(templatePath);
        if (stat.isDirectory()) {
            layout = await fs.readJson(path.join(templatePath, 'layout.json'));
        } else if (templatePath.endsWith('.json')) {
            layout = await fs.readJson(templatePath);
        } else {
             console.log(chalk.yellow("Validation currently supports Directories or JSON files directly. for .pdfCoret, unpack first."));
             return;
        }

        if (!layout.root) {
             throw new Error("Missing 'root' node");
        }
        console.log(chalk.green("Layout appears valid (basic check passed)."));
    } catch (e: any) {
        console.error(chalk.red("Validation failed:"), e.message);
        process.exit(1);
    }
  });

program.parse();
