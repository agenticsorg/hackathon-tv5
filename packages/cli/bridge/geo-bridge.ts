#!/usr/bin/env node
/**
 * GEO Bridge - Node.js bridge between Rust CLI and @arw/geo TypeScript package
 * This allows the Rust CLI to leverage the full @arw/geo functionality
 */

import { GEOOptimizer } from '@arw/geo';
import { promises as fs } from 'fs';
import * as path from 'path';
import { Command } from 'commander';

const program = new Command();

interface AnalyzeOptions {
  domain?: string;
  useLlm?: boolean;
  format?: string;
  output?: string;
}

interface ReportOptions {
  output?: string;
  format?: string;
  detailed?: boolean;
}

program
  .name('geo-bridge')
  .description('Bridge between Rust CLI and @arw/geo package');

// Analyze command
program
  .command('analyze')
  .argument('<path>', 'Path to content file or directory')
  .option('--domain <domain>', 'Domain for optimization')
  .option('--use-llm', 'Use LLM for enhanced analysis')
  .option('--format <format>', 'Output format', 'text')
  .option('--output <output>', 'Output file path')
  .action(async (pathArg: string, options: AnalyzeOptions) => {
    try {
      await analyzeContent(pathArg, options);
    } catch (error) {
      console.error('Error:', error);
      process.exit(1);
    }
  });

// Report command
program
  .command('report')
  .option('--output <output>', 'Output directory', './output/geo-reports')
  .option('--format <format>', 'Report format', 'json')
  .option('--detailed', 'Include detailed analysis')
  .action(async (options: ReportOptions) => {
    try {
      await generateReport(options);
    } catch (error) {
      console.error('Error:', error);
      process.exit(1);
    }
  });

// Validate command
program
  .command('validate')
  .argument('<path>', 'Path to validate')
  .action(async (pathArg: string) => {
    try {
      await validateGeo(pathArg);
    } catch (error) {
      console.error('Error:', error);
      process.exit(1);
    }
  });

async function analyzeContent(contentPath: string, options: AnalyzeOptions) {
  // Read content
  const content = await fs.readFile(contentPath, 'utf-8');

  // Initialize GEO optimizer
  const config: any = {
    profile: 'ARW-2.2',
  };

  if (options.domain) {
    config.domain = options.domain;
  }

  if (options.useLlm) {
    config.llm = {
      provider: process.env.ANTHROPIC_API_KEY ? 'anthropic' : 'openai',
      apiKey: process.env.ANTHROPIC_API_KEY || process.env.OPENAI_API_KEY,
      model: process.env.ANTHROPIC_API_KEY
        ? 'claude-3-5-sonnet-20241022'
        : 'gpt-4-turbo-preview',
    };
  }

  const optimizer = new GEOOptimizer(config);

  // Analyze content
  const result = await optimizer.analyze(content, {
    extractCitations: true,
    extractStatistics: true,
    extractQuotations: true,
    calculateQuality: true,
    extractEntities: true,
    useLLM: options.useLlm,
  });

  // Format output
  let output: string;

  if (options.format === 'json') {
    output = JSON.stringify(result, null, 2);
  } else if (options.format === 'html') {
    output = formatAsHTML(result, contentPath);
  } else {
    output = formatAsText(result, contentPath);
  }

  // Write or print output
  if (options.output) {
    await fs.writeFile(options.output, output, 'utf-8');
    console.log(`✓ Analysis saved to: ${options.output}`);
  } else {
    console.log(output);
  }
}

async function generateReport(options: ReportOptions) {
  const outputDir = options.output || './output/geo-reports';

  // Create output directory
  await fs.mkdir(outputDir, { recursive: true });

  // Placeholder: In a full implementation, this would scan all .llm.md files
  // and generate a comprehensive report
  const report = {
    timestamp: new Date().toISOString(),
    summary: {
      totalPages: 0,
      avgCitations: 0,
      avgQuality: 0,
    },
    pages: [],
  };

  const outputPath = path.join(outputDir, `report.${options.format}`);

  if (options.format === 'json') {
    await fs.writeFile(outputPath, JSON.stringify(report, null, 2));
  } else if (options.format === 'html') {
    await fs.writeFile(outputPath, generateHTMLReport(report));
  } else {
    await fs.writeFile(outputPath, generateMarkdownReport(report));
  }

  console.log(`✓ Report saved to: ${outputPath}`);
}

async function validateGeo(contentPath: string) {
  const content = await fs.readFile(contentPath, 'utf-8');

  // Basic validation - check for GEO metadata
  const hasMetadata = content.includes('<!--GEO');

  if (hasMetadata) {
    console.log('✓ GEO metadata found');
  } else {
    console.log('⚠️  No GEO metadata found');
  }

  console.log('\nValidation complete');
}

function formatAsText(result: any, filePath: string): string {
  const lines: string[] = [];

  lines.push(`🔍 Analyzing: ${filePath}`);
  lines.push('');
  lines.push('GEO Analysis Results');
  lines.push('━'.repeat(50));
  lines.push(`Profile: ${result.profile}`);
  lines.push('');
  lines.push('📊 Content Metrics:');
  lines.push(`  ├─ Citations: ${result.citations?.length || 0}`);
  lines.push(`  ├─ Statistics: ${result.statistics?.length || 0}`);
  lines.push(`  ├─ Quotations: ${result.quotations?.length || 0}`);
  lines.push(`  ├─ Quality Score: ${result.quality?.overall?.toFixed(2) || 'N/A'}`);
  lines.push(`  └─ Entities: ${result.entities?.length || 0}`);
  lines.push('');

  if (result.citations?.length > 0) {
    lines.push('📖 Top Citations:');
    result.citations.slice(0, 3).forEach((c: any, i: number) => {
      lines.push(`  ${i + 1}. ${c.url}`);
    });
    lines.push('');
  }

  lines.push('✓ Analysis complete!');

  return lines.join('\n');
}

function formatAsHTML(result: any, filePath: string): string {
  return `
<!DOCTYPE html>
<html>
<head>
  <title>GEO Analysis - ${path.basename(filePath)}</title>
  <style>
    body { font-family: system-ui; max-width: 800px; margin: 40px auto; padding: 0 20px; }
    h1 { color: #2563eb; }
    .metric { padding: 10px; margin: 10px 0; background: #f1f5f9; border-radius: 8px; }
    .score { font-size: 2em; font-weight: bold; color: #16a34a; }
  </style>
</head>
<body>
  <h1>GEO Analysis Results</h1>
  <p>File: ${filePath}</p>
  <p>Profile: ${result.profile}</p>

  <div class="metric">
    <h2>Quality Score</h2>
    <div class="score">${result.quality?.overall?.toFixed(2) || 'N/A'}</div>
  </div>

  <div class="metric">
    <h2>Citations</h2>
    <p>${result.citations?.length || 0} found</p>
  </div>

  <div class="metric">
    <h2>Statistics</h2>
    <p>${result.statistics?.length || 0} found</p>
  </div>
</body>
</html>
`;
}

function generateHTMLReport(report: any): string {
  return `
<!DOCTYPE html>
<html>
<head>
  <title>GEO Report</title>
  <style>
    body { font-family: system-ui; max-width: 1200px; margin: 40px auto; padding: 0 20px; }
  </style>
</head>
<body>
  <h1>GEO Optimization Report</h1>
  <p>Generated: ${report.timestamp}</p>
  <pre>${JSON.stringify(report, null, 2)}</pre>
</body>
</html>
`;
}

function generateMarkdownReport(report: any): string {
  return `# GEO Optimization Report

Generated: ${report.timestamp}

## Summary

- Total Pages: ${report.summary.totalPages}
- Average Citations: ${report.summary.avgCitations}
- Average Quality: ${report.summary.avgQuality}

`;
}

program.parse(process.argv);
