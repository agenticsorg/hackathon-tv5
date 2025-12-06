/**
 * EmotiStream CLI - Recommendations Display
 *
 * Table visualization of personalized recommendations.
 */

import chalk from 'chalk';
import Table from 'cli-table3';
import { Recommendation } from '../../types/index.js';

/**
 * Display recommendations as formatted table
 */
export function displayRecommendations(recommendations: Recommendation[], iteration: number): void {
  console.log(chalk.gray('\n┌' + '─'.repeat(68) + '┐'));
  console.log(chalk.bold(`  🎬 Top ${recommendations.length} Personalized Recommendations:\n`));

  const table = new Table({
    head: [
      chalk.white.bold('#'),
      chalk.white.bold('Title'),
      chalk.white.bold('Q-Value'),
      chalk.white.bold('Similarity'),
      chalk.white.bold('Type')
    ],
    colWidths: [4, 30, 10, 12, 12],
    style: {
      head: [],
      border: ['gray']
    },
    chars: {
      'top': '─',
      'top-mid': '┬',
      'top-left': '┌',
      'top-right': '┐',
      'bottom': '─',
      'bottom-mid': '┴',
      'bottom-left': '└',
      'bottom-right': '┘',
      'left': '│',
      'left-mid': '├',
      'mid': '─',
      'mid-mid': '┼',
      'right': '│',
      'right-mid': '┤',
      'middle': '│'
    }
  });

  recommendations.forEach((rec, index) => {
    const rank = (index + 1).toString();
    const title = truncate(rec.title, 28);
    const qValue = formatQValue(rec.qValue);
    const similarity = formatSimilarity(rec.similarityScore);
    const type = rec.isExploration ? chalk.yellow('🔍 Explore') : chalk.green('✓ Exploit');

    table.push([rank, title, qValue, similarity, type]);
  });

  console.log(table.toString());

  // Show legend
  console.log(chalk.gray('\n  Legend:'));
  console.log(chalk.gray('  • Q-Value: Learned value from past experiences'));
  console.log(chalk.gray('  • Similarity: Emotional profile match'));
  console.log(chalk.gray('  • Explore: Trying new content for learning'));
  console.log(chalk.gray('  • Exploit: Using learned knowledge'));

  console.log(chalk.gray('\n└' + '─'.repeat(68) + '┘'));
}

/**
 * Format Q-value with color coding
 */
function formatQValue(qValue: number): string {
  const formatted = qValue.toFixed(3);

  if (qValue > 0.7) return chalk.green.bold(formatted);
  if (qValue > 0.4) return chalk.yellow(formatted);
  if (qValue > 0.2) return chalk.white(formatted);
  return chalk.gray(formatted);
}

/**
 * Format similarity score with color coding
 */
function formatSimilarity(score: number): string {
  const formatted = score.toFixed(3);

  if (score > 0.8) return chalk.green(formatted);
  if (score > 0.6) return chalk.yellow(formatted);
  return chalk.white(formatted);
}

/**
 * Truncate string to max length
 */
function truncate(str: string, maxLength: number): string {
  if (str.length <= maxLength) return str;
  return str.substring(0, maxLength - 3) + '...';
}
