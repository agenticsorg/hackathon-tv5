/**
 * Build Command - Generate all ARW files from llms.txt
 * Creates llms.json, .well-known/arw-manifest.json, and updates robots.txt
 */

import fs from 'node:fs';
import path from 'node:path';
import yaml from 'js-yaml';
import chalk from 'chalk';

export async function buildCommand(options = {}) {
  const {
    source = 'public/llms.txt',
    outputDir = 'public',
    baseUrl = 'https://example.com',
  } = options;

  console.log(chalk.bold.blue('\n🔨 ARW Build Command\n'));

  const results = {
    success: true,
    files: [],
    errors: [],
  };

  try {
    // 1. Read and parse llms.txt
    console.log(chalk.blue('1. Reading llms.txt...'));
    if (!fs.existsSync(source)) {
      throw new Error(`Source file not found: ${source}`);
    }

    const yamlContent = fs.readFileSync(source, 'utf8');
    const data = yaml.load(yamlContent);
    console.log(chalk.green('   ✓ Parsed YAML manifest'));

    // 2. Generate llms.json
    console.log(chalk.blue('\n2. Generating llms.json...'));
    const jsonPath = path.join(outputDir, 'llms.json');
    const jsonContent = JSON.stringify(data, null, 2);
    fs.writeFileSync(jsonPath, jsonContent, 'utf8');
    results.files.push(jsonPath);
    console.log(chalk.green(`   ✓ Created ${jsonPath}`));

    // 3. Generate .well-known/arw-manifest.json
    console.log(chalk.blue('\n3. Generating .well-known/arw-manifest.json...'));
    const wellKnownDir = path.join(outputDir, '.well-known');
    if (!fs.existsSync(wellKnownDir)) {
      fs.mkdirSync(wellKnownDir, { recursive: true });
    }

    const wellKnownManifest = {
      version: data.version || '1.0',
      profile: data.profile || null,
      manifestUrl: `${baseUrl}/llms.txt`,
      manifestJsonUrl: `${baseUrl}/llms.json`,
      endpoints: {
        yaml: `${baseUrl}/llms.txt`,
        json: `${baseUrl}/llms.json`,
      },
      updated: new Date().toISOString(),
    };

    const wellKnownPath = path.join(wellKnownDir, 'arw-manifest.json');
    fs.writeFileSync(wellKnownPath, JSON.stringify(wellKnownManifest, null, 2), 'utf8');
    results.files.push(wellKnownPath);
    console.log(chalk.green(`   ✓ Created ${wellKnownPath}`));

    // 4. Update or create robots.txt
    console.log(chalk.blue('\n4. Updating robots.txt...'));
    const robotsPath = path.join(outputDir, 'robots.txt');
    let robotsContent = '';

    if (fs.existsSync(robotsPath)) {
      robotsContent = fs.readFileSync(robotsPath, 'utf8');
    }

    // Check if ARW hints already exist
    if (!robotsContent.includes('Agent-Ready Web') && !robotsContent.includes('llms.txt')) {
      const arwHints = `
# Agent-Ready Web (ARW)
# Main manifest: /llms.txt
# JSON format: /llms.json
# Discovery: /.well-known/arw-manifest.json
`;
      robotsContent = robotsContent + '\n' + arwHints;
      fs.writeFileSync(robotsPath, robotsContent, 'utf8');
      results.files.push(robotsPath);
      console.log(chalk.green(`   ✓ Updated ${robotsPath} with ARW hints`));
    } else {
      console.log(chalk.yellow('   ⚠ robots.txt already contains ARW hints'));
    }

    // 5. Summary
    console.log(chalk.bold.green('\n✅ Build Complete!'));
    console.log(chalk.gray('\nGenerated files:'));
    results.files.forEach(f => console.log(chalk.cyan(`  • ${f}`)));

    console.log(chalk.gray('\nNext steps:'));
    console.log(chalk.cyan('  • Deploy these files to your hosting platform'));
    console.log(chalk.cyan('  • Configure headers for AI-* attributes'));
    console.log(chalk.cyan('  • Run: arw validate <url> --platform <platform>'));
    console.log('');

    return results;

  } catch (error) {
    console.error(chalk.red(`\n✗ Build failed: ${error.message}`));
    results.success = false;
    results.errors.push(error.message);
    process.exitCode = 1;
    return results;
  }
}
