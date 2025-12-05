#!/usr/bin/env node

/**
 * ARW CLI - Enhanced with comprehensive validation and tools
 */

import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import chalk from 'chalk';
import { validateDomain } from '../src/validate.js';
import { convertCommand } from '../src/commands/convert.js';
import { doctorCommand } from '../src/commands/doctor.js';
import { testCompatibilityCommand } from '../src/commands/test-compatibility.js';
import { buildCommand } from '../src/commands/build.js';

yargs(hideBin(process.argv))
  .scriptName('arw')
  .usage('$0 <command> [options]')
  .version('2.0.0')
  .alias('v', 'version')
  .alias('h', 'help')

  // Validate command
  .command(
    'validate <url>',
    'Validate an ARW implementation',
    (yargs) => {
      return yargs
        .positional('url', {
          type: 'string',
          describe: 'Base URL to validate (e.g., https://example.com)',
        })
        .option('platform', {
          type: 'string',
          describe: 'Platform-specific validation (vercel, netlify, cloudflare)',
          choices: ['vercel', 'netlify', 'cloudflare'],
        })
        .option('full', {
          type: 'boolean',
          default: false,
          describe: 'Run full validation including all checks',
        })
        .option('timeout', {
          type: 'number',
          default: 8000,
          describe: 'HTTP timeout in milliseconds',
        })
        .option('json', {
          type: 'boolean',
          default: false,
          describe: 'Output results as JSON',
        });
    },
    async (argv) => {
      const { url, timeout, platform, full, json } = argv;

      try {
        const report = await validateDomain(url, { timeout, platform, full });

        if (json) {
          console.log(JSON.stringify(report, null, 2));
        } else {
          const ok = report.errors.length === 0;
          console.log('\n' + chalk.bold('ARW Validation Report'));
          console.log(chalk.gray('─'.repeat(50)));
          console.log('Domain:', chalk.cyan(url));
          console.log('Profile:', report.profile || 'unknown');
          console.log('Level:', report.level || 'undetected');
          console.log('Checks passed:', chalk.green(report.passed.length));
          console.log('Warnings:', report.warnings?.length ? chalk.yellow(report.warnings.length) : chalk.green('0'));
          console.log('Errors:', report.errors.length ? chalk.red(report.errors.length) : chalk.green('0'));

          if (report.passed.length) {
            console.log('\n' + chalk.green('✓ Passed'));
            report.passed.forEach((p) => console.log('  •', p));
          }

          if (report.warnings?.length) {
            console.log('\n' + chalk.yellow('⚠ Warnings'));
            report.warnings.forEach((w) => console.log('  •', w));
          }

          if (report.errors.length) {
            console.log('\n' + chalk.red('✗ Errors'));
            report.errors.forEach((e) => console.log('  •', e));
            process.exitCode = 1;
          } else {
            console.log('\n' + chalk.green('All checks passed!'));
          }

          console.log('');
        }
      } catch (e) {
        console.error(chalk.red('Validation failed:'), e.message || e);
        process.exitCode = 1;
      }
    }
  )

  // Doctor command
  .command(
    'doctor <url>',
    'Run comprehensive ARW health check',
    (yargs) => {
      return yargs
        .positional('url', {
          type: 'string',
          describe: 'Base URL to check (e.g., https://example.com)',
        })
        .option('platform', {
          type: 'string',
          describe: 'Platform-specific checks (vercel, netlify, cloudflare)',
          choices: ['vercel', 'netlify', 'cloudflare'],
        })
        .option('timeout', {
          type: 'number',
          default: 8000,
          describe: 'HTTP timeout in milliseconds',
        })
        .option('json', {
          type: 'boolean',
          default: false,
          describe: 'Output results as JSON',
        });
    },
    async (argv) => {
      await doctorCommand(argv.url, argv);
    }
  )

  // Test compatibility command
  .command(
    'test-compatibility <url>',
    'Test agent accessibility and compatibility',
    (yargs) => {
      return yargs
        .positional('url', {
          type: 'string',
          describe: 'Base URL to test (e.g., https://example.com)',
        })
        .option('timeout', {
          type: 'number',
          default: 8000,
          describe: 'HTTP timeout in milliseconds',
        })
        .option('json', {
          type: 'boolean',
          default: false,
          describe: 'Output results as JSON',
        });
    },
    async (argv) => {
      await testCompatibilityCommand(argv.url, argv);
    }
  )

  // Convert command
  .command(
    'convert <input> [output]',
    'Convert YAML manifest to JSON',
    (yargs) => {
      return yargs
        .positional('input', {
          type: 'string',
          describe: 'Input YAML file (e.g., llms.txt)',
        })
        .positional('output', {
          type: 'string',
          describe: 'Output JSON file (e.g., llms.json). Omit to print to stdout',
        });
    },
    async (argv) => {
      await convertCommand(argv.input, argv.output);
    }
  )

  // Build command
  .command(
    'build',
    'Generate all ARW files from llms.txt',
    (yargs) => {
      return yargs
        .option('source', {
          type: 'string',
          default: 'public/llms.txt',
          describe: 'Source llms.txt file',
        })
        .option('output-dir', {
          type: 'string',
          default: 'public',
          describe: 'Output directory for generated files',
        })
        .option('base-url', {
          type: 'string',
          default: 'https://example.com',
          describe: 'Base URL for your site',
        });
    },
    async (argv) => {
      await buildCommand(argv);
    }
  )

  .demandCommand(1, 'You must specify a command')
  .strict()
  .help()
  .parse();
