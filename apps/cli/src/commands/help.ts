/**
 * Help command - Detailed help and documentation
 */

import chalk from 'chalk';
import {
  HACKATHON_NAME,
  TRACKS,
  AVAILABLE_TOOLS,
  DISCORD_URL,
  WEBSITE_URL,
  GITHUB_URL
} from '../constants.js';
import { logger } from '../utils/index.js';

interface HelpOptions {
  topic?: string;
}

export async function helpCommand(options: HelpOptions): Promise<void> {
  const topic = options.topic?.toLowerCase();

  switch (topic) {
    case 'init':
      showInitHelp();
      break;
    case 'tools':
      showToolsHelp();
      break;
    case 'mcp':
      showMcpHelp();
      break;
    case 'tracks':
      showTracksHelp();
      break;
    case 'examples':
      showExamplesHelp();
      break;
    case 'packages':
      showPackagesHelp();
      break;
    default:
      showGeneralHelp();
  }
}

function showGeneralHelp(): void {
  console.log(chalk.bold.cyan(`
╔══════════════════════════════════════════════════════════════════════════════╗
║                     ${HACKATHON_NAME}                      ║
║                            DETAILED HELP                                     ║
╚══════════════════════════════════════════════════════════════════════════════╝
`));

  console.log(chalk.bold('DESCRIPTION'));
  console.log(`  The Agentics Hackathon CLI helps you set up and manage your hackathon
  project with support for 17+ AI development tools across 6 categories.
`);

  console.log(chalk.bold('QUICK START'));
  console.log(`  ${chalk.cyan('npx agentics-hackathon init')}     Initialize a new project
  ${chalk.cyan('npx agentics-hackathon tools')}    Browse & install AI tools
  ${chalk.cyan('npx agentics-hackathon info')}     View hackathon information
`);

  console.log(chalk.bold('COMMANDS'));
  console.log(`  ${chalk.cyan('init')}      Initialize a hackathon project with interactive setup
  ${chalk.cyan('tools')}     List, check, or install development tools
  ${chalk.cyan('status')}    Show current project configuration
  ${chalk.cyan('info')}      Display hackathon details and resources
  ${chalk.cyan('mcp')}       Start MCP server (stdio or sse transport)
  ${chalk.cyan('discord')}   Open Discord community link
  ${chalk.cyan('help')}      Show this help or topic-specific help
`);

  console.log(chalk.bold('HELP TOPICS'));
  console.log(`  ${chalk.cyan('npx agentics-hackathon help init')}       Setup and initialization options
  ${chalk.cyan('npx agentics-hackathon help tools')}      Available tools and installation
  ${chalk.cyan('npx agentics-hackathon help mcp')}        MCP server configuration
  ${chalk.cyan('npx agentics-hackathon help tracks')}     Hackathon track descriptions
  ${chalk.cyan('npx agentics-hackathon help examples')}   Usage examples and workflows
  ${chalk.cyan('npx agentics-hackathon help packages')}   All available packages detailed
`);

  console.log(chalk.bold('RESOURCES'));
  console.log(`  ${chalk.gray('Website:')}   ${chalk.cyan.underline(WEBSITE_URL)}
  ${chalk.gray('Discord:')}   ${chalk.cyan.underline(DISCORD_URL)}
  ${chalk.gray('GitHub:')}    ${chalk.cyan.underline(GITHUB_URL)}
`);
}

function showInitHelp(): void {
  console.log(chalk.bold.cyan('\n═══ INIT COMMAND HELP ═══\n'));

  console.log(chalk.bold('USAGE'));
  console.log(`  ${chalk.cyan('npx agentics-hackathon init [options]')}
`);

  console.log(chalk.bold('DESCRIPTION'));
  console.log(`  Initialize a new hackathon project with an interactive setup wizard.
  Creates a .hackathon.json config file and optionally installs tools.
`);

  console.log(chalk.bold('OPTIONS'));
  console.log(`  ${chalk.cyan('-f, --force')}           Reinitialize even if already configured
  ${chalk.cyan('-y, --yes')}             Skip prompts and use defaults
  ${chalk.cyan('-t, --tools <list>')}    Tools to install (space-separated)
  ${chalk.cyan('--track <track>')}       Select hackathon track
  ${chalk.cyan('--team <name>')}         Set team name
`);

  console.log(chalk.bold('EXAMPLES'));
  console.log(`  ${chalk.gray('# Interactive setup (recommended)')}
  ${chalk.cyan('npx agentics-hackathon init')}

  ${chalk.gray('# Quick setup with specific tools')}
  ${chalk.cyan('npx agentics-hackathon init --tools claudeFlow agenticFlow adk')}

  ${chalk.gray('# Non-interactive with all options')}
  ${chalk.cyan('npx agentics-hackathon init -y --team "AI Wizards" --track multi-agent-systems')}

  ${chalk.gray('# Force reinitialize')}
  ${chalk.cyan('npx agentics-hackathon init --force')}
`);
}

function showToolsHelp(): void {
  console.log(chalk.bold.cyan('\n═══ TOOLS COMMAND HELP ═══\n'));

  console.log(chalk.bold('USAGE'));
  console.log(`  ${chalk.cyan('npx agentics-hackathon tools [options]')}
`);

  console.log(chalk.bold('DESCRIPTION'));
  console.log(`  Browse, check, and install AI development tools for the hackathon.
  Currently supports ${AVAILABLE_TOOLS.length} tools across 6 categories.
`);

  console.log(chalk.bold('OPTIONS'));
  console.log(`  ${chalk.cyan('-l, --list')}              List all available tools
  ${chalk.cyan('-c, --check')}             Check which tools are installed
  ${chalk.cyan('-i, --install <tools>')}   Install specific tools
`);

  console.log(chalk.bold('CATEGORIES'));
  const categories = {
    'ai-assistants': 'AI Assistants',
    'orchestration': 'Orchestration & Agent Frameworks',
    'cloud-platform': 'Cloud Platform',
    'databases': 'Databases & Memory',
    'synthesis': 'Synthesis & Advanced Tools',
    'python-frameworks': 'Python Frameworks'
  };

  Object.entries(categories).forEach(([key, label]) => {
    const tools = AVAILABLE_TOOLS.filter(t => t.category === key);
    console.log(`  ${chalk.bold.magenta(label)} (${tools.length} tools)`);
    tools.forEach(t => {
      console.log(`    ${chalk.cyan(t.name.padEnd(20))} ${chalk.gray(t.description.substring(0, 50))}...`);
    });
  });

  console.log(`
${chalk.bold('EXAMPLES')}
  ${chalk.gray('# List all tools')}
  ${chalk.cyan('npx agentics-hackathon tools --list')}

  ${chalk.gray('# Check installed status')}
  ${chalk.cyan('npx agentics-hackathon tools --check')}

  ${chalk.gray('# Install multiple tools')}
  ${chalk.cyan('npx agentics-hackathon tools --install claudeFlow agenticFlow lionpride')}
`);
}

function showMcpHelp(): void {
  console.log(chalk.bold.cyan('\n═══ MCP SERVER HELP ═══\n'));

  console.log(chalk.bold('USAGE'));
  console.log(`  ${chalk.cyan('npx agentics-hackathon mcp [transport] [options]')}
`);

  console.log(chalk.bold('DESCRIPTION'));
  console.log(`  Start an MCP (Model Context Protocol) server for AI integration.
  Supports both STDIO and SSE (Server-Sent Events) transports.
`);

  console.log(chalk.bold('TRANSPORTS'));
  console.log(`  ${chalk.cyan('stdio')}    Standard input/output (default) - for local AI tools
  ${chalk.cyan('sse')}      Server-Sent Events - for web-based integrations
`);

  console.log(chalk.bold('OPTIONS'));
  console.log(`  ${chalk.cyan('-p, --port <port>')}    Port for SSE server (default: 3000)
`);

  console.log(chalk.bold('MCP TOOLS PROVIDED'));
  console.log(`  ${chalk.cyan('get_hackathon_info')}     Get hackathon information
  ${chalk.cyan('get_tracks')}             List available tracks
  ${chalk.cyan('get_available_tools')}    List development tools
  ${chalk.cyan('get_project_status')}     Check project configuration
  ${chalk.cyan('check_tool_installed')}   Verify tool installation
  ${chalk.cyan('get_resources')}          Get hackathon resources
`);

  console.log(chalk.bold('CLAUDE DESKTOP CONFIG'));
  console.log(`  Add to your Claude configuration (~/.claude/claude_desktop_config.json):

  ${chalk.cyan(`{
    "mcpServers": {
      "hackathon": {
        "command": "npx",
        "args": ["agentics-hackathon", "mcp", "stdio"]
      }
    }
  }`)}
`);

  console.log(chalk.bold('EXAMPLES'));
  console.log(`  ${chalk.gray('# Start STDIO server')}
  ${chalk.cyan('npx agentics-hackathon mcp stdio')}

  ${chalk.gray('# Start SSE server on port 3001')}
  ${chalk.cyan('npx agentics-hackathon mcp sse --port 3001')}
`);
}

function showTracksHelp(): void {
  console.log(chalk.bold.cyan('\n═══ HACKATHON TRACKS ═══\n'));

  Object.entries(TRACKS).forEach(([key, { name, description }]) => {
    console.log(chalk.bold.magenta(`${name}`));
    console.log(`  ${chalk.gray('ID:')} ${chalk.cyan(key)}`);
    console.log(`  ${chalk.gray('Description:')} ${description}`);
    console.log();
  });

  console.log(chalk.bold('RECOMMENDED TOOLS BY TRACK'));
  console.log(`
  ${chalk.bold.magenta('Entertainment Discovery')}
    ${chalk.cyan('claudeFlow, geminiCli, vertexAi, ruvector')}

  ${chalk.bold.magenta('Multi-Agent Systems')}
    ${chalk.cyan('agenticFlow, flowNexus, adk, lionpride, openaiAgents')}

  ${chalk.bold.magenta('Agentic Workflows')}
    ${chalk.cyan('claudeFlow, sparc, strangeLoops, agenticFramework')}

  ${chalk.bold.magenta('Open Innovation')}
    ${chalk.cyan('Choose based on your project needs!')}
`);
}

function showExamplesHelp(): void {
  console.log(chalk.bold.cyan('\n═══ USAGE EXAMPLES ═══\n'));

  console.log(chalk.bold('GETTING STARTED'));
  console.log(`  ${chalk.gray('# 1. Initialize your project')}
  ${chalk.cyan('npx agentics-hackathon init')}

  ${chalk.gray('# 2. Install recommended tools')}
  ${chalk.cyan('npx agentics-hackathon tools --install claudeFlow agenticFlow adk')}

  ${chalk.gray('# 3. Check your setup')}
  ${chalk.cyan('npx agentics-hackathon status')}
`);

  console.log(chalk.bold('MULTI-AGENT WORKFLOW'));
  console.log(`  ${chalk.gray('# Set up a multi-agent project')}
  ${chalk.cyan('npx agentics-hackathon init --track multi-agent-systems')}
  ${chalk.cyan('npx agentics-hackathon tools --install agenticFlow flowNexus lionpride')}
  ${chalk.cyan('npx agentic-flow init')}
  ${chalk.cyan('npx flow-nexus init')}
`);

  console.log(chalk.bold('USING WITH CLAUDE'));
  console.log(`  ${chalk.gray('# Start MCP server for Claude integration')}
  ${chalk.cyan('npx agentics-hackathon mcp stdio')}

  ${chalk.gray('# Or for web-based access')}
  ${chalk.cyan('npx agentics-hackathon mcp sse --port 3000')}
`);

  console.log(chalk.bold('PYTHON-BASED PROJECT'));
  console.log(`  ${chalk.gray('# Install Python agent frameworks')}
  ${chalk.cyan('npx agentics-hackathon tools --install lionpride agenticFramework openaiAgents adk')}

  ${chalk.gray('# Then in your Python code:')}
  ${chalk.green(`from lionpride import Agent
from agentic import create_agent`)}
`);

  console.log(chalk.bold('QUICK DEMO'));
  console.log(`  ${chalk.gray('# One-liner to test everything')}
  ${chalk.cyan('npx agentics-hackathon init -y && npx agentics-hackathon tools --list && npx agentics-hackathon status')}
`);
}

function showPackagesHelp(): void {
  console.log(chalk.bold.cyan('\n═══ ALL AVAILABLE PACKAGES ═══\n'));

  console.log(chalk.bold('NPM PACKAGES (Node.js)'));
  logger.divider();

  const npmTools = AVAILABLE_TOOLS.filter(t =>
    t.installCommand.includes('npm') || t.installCommand.includes('npx')
  );

  npmTools.forEach(tool => {
    console.log(`
  ${chalk.bold.cyan(tool.displayName)} (${chalk.gray(tool.name)})
    ${tool.description}
    ${chalk.yellow('Install:')} ${chalk.white(tool.installCommand)}
    ${chalk.yellow('Docs:')}    ${chalk.cyan.underline(tool.docUrl)}`);
  });

  console.log(chalk.bold('\n\nPIP PACKAGES (Python)'));
  logger.divider();

  const pipTools = AVAILABLE_TOOLS.filter(t => t.installCommand.includes('pip'));

  pipTools.forEach(tool => {
    console.log(`
  ${chalk.bold.cyan(tool.displayName)} (${chalk.gray(tool.name)})
    ${tool.description}
    ${chalk.yellow('Install:')} ${chalk.white(tool.installCommand)}
    ${chalk.yellow('Docs:')}    ${chalk.cyan.underline(tool.docUrl)}`);
  });

  console.log(chalk.bold('\n\nOTHER INSTALLATIONS'));
  logger.divider();

  const otherTools = AVAILABLE_TOOLS.filter(t =>
    !t.installCommand.includes('npm') &&
    !t.installCommand.includes('npx') &&
    !t.installCommand.includes('pip')
  );

  otherTools.forEach(tool => {
    console.log(`
  ${chalk.bold.cyan(tool.displayName)} (${chalk.gray(tool.name)})
    ${tool.description}
    ${chalk.yellow('Install:')} ${chalk.white(tool.installCommand)}
    ${chalk.yellow('Docs:')}    ${chalk.cyan.underline(tool.docUrl)}`);
  });

  console.log(`

${chalk.bold('QUICK INSTALL BY CATEGORY')}
  ${chalk.gray('# All orchestration tools')}
  ${chalk.cyan('npx agentics-hackathon tools --install claudeFlow agenticFlow flowNexus adk')}

  ${chalk.gray('# All Python frameworks')}
  ${chalk.cyan('npx agentics-hackathon tools --install lionpride agenticFramework openaiAgents')}

  ${chalk.gray('# Recommended starter pack')}
  ${chalk.cyan('npx agentics-hackathon tools --install claudeCode claudeFlow agenticFlow geminiCli')}
`);
}
