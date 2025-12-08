#!/usr/bin/env node

/**
 * EmotiStream CLI Demo - Entry Point
 *
 * Interactive demonstration of the emotion-aware recommendation system.
 */

import { DemoFlow } from './demo.js';
import chalk from 'chalk';

/**
 * Main CLI entry point
 */
async function main(): Promise<void> {
  try {
    const demo = new DemoFlow();
    await demo.run();
    process.exit(0);
  } catch (error) {
    console.error(chalk.red('\n❌ Demo error:'), error);
    console.error(chalk.gray('\nStack trace:'), error instanceof Error ? error.stack : '');
    process.exit(1);
  }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log(chalk.yellow('\n\n👋 Demo interrupted. Thank you for trying EmotiStream!'));
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.log(chalk.yellow('\n\n👋 Demo terminated. Goodbye!'));
  process.exit(0);
});

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
  console.error(chalk.red('\n❌ Unhandled Rejection at:'), promise);
  console.error(chalk.red('Reason:'), reason);
  process.exit(1);
});

// Run the demo
main();
