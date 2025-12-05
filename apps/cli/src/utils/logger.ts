/**
 * Logger utilities with colored output
 */

import chalk from 'chalk';
import boxen from 'boxen';
import gradient from 'gradient-string';

const agenticsGradient = gradient(['#6366f1', '#8b5cf6', '#a855f7']);

export const logger = {
  banner(text: string): void {
    console.log(agenticsGradient(text));
  },

  info(message: string): void {
    console.log(chalk.blue('ℹ'), message);
  },

  success(message: string): void {
    console.log(chalk.green('✔'), message);
  },

  warning(message: string): void {
    console.log(chalk.yellow('⚠'), message);
  },

  error(message: string): void {
    console.log(chalk.red('✖'), message);
  },

  step(step: number, total: number, message: string): void {
    console.log(chalk.cyan(`[${step}/${total}]`), message);
  },

  box(content: string, title?: string): void {
    console.log(
      boxen(content, {
        padding: 1,
        margin: 1,
        borderStyle: 'round',
        borderColor: 'magenta',
        title: title,
        titleAlignment: 'center'
      })
    );
  },

  divider(): void {
    console.log(chalk.gray('─'.repeat(60)));
  },

  newline(): void {
    console.log();
  },

  link(text: string, url: string): void {
    console.log(chalk.cyan.underline(`${text}: ${url}`));
  },

  list(items: string[]): void {
    items.forEach(item => {
      console.log(chalk.gray('  •'), item);
    });
  },

  table(data: Record<string, string>): void {
    const maxKeyLength = Math.max(...Object.keys(data).map(k => k.length));
    Object.entries(data).forEach(([key, value]) => {
      console.log(
        chalk.gray('  '),
        chalk.white(key.padEnd(maxKeyLength)),
        chalk.gray(':'),
        chalk.cyan(value)
      );
    });
  }
};
