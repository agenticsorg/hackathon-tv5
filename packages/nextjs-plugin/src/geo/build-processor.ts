/**
 * GEO Build Processor for Next.js Plugin
 * Automatically analyzes and enhances content during build
 */

import { GEOOptimizer } from '@arw/geo';
import * as fs from 'fs/promises';
import * as path from 'path';
import type { GEOConfig, GEOReport, PageAnalysis } from './types';

export class GEOBuildProcessor {
  private config: GEOConfig;
  private optimizer: GEOOptimizer;

  constructor(config: GEOConfig) {
    this.config = config;

    // Initialize GEO optimizer
    this.optimizer = new GEOOptimizer({
      profile: config.profile,
      domain: config.domain,
      llm: config.llm,
    });
  }

  /**
   * Process all .llm.md files in the build directory
   */
  async processAll(buildDir: string = './public'): Promise<GEOReport> {
    console.log('[@arw/geo] GEO optimization enabled');
    console.log(`[@arw/geo] Starting GEO analysis... (Profile: ${this.config.profile})`);

    const startTime = Date.now();
    const pages: PageAnalysis[] = [];

    // Find all .llm.md files
    const llmFiles = await this.findLLMFiles(buildDir);

    console.log(`[@arw/geo] Found ${llmFiles.length} pages to analyze`);

    // Process each file
    for (const filePath of llmFiles) {
      try {
        const analysis = await this.processFile(filePath);
        pages.push(analysis);

        // Enhance the file with GEO metadata if configured
        if (this.config.output?.includeInLLMFiles) {
          await this.enhanceFile(filePath, analysis);
        }
      } catch (error) {
        console.error(`[@arw/geo] Failed to process ${filePath}:`, error);
      }
    }

    const duration = Date.now() - startTime;

    // Calculate summary statistics
    const summary = {
      totalPages: pages.length,
      avgCitations: this.calculateAverage(pages.map(p => p.analysis.citations.length)),
      avgStatistics: this.calculateAverage(pages.map(p => p.analysis.statistics.length)),
      avgQuality: this.calculateAverage(pages.map(p => p.analysis.quality.overall)),
      totalEntities: pages.reduce((sum, p) => sum + p.analysis.entities.length, 0),
    };

    console.log(`[@arw/geo] ✓ Analyzed ${pages.length} pages in ${duration}ms`);
    console.log(`[@arw/geo] ✓ Avg citations: ${summary.avgCitations.toFixed(1)}`);
    console.log(`[@arw/geo] ✓ Avg quality: ${(summary.avgQuality * 100).toFixed(0)}%`);

    const report: GEOReport = {
      timestamp: new Date().toISOString(),
      summary,
      pages,
      recommendations: this.generateRecommendations(pages),
    };

    // Generate report if configured
    if (this.config.output?.generateReports) {
      await this.saveReport(report);
    }

    return report;
  }

  /**
   * Process a single .llm.md file
   */
  private async processFile(filePath: string): Promise<PageAnalysis> {
    console.log(`[@arw/geo] Analyzing: ${path.basename(filePath)}`);

    const content = await fs.readFile(filePath, 'utf-8');

    // Run GEO analysis
    const analysis = await this.optimizer.analyze(content, {
      extractCitations: this.config.analysis?.extractCitations !== false,
      extractStatistics: this.config.analysis?.extractStatistics !== false,
      extractQuotations: this.config.analysis?.extractQuotations !== false,
      calculateQuality: this.config.analysis?.calculateQuality !== false,
      extractEntities: this.config.analysis?.extractEntities !== false,
      useLLM: this.config.analysis?.useLLM || false,
    });

    // Calculate visibility improvement
    const visibilityImprovement = this.calculateVisibilityImprovement(analysis as any);

    // Generate suggestions
    const suggestions = this.generateSuggestions(analysis as any);
    const issues = this.identifyIssues(analysis as any);

    return {
      path: filePath,
      url: this.filePathToURL(filePath),
      analysis: analysis as any,
      visibilityImprovement,
      issues,
      suggestions,
    };
  }

  /**
   * Enhance .llm.md file with GEO metadata
   */
  private async enhanceFile(filePath: string, analysis: PageAnalysis): Promise<void> {
    const content = await fs.readFile(filePath, 'utf-8');

    // Check if GEO metadata already exists
    if (content.includes('<!--GEO')) {
      return; // Already enhanced
    }

    // Add GEO metadata as HTML comment at the top
    const geoMetadata = `<!--GEO
Profile: ${this.config.profile}
Citations: ${analysis.analysis.citations.length}
Statistics: ${analysis.analysis.statistics.length}
Quotations: ${analysis.analysis.quotations.length}
Quality: ${(analysis.analysis.quality.overall * 100).toFixed(0)}%
Entities: ${analysis.analysis.entities.length}
Visibility Improvement: +${analysis.visibilityImprovement}%
Generated: ${new Date().toISOString()}
-->

`;

    const enhancedContent = geoMetadata + content;
    await fs.writeFile(filePath, enhancedContent, 'utf-8');
  }

  /**
   * Find all .llm.md files in directory
   */
  private async findLLMFiles(dir: string): Promise<string[]> {
    const files: string[] = [];

    async function walk(currentPath: string) {
      const entries = await fs.readdir(currentPath, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = path.join(currentPath, entry.name);

        if (entry.isDirectory()) {
          // Skip node_modules and hidden directories
          if (!entry.name.startsWith('.') && entry.name !== 'node_modules') {
            await walk(fullPath);
          }
        } else if (entry.name.endsWith('.llm.md')) {
          files.push(fullPath);
        }
      }
    }

    try {
      await walk(dir);
    } catch (error) {
      console.warn(`[@arw/geo] Could not scan directory ${dir}:`, error);
    }

    return files;
  }

  /**
   * Save GEO report to disk
   */
  private async saveReport(report: GEOReport): Promise<void> {
    const outputDir = this.config.output?.reportDir || './output/geo-reports';
    await fs.mkdir(outputDir, { recursive: true });

    const format = this.config.output?.reportFormat || 'json';
    const filename = `latest.${format}`;
    const outputPath = path.join(outputDir, filename);

    if (format === 'json') {
      await fs.writeFile(outputPath, JSON.stringify(report, null, 2));
    } else if (format === 'html') {
      await fs.writeFile(outputPath, this.generateHTMLReport(report));
    } else {
      await fs.writeFile(outputPath, this.generateMarkdownReport(report));
    }

    console.log(`[@arw/geo] Report saved to: ${outputPath}`);
  }

  /**
   * Calculate visibility improvement based on GEO enhancements
   */
  private calculateVisibilityImprovement(analysis: any): number {
    let improvement = 0;

    // Citations: +40%
    if (analysis.citations.length > 0) {
      improvement += 40;
    }

    // Statistics: +40%
    if (analysis.statistics.length > 0) {
      improvement += 40;
    }

    // Quotations: +40%
    if (analysis.quotations.length > 0) {
      improvement += 40;
    }

    // Quality signals: +25-35%
    if (analysis.quality.overall > 0.7) {
      improvement += 30;
    }

    // Entities: +30-40%
    if (analysis.entities.length > 5) {
      improvement += 35;
    }

    return improvement;
  }

  /**
   * Generate optimization suggestions
   */
  private generateSuggestions(analysis: any): string[] {
    const suggestions: string[] = [];

    if (analysis.citations.length < 3) {
      suggestions.push('Add 2-3 more authoritative citations to boost credibility');
    }

    if (analysis.statistics.length < 2) {
      suggestions.push('Include relevant statistics with context');
    }

    if (analysis.quotations.length === 0) {
      suggestions.push('Add expert quotations with credentials');
    }

    if (analysis.quality.overall < 0.7) {
      suggestions.push('Improve content quality and readability');
    }

    if (analysis.entities.length < 5) {
      suggestions.push('Enrich content with more named entities');
    }

    return suggestions;
  }

  /**
   * Identify issues in content
   */
  private identifyIssues(analysis: any): string[] {
    const issues: string[] = [];

    if (analysis.citations.length === 0) {
      issues.push('No citations found');
    }

    if (analysis.quality.overall < 0.5) {
      issues.push('Low quality score');
    }

    return issues;
  }

  /**
   * Generate recommendations for overall improvement
   */
  private generateRecommendations(pages: PageAnalysis[]): string[] {
    const recommendations: string[] = [];

    const lowCitationPages = pages.filter(p => p.analysis.citations.length < 3);
    if (lowCitationPages.length > 0) {
      recommendations.push(
        `${lowCitationPages.length} pages need more citations`
      );
    }

    const lowQualityPages = pages.filter(p => p.analysis.quality.overall < 0.7);
    if (lowQualityPages.length > 0) {
      recommendations.push(
        `${lowQualityPages.length} pages need quality improvements`
      );
    }

    return recommendations;
  }

  private filePathToURL(filePath: string): string {
    return filePath
      .replace(/^\.\/public/, '')
      .replace(/\.llm\.md$/, '');
  }

  private calculateAverage(numbers: number[]): number {
    if (numbers.length === 0) return 0;
    return numbers.reduce((sum, n) => sum + n, 0) / numbers.length;
  }

  private generateHTMLReport(report: GEOReport): string {
    return `<!DOCTYPE html>
<html>
<head>
  <title>GEO Optimization Report</title>
  <style>
    body { font-family: system-ui; max-width: 1200px; margin: 40px auto; padding: 0 20px; }
    .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; }
    .metric { padding: 20px; background: #f1f5f9; border-radius: 8px; }
    .metric-value { font-size: 2em; font-weight: bold; color: #2563eb; }
  </style>
</head>
<body>
  <h1>GEO Optimization Report</h1>
  <p>Generated: ${report.timestamp}</p>

  <div class="summary">
    <div class="metric">
      <div>Total Pages</div>
      <div class="metric-value">${report.summary.totalPages}</div>
    </div>
    <div class="metric">
      <div>Avg Citations</div>
      <div class="metric-value">${report.summary.avgCitations.toFixed(1)}</div>
    </div>
    <div class="metric">
      <div>Avg Quality</div>
      <div class="metric-value">${(report.summary.avgQuality * 100).toFixed(0)}%</div>
    </div>
  </div>

  <h2>Page Analysis</h2>
  ${report.pages.map(page => `
    <div style="margin: 20px 0; padding: 15px; border-left: 4px solid #2563eb;">
      <h3>${page.url}</h3>
      <p>Citations: ${page.analysis.citations.length} |
         Statistics: ${page.analysis.statistics.length} |
         Quality: ${(page.analysis.quality.overall * 100).toFixed(0)}%</p>
      <p>Visibility Improvement: +${page.visibilityImprovement}%</p>
    </div>
  `).join('')}
</body>
</html>`;
  }

  private generateMarkdownReport(report: GEOReport): string {
    return `# GEO Optimization Report

Generated: ${report.timestamp}

## Summary

- **Total Pages**: ${report.summary.totalPages}
- **Avg Citations**: ${report.summary.avgCitations.toFixed(1)}
- **Avg Statistics**: ${report.summary.avgStatistics.toFixed(1)}
- **Avg Quality**: ${(report.summary.avgQuality * 100).toFixed(0)}%
- **Total Entities**: ${report.summary.totalEntities}

## Recommendations

${report.recommendations.map(r => `- ${r}`).join('\n')}

## Page Analysis

${report.pages.map(page => `
### ${page.url}

- Citations: ${page.analysis.citations.length}
- Statistics: ${page.analysis.statistics.length}
- Quotations: ${page.analysis.quotations.length}
- Quality: ${(page.analysis.quality.overall * 100).toFixed(0)}%
- Visibility Improvement: +${page.visibilityImprovement}%

${page.suggestions.length > 0 ? `**Suggestions:**\n${page.suggestions.map(s => `- ${s}`).join('\n')}` : ''}
`).join('\n---\n')}
`;
  }
}
