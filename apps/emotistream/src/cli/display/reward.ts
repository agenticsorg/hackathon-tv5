/**
 * EmotiStream CLI - Reward Display
 *
 * Visualization of reward calculation and Q-value updates.
 */

import chalk from 'chalk';
import { FeedbackResponse, Recommendation, EmotionalState, DesiredState } from '../../types/index.js';

/**
 * Display reward update and Q-value change
 */
export function displayRewardUpdate(
  response: FeedbackResponse,
  content: Recommendation,
  stateBefore: EmotionalState,
  stateAfter: EmotionalState,
  desired: DesiredState
): void {
  console.log(chalk.gray('\n┌' + '─'.repeat(68) + '┐'));
  console.log(chalk.bold('  🎯 Reinforcement Learning Update:\n'));

  // Content info
  console.log(chalk.white(`   Content: ${chalk.cyan.bold(content.title)}`));
  console.log(chalk.white(`   Type: ${content.isExploration ? chalk.yellow('Exploration') : chalk.green('Exploitation')}`));

  console.log(chalk.gray('\n   ' + '─'.repeat(64)));

  // Emotional journey
  console.log(chalk.white('\n   📊 Emotional Journey:'));

  const beforeBar = createEmotionBar(stateBefore);
  const afterBar = createEmotionBar(stateAfter);
  const desiredBar = createTargetBar(desired);

  console.log(chalk.gray('   Before:  ') + beforeBar);
  console.log(chalk.gray('   After:   ') + afterBar);
  console.log(chalk.gray('   Target:  ') + desiredBar);

  console.log(chalk.gray('\n   ' + '─'.repeat(64)));

  // Reward calculation
  const rewardColor = getRewardColor(response.reward);
  const rewardBar = createRewardBar(response.reward);

  console.log(chalk.white('\n   💰 Reward Calculation:'));
  console.log(`   ${rewardBar} ${rewardColor.bold(response.reward.toFixed(3))}`);
  console.log(chalk.gray(`   ${getRewardMessage(response.reward)}`));

  console.log(chalk.gray('\n   ' + '─'.repeat(64)));

  // Q-value update
  const qDelta = response.newQValue - content.qValue;
  const qDeltaColor = qDelta >= 0 ? chalk.green : chalk.red;
  const qDeltaSign = qDelta >= 0 ? '+' : '';

  console.log(chalk.white('\n   📈 Q-Value Update:'));
  console.log(chalk.gray('   Old Q-value: ') + formatQValue(content.qValue));
  console.log(chalk.gray('   New Q-value: ') + formatQValue(response.newQValue));
  console.log(chalk.gray('   Change:      ') + qDeltaColor.bold(`${qDeltaSign}${qDelta.toFixed(4)}`));

  if (response.policyUpdated) {
    console.log(chalk.green('\n   ✓ Policy successfully updated'));
  } else {
    console.log(chalk.yellow('\n   ⚠ Policy not updated (error occurred)'));
  }

  console.log(chalk.gray('\n└' + '─'.repeat(68) + '┘'));
}

/**
 * Create emotion state visualization bar
 */
function createEmotionBar(state: EmotionalState): string {
  const valence = state.valence.toFixed(2).padStart(5);
  const arousal = state.arousal.toFixed(2).padStart(5);
  const stress = state.stressLevel.toFixed(2).padStart(4);

  return `V:${chalk.cyan(valence)} A:${chalk.yellow(arousal)} S:${chalk.red(stress)} ${getEmotionEmoji(state.primaryEmotion)}`;
}

/**
 * Create target state visualization bar
 */
function createTargetBar(desired: DesiredState): string {
  const valence = desired.targetValence.toFixed(2).padStart(5);
  const arousal = desired.targetArousal.toFixed(2).padStart(5);
  const stress = desired.targetStress.toFixed(2).padStart(4);

  return `V:${chalk.green(valence)} A:${chalk.blue(arousal)} S:${chalk.cyan(stress)} 🎯`;
}

/**
 * Create reward visualization bar
 */
function createRewardBar(reward: number): string {
  const normalized = (reward + 1) / 2; // Map -1..1 to 0..1
  const width = 20;
  const filledWidth = Math.round(normalized * width);

  const color = getRewardColor(reward);
  const filled = color('█'.repeat(filledWidth));
  const empty = chalk.gray('░'.repeat(width - filledWidth));

  return filled + empty;
}

/**
 * Get reward color based on value
 */
function getRewardColor(reward: number): typeof chalk {
  if (reward > 0.7) return chalk.green;
  if (reward > 0.4) return chalk.yellow;
  if (reward > 0) return chalk.white;
  if (reward > -0.3) return chalk.gray;
  return chalk.red;
}

/**
 * Get reward message
 */
function getRewardMessage(reward: number): string {
  if (reward > 0.7) return 'Excellent match! System learning strongly.';
  if (reward > 0.4) return 'Good recommendation. Positive reinforcement.';
  if (reward > 0.1) return 'Moderate match. System adjusting.';
  if (reward > -0.2) return 'Neutral outcome. More data needed.';
  return 'Poor match. System learning from mistake.';
}

/**
 * Format Q-value with color
 */
function formatQValue(qValue: number): string {
  if (qValue > 0.7) return chalk.green.bold(qValue.toFixed(4));
  if (qValue > 0.4) return chalk.yellow(qValue.toFixed(4));
  if (qValue > 0.2) return chalk.white(qValue.toFixed(4));
  return chalk.gray(qValue.toFixed(4));
}

/**
 * Get emotion emoji
 */
function getEmotionEmoji(emotion: string): string {
  const emojiMap: Record<string, string> = {
    joy: '😊',
    sadness: '😔',
    anger: '😠',
    fear: '😨',
    surprise: '😲',
    disgust: '🤢',
    trust: '🤗',
    anticipation: '🤔',
    neutral: '😐',
    relaxation: '😌',
    contentment: '😌',
    excitement: '🤩'
  };

  return emojiMap[emotion.toLowerCase()] || '🎭';
}
