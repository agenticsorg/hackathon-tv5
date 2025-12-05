/**
 * Info command - Display hackathon information and resources
 */

import chalk from 'chalk';
import {
  HACKATHON_NAME,
  HACKATHON_TAGLINE,
  HACKATHON_SPONSOR,
  HACKATHON_DESCRIPTION,
  TRACKS,
  DISCORD_URL,
  WEBSITE_URL,
  GITHUB_URL,
  BANNER
} from '../constants.js';
import { logger } from '../utils/index.js';

interface InfoOptions {
  json?: boolean;
  quiet?: boolean;
}

export async function infoCommand(options: InfoOptions = {}): Promise<void> {
  // JSON output
  if (options.json) {
    console.log(JSON.stringify({
      success: true,
      hackathon: {
        name: HACKATHON_NAME,
        tagline: HACKATHON_TAGLINE,
        sponsor: HACKATHON_SPONSOR,
        description: HACKATHON_DESCRIPTION
      },
      tracks: Object.entries(TRACKS).map(([id, { name, description }]) => ({
        id,
        name,
        description
      })),
      technologies: [
        'Google Gemini 2.5 Pro (1M token context)',
        'Google Agent Development Kit (ADK)',
        'Vertex AI & Google Cloud Platform',
        'Claude Code & Claude Flow',
        'Multi-agent orchestration systems'
      ],
      resources: {
        website: WEBSITE_URL,
        discord: DISCORD_URL,
        github: GITHUB_URL,
        adkDocs: 'https://google.github.io/adk-docs/',
        vertexAiDocs: 'https://cloud.google.com/vertex-ai/docs',
        claudeDocs: 'https://docs.anthropic.com'
      }
    }));
    return;
  }

  logger.banner(BANNER);
  logger.newline();

  // Hackathon overview
  logger.box(
    `${chalk.bold(HACKATHON_NAME)}\n` +
    `${HACKATHON_TAGLINE}\n\n` +
    `${chalk.gray('Supported by Google Cloud')}\n\n` +
    `Every night, millions spend up to 45 minutes deciding what to watch —\n` +
    `billions of hours lost every day. Not from lack of content, but from\n` +
    `fragmentation. Join us to build the future of agentic AI systems.`,
    'About the Hackathon'
  );

  // Tracks
  console.log(chalk.bold.cyan('\n  Hackathon Tracks:\n'));
  Object.entries(TRACKS).forEach(([key, { name, description }]) => {
    console.log(`  ${chalk.bold.magenta('●')} ${chalk.bold(name)}`);
    console.log(`    ${chalk.gray(description)}\n`);
  });

  // What you'll build
  logger.box(
    `${chalk.bold('Technologies:')}\n` +
    `  • Google Gemini 2.5 Pro (1M token context)\n` +
    `  • Google Agent Development Kit (ADK)\n` +
    `  • Vertex AI & Google Cloud Platform\n` +
    `  • Claude Code & Claude Flow\n` +
    `  • Multi-agent orchestration systems\n\n` +
    `${chalk.bold('Project Types:')}\n` +
    `  • Content discovery & recommendation agents\n` +
    `  • Multi-agent collaboration systems\n` +
    `  • Agentic workflow automation\n` +
    `  • Open innovation solutions`,
    'What You\'ll Build'
  );

  // Resources
  console.log(chalk.bold.cyan('\n  Resources:\n'));
  logger.table({
    'Website': WEBSITE_URL,
    'Discord': DISCORD_URL,
    'GitHub': GITHUB_URL,
    'Google ADK Docs': 'https://google.github.io/adk-docs/',
    'Vertex AI Docs': 'https://cloud.google.com/vertex-ai/docs',
    'Claude Docs': 'https://docs.anthropic.com'
  });

  // Quick start
  logger.box(
    `${chalk.bold('1.')} Initialize your project:\n` +
    `   ${chalk.cyan('npx agentics-hackathon init')}\n\n` +
    `${chalk.bold('2.')} Install recommended tools:\n` +
    `   ${chalk.cyan('npx agentics-hackathon tools --install claudeFlow geminiCli adk')}\n\n` +
    `${chalk.bold('3.')} Join the community:\n` +
    `   ${chalk.cyan(DISCORD_URL)}\n\n` +
    `${chalk.bold('4.')} Start building!`,
    'Quick Start'
  );

  logger.newline();
}
