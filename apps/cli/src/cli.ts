#!/usr/bin/env node
/**
 * Agentics Foundation TV5 Hackathon CLI
 *
 * Main entry point for the hackathon CLI tool.
 * Run with: npx @agenticsorg/hackathon
 */

import { Command } from 'commander';
import chalk from 'chalk';
import { createRequire } from 'module';

import { initCommand, toolsCommand, statusCommand, infoCommand, helpCommand } from './commands/index.js';
import { startSseServer } from './mcp/sse.js';
import { BANNER, DISCORD_URL, WEBSITE_URL, HACKATHON_NAME } from './constants.js';
import { logger } from './utils/index.js';

// Read version from package.json
const require = createRequire(import.meta.url);
const { version } = require('../package.json');

const program = new Command();

program
  .name('hackathon')
  .description(`${HACKATHON_NAME} CLI - Build the future of agentic AI`)
  .version(version)
  .hook('preAction', () => {
    // Show abbreviated banner for commands
  });

// Init command
program
  .command('init')
  .description('Initialize a new hackathon project with interactive setup')
  .option('-f, --force', 'Force reinitialize even if already configured')
  .option('-y, --yes', 'Skip prompts and use defaults')
  .option('-t, --tools <tools...>', 'Tools to install (space-separated)')
  .option('--track <track>', 'Hackathon track to participate in')
  .option('--team <name>', 'Team name')
  .option('--project <name>', 'Project name')
  .option('--mcp', 'Enable MCP server')
  .option('--json', 'Output result as JSON (implies --yes)')
  .option('-q, --quiet', 'Suppress non-essential output')
  .action(async (options) => {
    await initCommand(options);
  });

// Tools command
program
  .command('tools')
  .description('List, check, or install hackathon development tools')
  .option('-l, --list', 'List all available tools')
  .option('-c, --check', 'Check which tools are installed')
  .option('-i, --install <tools...>', 'Install specific tools')
  .option('--category <category>', 'Filter by category (ai-assistants, orchestration, databases, cloud-platform, synthesis, python-frameworks)')
  .option('--available', 'List available tools (alias for --list)')
  .option('--json', 'Output result as JSON')
  .option('-q, --quiet', 'Suppress non-essential output')
  .action(async (options) => {
    await toolsCommand(options);
  });

// Status command
program
  .command('status')
  .description('Show current hackathon project status')
  .option('--json', 'Output result as JSON')
  .option('-q, --quiet', 'Suppress non-essential output')
  .action(async (options) => {
    await statusCommand(options);
  });

// Info command
program
  .command('info')
  .description('Display hackathon information and resources')
  .option('--json', 'Output result as JSON')
  .option('-q, --quiet', 'Suppress non-essential output')
  .action(async (options) => {
    await infoCommand(options);
  });

// MCP command
program
  .command('mcp')
  .description('Start the MCP (Model Context Protocol) server')
  .argument('[transport]', 'Transport type: stdio or sse', 'stdio')
  .option('-p, --port <port>', 'Port for SSE server', '3000')
  .action(async (transport, options) => {
    if (transport === 'sse') {
      startSseServer(parseInt(options.port, 10));
    } else {
      // Import and run STDIO server
      await import('./mcp/stdio.js');
    }
  });

// Discord command
program
  .command('discord')
  .description('Open Discord for team coordination and support')
  .option('--json', 'Output result as JSON')
  .action((options) => {
    if (options.json) {
      console.log(JSON.stringify({ success: true, discord: DISCORD_URL }));
      return;
    }
    logger.box(
      `Join the Agentics Foundation Discord community!\n\n` +
      `${chalk.bold('Benefits:')}\n` +
      `  • Team formation & networking\n` +
      `  • Technical support & mentorship\n` +
      `  • Announcements & updates\n` +
      `  • Share your progress\n\n` +
      chalk.cyan.bold.underline(DISCORD_URL),
      'Discord Community'
    );
    logger.newline();
    logger.info('Open the URL above in your browser to join!');
  });

// Website command
program
  .command('website')
  .alias('web')
  .description('Open the hackathon website')
  .option('--json', 'Output result as JSON')
  .action((options) => {
    if (options.json) {
      console.log(JSON.stringify({ success: true, website: WEBSITE_URL }));
      return;
    }
    logger.box(
      `Visit the official hackathon website:\n\n` +
      chalk.cyan.bold.underline(WEBSITE_URL),
      'Hackathon Website'
    );
  });

// Help command with topics
program
  .command('help [topic]')
  .description('Show detailed help (topics: init, tools, mcp, tracks, examples, packages)')
  .action(async (topic) => {
    await helpCommand({ topic });
  });

// Default action (no command)
program
  .action(async () => {
    // Show full banner and menu
    logger.banner(BANNER);
    logger.newline();

    console.log(chalk.bold('  Welcome to the Agentics Foundation TV5 Hackathon!'));
    console.log(chalk.gray('  Build the future of agentic AI with Google Cloud'));
    logger.newline();

    console.log(chalk.bold.cyan('  Quick Commands:'));
    logger.newline();

    const commands = [
      { cmd: 'npx agentics-hackathon init', desc: 'Initialize a new project' },
      { cmd: 'npx agentics-hackathon tools', desc: 'Browse and install 17+ AI tools' },
      { cmd: 'npx agentics-hackathon status', desc: 'Check project status' },
      { cmd: 'npx agentics-hackathon info', desc: 'View hackathon details' },
      { cmd: 'npx agentics-hackathon mcp', desc: 'Start MCP server' },
      { cmd: 'npx agentics-hackathon discord', desc: 'Join the community' },
      { cmd: 'npx agentics-hackathon help', desc: 'Detailed help & examples' }
    ];

    commands.forEach(({ cmd, desc }) => {
      console.log(`  ${chalk.cyan(cmd.padEnd(24))} ${chalk.gray(desc)}`);
    });

    logger.newline();
    logger.divider();
    logger.newline();

    console.log(chalk.bold('  Get Started:'));
    console.log(`  ${chalk.yellow('$')} ${chalk.cyan('npx agentics-hackathon init')}`);
    logger.newline();

    console.log(chalk.bold('  Resources:'));
    console.log(`  ${chalk.gray('Website:')}  ${chalk.cyan.underline(WEBSITE_URL)}`);
    console.log(`  ${chalk.gray('Discord:')}  ${chalk.cyan.underline(DISCORD_URL)}`);
    logger.newline();
  });

// Parse arguments
program.parse();
