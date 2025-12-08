/**
 * EmotiStream CLI - Learning Progress Display
 *
 * Visualization of learning metrics and convergence.
 */

import chalk from 'chalk';
import { LearningProgress, EmotionalExperience } from '../../types/index.js';

/**
 * Display current learning progress
 */
export async function displayLearningProgress(
  userId: string,
  iteration: number,
  progress: LearningProgress
): Promise<void> {
  console.log(chalk.gray('┌' + '─'.repeat(68) + '┐'));
  console.log(chalk.bold('  📚 Learning Progress:\n'));

  // Experience count
  console.log(chalk.white(`   Total Experiences: ${chalk.cyan.bold(progress.totalExperiences.toString())}`));

  // Average reward
  const avgRewardColor = progress.avgReward > 0.5 ? chalk.green :
                        progress.avgReward > 0 ? chalk.yellow :
                        chalk.red;
  const avgRewardBar = createProgressBar(progress.avgReward, -1, 1, 20);

  console.log(chalk.white(`   Average Reward:    ${avgRewardColor(avgRewardBar)} ${avgRewardColor.bold(progress.avgReward.toFixed(3))}`));

  // Exploration rate
  const exploreBar = createProgressBar(progress.explorationRate, 0, 1, 20);
  const explorePercent = (progress.explorationRate * 100).toFixed(1);

  console.log(chalk.white(`   Exploration Rate:  ${chalk.yellow(exploreBar)} ${chalk.yellow(explorePercent + '%')}`));

  // Convergence score
  const convergenceBar = createProgressBar(progress.convergenceScore, 0, 1, 20);
  const convergencePercent = (progress.convergenceScore * 100).toFixed(1);
  const convergenceColor = progress.convergenceScore > 0.7 ? chalk.green :
                          progress.convergenceScore > 0.4 ? chalk.yellow :
                          chalk.white;

  console.log(chalk.white(`   Convergence:       ${convergenceColor(convergenceBar)} ${convergenceColor(convergencePercent + '%')}`));

  console.log(chalk.gray('\n   ' + '─'.repeat(64)));

  // Interpretation
  console.log(chalk.white('\n   💡 Interpretation:'));

  if (progress.avgReward > 0.5) {
    console.log(chalk.green('   ✓ System is learning effectively'));
    console.log(chalk.gray('   Recommendations are consistently good'));
  } else if (progress.avgReward > 0) {
    console.log(chalk.yellow('   ⚠ Learning in progress'));
    console.log(chalk.gray('   System needs more experiences to improve'));
  } else {
    console.log(chalk.red('   ⚠ Initial learning phase'));
    console.log(chalk.gray('   Keep providing feedback to train the model'));
  }

  if (progress.explorationRate > 0.2) {
    console.log(chalk.gray('   Actively exploring to find better content'));
  } else {
    console.log(chalk.gray('   Mostly exploiting learned knowledge'));
  }

  console.log(chalk.gray('\n└' + '─'.repeat(68) + '┘'));
}

/**
 * Display final summary of all experiences
 */
export function displayFinalSummary(experiences: EmotionalExperience[]): void {
  console.log(chalk.cyan.bold('📊 Session Summary\n'));

  if (experiences.length === 0) {
    console.log(chalk.gray('No experiences recorded in this session.\n'));
    return;
  }

  // Calculate summary statistics
  const totalReward = experiences.reduce((sum, exp) => sum + exp.reward, 0);
  const avgReward = totalReward / experiences.length;
  const maxReward = Math.max(...experiences.map(exp => exp.reward));
  const minReward = Math.min(...experiences.map(exp => exp.reward));

  console.log(chalk.gray('┌' + '─'.repeat(68) + '┐'));
  console.log(chalk.white('  Total Experiences:  ') + chalk.cyan.bold(experiences.length.toString()));
  console.log(chalk.white('  Average Reward:     ') + formatReward(avgReward));
  console.log(chalk.white('  Best Reward:        ') + formatReward(maxReward));
  console.log(chalk.white('  Worst Reward:       ') + formatReward(minReward));

  console.log(chalk.gray('\n  ' + '─'.repeat(64)));
  console.log(chalk.white('\n  📈 Reward Trend:\n'));

  // Create ASCII chart
  const chartHeight = 5;
  const chartWidth = experiences.length * 3;

  experiences.forEach((exp, index) => {
    const normalized = (exp.reward + 1) / 2; // Map -1..1 to 0..1
    const barHeight = Math.round(normalized * chartHeight);
    const color = exp.reward > 0.5 ? chalk.green :
                 exp.reward > 0 ? chalk.yellow :
                 chalk.red;

    const bar = '▂▃▄▅▆▇█'[Math.min(6, Math.floor(normalized * 7))];
    process.stdout.write(color(`  ${bar}`));
  });

  console.log('\n');

  // Emotional journey
  const valenceDelta = experiences[experiences.length - 1].stateAfter.valence -
                      experiences[0].stateBefore.valence;
  const stressDelta = experiences[experiences.length - 1].stateAfter.stressLevel -
                     experiences[0].stateBefore.stressLevel;

  console.log(chalk.gray('  ' + '─'.repeat(64)));
  console.log(chalk.white('\n  🎭 Emotional Journey:\n'));

  const valenceDeltaColor = valenceDelta >= 0 ? chalk.green : chalk.red;
  const stressDeltaColor = stressDelta <= 0 ? chalk.green : chalk.red;

  console.log(chalk.white('  Valence Change:     ') + valenceDeltaColor(`${valenceDelta > 0 ? '+' : ''}${valenceDelta.toFixed(3)}`));
  console.log(chalk.white('  Stress Change:      ') + stressDeltaColor(`${stressDelta > 0 ? '+' : ''}${stressDelta.toFixed(3)}`));

  if (valenceDelta > 0 && stressDelta < 0) {
    console.log(chalk.green('\n  ✓ Positive emotional improvement!'));
  } else if (valenceDelta > 0 || stressDelta < 0) {
    console.log(chalk.yellow('\n  ⚠ Some emotional improvement'));
  } else {
    console.log(chalk.gray('\n  ℹ Continue using to see improvements'));
  }

  console.log(chalk.gray('\n└' + '─'.repeat(68) + '┘'));
}

/**
 * Create progress bar
 */
function createProgressBar(value: number, min: number, max: number, width: number): string {
  const normalized = (value - min) / (max - min);
  const clamped = Math.max(0, Math.min(1, normalized));
  const filledWidth = Math.round(clamped * width);

  const filled = '█'.repeat(filledWidth);
  const empty = '░'.repeat(width - filledWidth);

  return filled + empty;
}

/**
 * Format reward with color
 */
function formatReward(reward: number): string {
  const formatted = reward.toFixed(3);

  if (reward > 0.7) return chalk.green.bold(formatted);
  if (reward > 0.4) return chalk.yellow(formatted);
  if (reward > 0) return chalk.white(formatted);
  if (reward > -0.3) return chalk.gray(formatted);
  return chalk.red(formatted);
}
