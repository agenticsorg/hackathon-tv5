/**
 * Status command - Show current hackathon project status
 */

import chalk from 'chalk';
import { AVAILABLE_TOOLS, TRACKS, DISCORD_URL, WEBSITE_URL } from '../constants.js';
import { logger, loadConfig, configExists, checkToolInstalled } from '../utils/index.js';

interface StatusOptions {
  json?: boolean;
  quiet?: boolean;
}

export async function statusCommand(options: StatusOptions = {}): Promise<void> {
  if (!configExists()) {
    if (options.json) {
      console.log(JSON.stringify({ success: false, error: 'not_initialized', message: 'Project not initialized. Run init first.' }));
      process.exit(1);
    }
    logger.warning('Not initialized. Run `npx agentics-hackathon init` first.');
    return;
  }

  const config = loadConfig();
  if (!config) {
    if (options.json) {
      console.log(JSON.stringify({ success: false, error: 'config_error', message: 'Failed to load configuration.' }));
      process.exit(1);
    }
    logger.error('Failed to load configuration');
    return;
  }

  // Check tools status
  const enabledTools = Object.entries(config.tools)
    .filter(([_, enabled]) => enabled)
    .map(([name]) => name);

  const toolsStatus: { name: string; displayName: string; installed: boolean }[] = [];
  for (const toolName of enabledTools) {
    const tool = AVAILABLE_TOOLS.find(t => t.name === toolName);
    if (tool) {
      const installed = await checkToolInstalled(tool);
      toolsStatus.push({ name: tool.name, displayName: tool.displayName, installed });
    }
  }

  // JSON output
  if (options.json) {
    console.log(JSON.stringify({
      success: true,
      config: {
        projectName: config.projectName,
        teamName: config.teamName,
        track: config.track,
        trackName: config.track ? TRACKS[config.track].name : null,
        mcpEnabled: config.mcpEnabled,
        discordLinked: config.discordLinked,
        initialized: config.initialized,
        createdAt: config.createdAt
      },
      tools: toolsStatus,
      resources: {
        website: WEBSITE_URL,
        discord: DISCORD_URL,
        configFile: '.hackathon.json'
      }
    }));
    return;
  }

  logger.divider();
  console.log(chalk.bold.cyan('  Hackathon Project Status'));
  logger.divider();
  logger.newline();

  // Project info
  logger.table({
    'Project': config.projectName,
    'Team': config.teamName || 'Not set',
    'Track': config.track ? TRACKS[config.track].name : 'Not selected',
    'Initialized': new Date(config.createdAt).toLocaleDateString(),
    'MCP Server': config.mcpEnabled ? 'Enabled' : 'Disabled',
    'Discord': config.discordLinked ? 'Connected' : 'Not connected'
  });

  logger.newline();
  logger.divider();
  console.log(chalk.bold.cyan('  Tools Status'));
  logger.divider();
  logger.newline();

  if (toolsStatus.length === 0) {
    logger.info('No tools configured. Run `npx agentics-hackathon tools` to install tools.');
  } else {
    for (const { displayName, installed } of toolsStatus) {
      const status = installed ? chalk.green('✔ Ready') : chalk.yellow('⚠ Needs setup');
      console.log(`  ${status} ${displayName}`);
    }
  }

  logger.newline();
  logger.divider();
  console.log(chalk.bold.cyan('  Resources'));
  logger.divider();
  logger.newline();

  logger.table({
    'Website': WEBSITE_URL,
    'Discord': DISCORD_URL,
    'Config File': '.hackathon.json'
  });

  logger.newline();
}
