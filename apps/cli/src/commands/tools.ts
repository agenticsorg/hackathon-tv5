/**
 * Tools command - List and install hackathon tools
 */

import Enquirer from 'enquirer';
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const enquirerPrompt = (Enquirer as any).prompt as (options: object) => Promise<any>;
const prompt = <T>(options: object): Promise<T> => enquirerPrompt(options);
import chalk from 'chalk';
import type { Tool } from '../types.js';
import { AVAILABLE_TOOLS } from '../constants.js';
import {
  logger,
  loadConfig,
  updateConfig,
  checkToolInstalled,
  installTool
} from '../utils/index.js';

interface ToolsOptions {
  install?: string[];
  list?: boolean;
  check?: boolean;
  json?: boolean;
  quiet?: boolean;
  category?: string;
  available?: boolean;
}

export async function toolsCommand(options: ToolsOptions): Promise<void> {
  // List available tools (with optional category filter)
  if (options.list || options.available || (!options.install && !options.check && options.json)) {
    await listTools(options);
    return;
  }

  // Check installed status
  if (options.check) {
    await checkTools(options);
    return;
  }

  // Install specific tools
  if (options.install && options.install.length > 0) {
    await installTools(options.install, options);
    return;
  }

  // Interactive mode (only if not json/quiet)
  if (!options.json && !options.quiet) {
    await interactiveToolInstall();
  }
}

async function listTools(options: ToolsOptions): Promise<void> {
  const categories: Record<string, string> = {
    'ai-assistants': 'AI Assistants',
    'orchestration': 'Orchestration & Agent Frameworks',
    'cloud-platform': 'Cloud Platform',
    'databases': 'Databases & Memory',
    'synthesis': 'Synthesis & Advanced Tools',
    'python-frameworks': 'Python Frameworks'
  };

  // Filter by category if specified
  let tools = AVAILABLE_TOOLS;
  if (options.category) {
    tools = AVAILABLE_TOOLS.filter(t => t.category === options.category);
    if (tools.length === 0) {
      if (options.json) {
        console.log(JSON.stringify({ success: false, error: 'invalid_category', message: `Unknown category: ${options.category}`, validCategories: Object.keys(categories) }));
        process.exit(1);
      }
      logger.error(`Unknown category: ${options.category}`);
      logger.info(`Valid categories: ${Object.keys(categories).join(', ')}`);
      return;
    }
  }

  // JSON output
  if (options.json) {
    const toolsWithStatus = await Promise.all(tools.map(async (tool) => ({
      name: tool.name,
      displayName: tool.displayName,
      description: tool.description,
      category: tool.category,
      installCommand: tool.installCommand,
      docUrl: tool.docUrl,
      installed: await checkToolInstalled(tool)
    })));
    console.log(JSON.stringify({ success: true, tools: toolsWithStatus, categories: Object.keys(categories) }));
    return;
  }

  logger.info('Available tools for the hackathon:\n');

  for (const [category, label] of Object.entries(categories)) {
    if (options.category && options.category !== category) continue;
    const categoryTools = tools.filter(t => t.category === category);
    if (categoryTools.length > 0) {
      console.log(chalk.bold.cyan(`\n${label}:`));
      for (const tool of categoryTools) {
        const installed = await checkToolInstalled(tool);
        const status = installed ? chalk.green('✔') : chalk.gray('○');
        console.log(`  ${status} ${chalk.bold(tool.displayName)}`);
        console.log(`    ${chalk.gray(tool.description)}`);
        console.log(`    ${chalk.gray('Install:')} ${chalk.cyan(tool.installCommand)}`);
      }
    }
  }

  logger.newline();
  logger.info('Run `npx agentics-hackathon tools --install <tool>` to install a specific tool');
  logger.info('Run `npx agentics-hackathon tools --check` to check installed status');
}

async function checkTools(options: ToolsOptions): Promise<void> {
  const results: { tool: Tool; installed: boolean }[] = [];

  for (const tool of AVAILABLE_TOOLS) {
    const installed = await checkToolInstalled(tool);
    results.push({ tool, installed });
  }

  const installedTools = results.filter(r => r.installed);
  const notInstalledTools = results.filter(r => !r.installed);

  // JSON output
  if (options.json) {
    console.log(JSON.stringify({
      success: true,
      installed: installedTools.map(r => ({ name: r.tool.name, displayName: r.tool.displayName })),
      notInstalled: notInstalledTools.map(r => ({ name: r.tool.name, displayName: r.tool.displayName })),
      summary: { installed: installedTools.length, total: results.length }
    }));
    return;
  }

  logger.info('Checking installed tools...\n');

  if (installedTools.length > 0) {
    console.log(chalk.bold.green('Installed:'));
    installedTools.forEach(({ tool }) => {
      console.log(`  ${chalk.green('✔')} ${tool.displayName}`);
    });
  }

  if (notInstalledTools.length > 0) {
    console.log(chalk.bold.yellow('\nNot Installed:'));
    notInstalledTools.forEach(({ tool }) => {
      console.log(`  ${chalk.gray('○')} ${tool.displayName}`);
    });
  }

  logger.newline();
  logger.info(`${installedTools.length}/${results.length} tools installed`);
}

async function installTools(toolNames: string[], options: ToolsOptions): Promise<void> {
  const config = loadConfig();
  const results: { name: string; success: boolean; error?: string }[] = [];

  for (const name of toolNames) {
    const tool = AVAILABLE_TOOLS.find(
      t => t.name === name || t.displayName.toLowerCase() === name.toLowerCase()
    );

    if (!tool) {
      if (options.json) {
        results.push({ name, success: false, error: 'unknown_tool' });
        continue;
      }
      logger.error(`Unknown tool: ${name}`);
      logger.info('Run `npx agentics-hackathon tools --list` to see available tools');
      continue;
    }

    if (options.json || options.quiet) {
      // Silent install - just update config
      results.push({ name: tool.name, success: true });
      if (config) {
        const toolKey = tool.name as keyof typeof config.tools;
        updateConfig({
          tools: { ...config.tools, [toolKey]: true }
        });
      }
    } else {
      const result = await installTool(tool);
      results.push({ name: tool.name, success: result.status === 'success', error: result.message });

      if (result.status === 'success' && config) {
        const toolKey = tool.name as keyof typeof config.tools;
        updateConfig({
          tools: { ...config.tools, [toolKey]: true }
        });
      }
    }
  }

  if (options.json) {
    console.log(JSON.stringify({
      success: results.every(r => r.success),
      results,
      summary: { successful: results.filter(r => r.success).length, total: results.length }
    }));
  }
}

async function interactiveToolInstall(): Promise<void> {
  const choices = AVAILABLE_TOOLS.map(tool => ({
    name: tool.name,
    message: tool.displayName,
    hint: tool.description
  }));

  const { selectedTools } = await prompt<{ selectedTools: string[] }>({
    type: 'multiselect',
    name: 'selectedTools',
    message: 'Select tools to install:',
    choices
  });

  if (selectedTools.length === 0) {
    logger.info('No tools selected');
    return;
  }

  await installTools(selectedTools, {});
}
