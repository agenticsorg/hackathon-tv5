/**
 * Init command - Interactive setup wizard for hackathon projects
 */

import Enquirer from 'enquirer';
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const enquirerPrompt = (Enquirer as any).prompt as (options: object) => Promise<any>;
const prompt = <T>(options: object): Promise<T> => enquirerPrompt(options);
import chalk from 'chalk';
import ora from 'ora';
import path from 'path';
import type { HackathonConfig, HackathonTrack, ToolSelection } from '../types.js';
import {
  HACKATHON_NAME,
  TRACKS,
  AVAILABLE_TOOLS,
  DISCORD_URL,
  WEBSITE_URL,
  BANNER,
  WELCOME_MESSAGE
} from '../constants.js';
import {
  logger,
  configExists,
  loadConfig,
  saveConfig,
  createDefaultConfig,
  checkPrerequisites,
  installTool
} from '../utils/index.js';

interface InitOptions {
  force?: boolean;
  yes?: boolean;
  tools?: string[];
  track?: string;
  team?: string;
  project?: string;
  mcp?: boolean;
  json?: boolean;
  quiet?: boolean;
}

/**
 * Validates and sanitizes a project name to prevent path traversal attacks
 * @param name - The project name to validate
 * @returns A sanitized project name or default if validation fails
 */
function sanitizeProjectName(name: string): string {
  // Default fallback
  const defaultName = 'hackathon-project';

  if (!name || typeof name !== 'string') {
    return defaultName;
  }

  // Trim whitespace
  const trimmed = name.trim();

  // Reject empty strings
  if (!trimmed) {
    return defaultName;
  }

  // Reject path traversal patterns: .., /, \, and other dangerous characters
  const dangerousPatterns = /[./\\:*?"<>|]/;
  if (dangerousPatterns.test(trimmed)) {
    return defaultName;
  }

  // Only allow alphanumeric characters, hyphens, and underscores
  const validPattern = /^[a-zA-Z0-9_-]+$/;
  if (!validPattern.test(trimmed)) {
    return defaultName;
  }

  // Reject names that are too long (max 100 characters)
  if (trimmed.length > 100) {
    return defaultName;
  }

  return trimmed;
}

/**
 * Gets a safe default project name from the current working directory
 * @returns A sanitized project name
 */
function getDefaultProjectName(): string {
  try {
    const basename = path.basename(process.cwd());
    return sanitizeProjectName(basename);
  } catch {
    return 'hackathon-project';
  }
}

export async function initCommand(options: InitOptions): Promise<void> {
  const isQuiet = options.quiet || options.json;

  // Show banner (unless quiet/json mode)
  if (!isQuiet) {
    logger.banner(BANNER);
    console.log(WELCOME_MESSAGE);
    logger.divider();
  }

  // Check if already initialized
  if (configExists() && !options.force) {
    const config = loadConfig();
    if (config?.initialized) {
      if (options.json) {
        console.log(JSON.stringify({ success: false, error: 'already_initialized', message: 'Project already initialized. Use --force to reinitialize.' }));
        process.exit(1);
      }
      logger.warning('Project already initialized!');
      logger.info('Use --force to reinitialize');
      return;
    }
  }

  // Check prerequisites
  if (!isQuiet) logger.info('Checking prerequisites...');
  const prereqs = await checkPrerequisites();

  if (!prereqs.node || !prereqs.npm) {
    if (options.json) {
      console.log(JSON.stringify({ success: false, error: 'missing_prerequisites', message: 'Node.js and npm are required.', prereqs }));
      process.exit(1);
    }
    logger.error('Node.js and npm are required. Please install them first.');
    logger.link('Download Node.js', 'https://nodejs.org');
    return;
  }

  if (!isQuiet) {
    logger.success('Prerequisites check passed');
    logger.newline();
  }

  let config: HackathonConfig;

  if (options.yes || options.json) {
    // Non-interactive mode with defaults
    config = await runNonInteractive(options);
  } else {
    // Interactive mode
    config = await runInteractive(options);
  }

  // Save configuration
  if (!isQuiet) {
    const spinner = ora('Saving configuration...').start();
    config.initialized = true;
    saveConfig(config);
    spinner.succeed('Configuration saved');
  } else {
    config.initialized = true;
    saveConfig(config);
  }

  // Output result
  if (options.json) {
    console.log(JSON.stringify({ success: true, config }));
  } else {
    showSummary(config);
  }
}

async function runInteractive(options: InitOptions): Promise<HackathonConfig> {
  // Project name
  const { projectName: rawProjectName } = await prompt<{ projectName: string }>({
    type: 'input',
    name: 'projectName',
    message: 'Project name:',
    initial: getDefaultProjectName()
  });

  // Sanitize user input
  const projectName = sanitizeProjectName(rawProjectName);

  // Team name
  const { teamName } = await prompt<{ teamName: string }>({
    type: 'input',
    name: 'teamName',
    message: 'Team name (optional):',
    initial: options.team || ''
  });

  // Track selection
  const trackChoices = Object.entries(TRACKS).map(([value, { name, description }]) => ({
    name: value,
    message: name,
    hint: description
  }));

  const { track } = await prompt<{ track: HackathonTrack }>({
    type: 'select',
    name: 'track',
    message: 'Select hackathon track:',
    choices: trackChoices,
    initial: options.track ? Object.keys(TRACKS).indexOf(options.track) : 0
  });

  logger.newline();
  logger.info('Select tools to install (all optional):');
  logger.newline();

  // Group tools by category
  const toolCategories = {
    'AI Assistants': AVAILABLE_TOOLS.filter(t => t.category === 'ai-assistants'),
    'Orchestration': AVAILABLE_TOOLS.filter(t => t.category === 'orchestration'),
    'Databases': AVAILABLE_TOOLS.filter(t => t.category === 'databases'),
    'Cloud Platform': AVAILABLE_TOOLS.filter(t => t.category === 'cloud-platform'),
    'Synthesis': AVAILABLE_TOOLS.filter(t => t.category === 'synthesis')
  };

  const toolChoices = Object.entries(toolCategories).flatMap(([category, tools]) => [
    { name: `--- ${category} ---`, disabled: true } as any,
    ...tools.map(tool => ({
      name: tool.name,
      message: `${tool.displayName}`,
      hint: tool.description,
      value: tool.name
    }))
  ]);

  const { selectedTools } = await (prompt as any)({
    type: 'multiselect',
    name: 'selectedTools',
    message: 'Select tools to install:',
    choices: toolChoices,
    initial: options.tools || []
  }) as { selectedTools: string[] };

  // MCP configuration
  const { enableMcp } = await prompt<{ enableMcp: boolean }>({
    type: 'confirm',
    name: 'enableMcp',
    message: 'Enable MCP (Model Context Protocol) server?',
    initial: false
  });

  // Discord
  const { joinDiscord } = await prompt<{ joinDiscord: boolean }>({
    type: 'confirm',
    name: 'joinDiscord',
    message: `Join Discord for team coordination? (${DISCORD_URL})`,
    initial: true
  });

  // Build configuration
  const tools: ToolSelection = {
    // AI Assistants
    claudeCode: selectedTools.includes('claudeCode'),
    geminiCli: selectedTools.includes('geminiCli'),
    // Orchestration
    claudeFlow: selectedTools.includes('claudeFlow'),
    agenticFlow: selectedTools.includes('agenticFlow'),
    flowNexus: selectedTools.includes('flowNexus'),
    adk: selectedTools.includes('adk'),
    // Cloud Platform
    googleCloudCli: selectedTools.includes('googleCloudCli'),
    vertexAi: selectedTools.includes('vertexAi'),
    // Databases
    ruvector: selectedTools.includes('ruvector'),
    agentDb: selectedTools.includes('agentDb'),
    // Synthesis
    agenticSynth: selectedTools.includes('agenticSynth'),
    strangeLoops: selectedTools.includes('strangeLoops'),
    sparc: selectedTools.includes('sparc'),
    // Python Frameworks
    lionpride: selectedTools.includes('lionpride'),
    agenticFramework: selectedTools.includes('agenticFramework'),
    openaiAgents: selectedTools.includes('openaiAgents')
  };

  // Install selected tools
  if (selectedTools.length > 0) {
    logger.newline();
    logger.divider();
    logger.info('Installing selected tools...');
    logger.newline();

    for (const toolName of selectedTools) {
      const tool = AVAILABLE_TOOLS.find(t => t.name === toolName);
      if (tool) {
        await installTool(tool);
      }
    }
  }

  // Open Discord if selected
  if (joinDiscord) {
    logger.newline();
    logger.box(
      `Join our Discord community for:\n` +
      `  • Team formation\n` +
      `  • Technical support\n` +
      `  • Announcements\n` +
      `  • Networking\n\n` +
      chalk.cyan.underline(DISCORD_URL),
      'Discord Community'
    );
  }

  return {
    projectName,
    teamName: teamName || undefined,
    track,
    tools,
    mcpEnabled: enableMcp,
    discordLinked: joinDiscord,
    initialized: true,
    createdAt: new Date().toISOString()
  };
}

async function runNonInteractive(options: InitOptions): Promise<HackathonConfig> {
  const rawProjectName = options.project || getDefaultProjectName();
  const projectName = sanitizeProjectName(rawProjectName);
  const isQuiet = options.quiet || options.json;

  const tools: ToolSelection = {
    // AI Assistants
    claudeCode: options.tools?.includes('claudeCode') || false,
    geminiCli: options.tools?.includes('geminiCli') || false,
    // Orchestration
    claudeFlow: options.tools?.includes('claudeFlow') || false,
    agenticFlow: options.tools?.includes('agenticFlow') || false,
    flowNexus: options.tools?.includes('flowNexus') || false,
    adk: options.tools?.includes('adk') || false,
    // Cloud Platform
    googleCloudCli: options.tools?.includes('googleCloudCli') || false,
    vertexAi: options.tools?.includes('vertexAi') || false,
    // Databases
    ruvector: options.tools?.includes('ruvector') || false,
    agentDb: options.tools?.includes('agentDb') || false,
    // Synthesis
    agenticSynth: options.tools?.includes('agenticSynth') || false,
    strangeLoops: options.tools?.includes('strangeLoops') || false,
    sparc: options.tools?.includes('sparc') || false,
    // Python Frameworks
    lionpride: options.tools?.includes('lionpride') || false,
    agenticFramework: options.tools?.includes('agenticFramework') || false,
    openaiAgents: options.tools?.includes('openaiAgents') || false
  };

  // Install selected tools (skip in quiet mode unless explicitly requested)
  if (options.tools && options.tools.length > 0 && !isQuiet) {
    for (const toolName of options.tools) {
      const tool = AVAILABLE_TOOLS.find(t => t.name === toolName);
      if (tool) {
        await installTool(tool);
      }
    }
  }

  return {
    projectName,
    teamName: options.team,
    track: options.track as HackathonTrack | undefined,
    tools,
    mcpEnabled: options.mcp || false,
    discordLinked: false,
    initialized: true,
    createdAt: new Date().toISOString()
  };
}

function showSummary(config: HackathonConfig): void {
  logger.newline();
  logger.divider();
  logger.box(
    `${chalk.bold('Project:')} ${config.projectName}\n` +
    (config.teamName ? `${chalk.bold('Team:')} ${config.teamName}\n` : '') +
    (config.track ? `${chalk.bold('Track:')} ${TRACKS[config.track].name}\n` : '') +
    `${chalk.bold('MCP:')} ${config.mcpEnabled ? 'Enabled' : 'Disabled'}\n` +
    `\n${chalk.bold('Installed Tools:')}\n` +
    Object.entries(config.tools)
      .filter(([_, enabled]) => enabled)
      .map(([name]) => {
        const tool = AVAILABLE_TOOLS.find(t => t.name === name);
        return `  • ${tool?.displayName || name}`;
      })
      .join('\n') || '  • None',
    `${HACKATHON_NAME} - Setup Complete`
  );

  logger.newline();
  logger.info('Next steps:');
  logger.list([
    'Start building your project',
    config.mcpEnabled ? 'Run `hackathon mcp` to start the MCP server' : '',
    `Visit ${WEBSITE_URL} for resources`,
    `Join ${DISCORD_URL} for support`
  ].filter(Boolean));
  logger.newline();
}
