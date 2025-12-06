/**
 * EmotiStream CLI - Emotion Display
 *
 * Visual representation of emotional states.
 */

import chalk from 'chalk';
import { EmotionalState, DesiredState } from '../../types/index.js';

/**
 * Display detected emotional state
 */
export function displayEmotionAnalysis(state: EmotionalState): void {
  console.log(chalk.gray('\n┌' + '─'.repeat(68) + '┐'));
  console.log(chalk.bold('  📊 Emotional State Analysis:\n'));

  // Valence
  const valenceBar = createProgressBar(state.valence, -1, 1, 20);
  const valenceColor = state.valence >= 0 ? chalk.green : chalk.red;
  const valenceLabel = getValenceLabel(state.valence);

  console.log(
    `   ${chalk.white('Valence:')}  ${valenceColor(valenceBar)} ${state.valence.toFixed(2).padStart(5)} ${chalk.gray(`(${valenceLabel})`)}`
  );

  // Arousal
  const arousalBar = createProgressBar(state.arousal, -1, 1, 20);
  const arousalColor = state.arousal >= 0 ? chalk.yellow : chalk.blue;
  const arousalLabel = getArousalLabel(state.arousal);

  console.log(
    `   ${chalk.white('Arousal:')}  ${arousalColor(arousalBar)} ${state.arousal.toFixed(2).padStart(5)} ${chalk.gray(`(${arousalLabel})`)}`
  );

  // Stress
  const stressBar = createProgressBar(state.stressLevel, 0, 1, 20);
  const stressColor = getStressColor(state.stressLevel);
  const stressLabel = getStressLabel(state.stressLevel);

  console.log(
    `   ${chalk.white('Stress: ')}  ${stressColor(stressBar)} ${state.stressLevel.toFixed(2).padStart(5)} ${chalk.gray(`(${stressLabel})`)}`
  );

  // Primary emotion
  const emoji = getEmotionEmoji(state.primaryEmotion);
  const confidence = (state.confidence * 100).toFixed(0);

  console.log(
    chalk.gray('\n   ─'.repeat(34))
  );
  console.log(
    `\n   ${chalk.white('Primary:')}  ${emoji}  ${chalk.bold.cyan(state.primaryEmotion.toUpperCase())} ${chalk.gray(`(${confidence}% confidence)`)}`
  );

  console.log(chalk.gray('\n└' + '─'.repeat(68) + '┘'));
}

/**
 * Display predicted desired state
 */
export function displayDesiredState(desired: DesiredState): void {
  console.log(chalk.gray('\n┌' + '─'.repeat(68) + '┐'));
  console.log(chalk.bold('  🎯 Predicted Desired State:\n'));

  const targetValenceBar = createProgressBar(desired.targetValence, -1, 1, 20);
  const targetArousalBar = createProgressBar(desired.targetArousal, -1, 1, 20);
  const targetStressBar = createProgressBar(desired.targetStress, 0, 1, 20);

  console.log(
    `   ${chalk.white('Target Valence:')}  ${chalk.green(targetValenceBar)} ${desired.targetValence.toFixed(2).padStart(5)}`
  );
  console.log(
    `   ${chalk.white('Target Arousal:')}  ${chalk.blue(targetArousalBar)} ${desired.targetArousal.toFixed(2).padStart(5)}`
  );
  console.log(
    `   ${chalk.white('Target Stress: ')}  ${chalk.cyan(targetStressBar)} ${desired.targetStress.toFixed(2).padStart(5)}`
  );

  const intensityColor = desired.intensity === 'significant' ? chalk.red :
                         desired.intensity === 'moderate' ? chalk.yellow :
                         chalk.green;

  console.log(
    `\n   ${chalk.white('Intensity:')} ${intensityColor.bold(desired.intensity.toUpperCase())}`
  );
  console.log(
    `   ${chalk.white('Reasoning:')} ${chalk.gray(desired.reasoning)}`
  );

  console.log(chalk.gray('\n└' + '─'.repeat(68) + '┘'));
}

/**
 * Create ASCII progress bar
 */
function createProgressBar(
  value: number,
  min: number,
  max: number,
  width: number
): string {
  const normalized = (value - min) / (max - min);
  const clamped = Math.max(0, Math.min(1, normalized));
  const filledWidth = Math.round(clamped * width);

  const filled = '█'.repeat(filledWidth);
  const empty = '░'.repeat(width - filledWidth);

  return filled + empty;
}

/**
 * Get valence label
 */
function getValenceLabel(valence: number): string {
  if (valence > 0.6) return 'very positive';
  if (valence > 0.2) return 'positive';
  if (valence > -0.2) return 'neutral';
  if (valence > -0.6) return 'negative';
  return 'very negative';
}

/**
 * Get arousal label
 */
function getArousalLabel(arousal: number): string {
  if (arousal > 0.6) return 'very excited';
  if (arousal > 0.2) return 'excited';
  if (arousal > -0.2) return 'neutral';
  if (arousal > -0.6) return 'calm';
  return 'very calm';
}

/**
 * Get stress label
 */
function getStressLabel(stress: number): string {
  if (stress > 0.8) return 'very high';
  if (stress > 0.6) return 'high';
  if (stress > 0.4) return 'moderate';
  if (stress > 0.2) return 'low';
  return 'minimal';
}

/**
 * Get stress color
 */
function getStressColor(stress: number): typeof chalk {
  if (stress > 0.8) return chalk.red;
  if (stress > 0.6) return chalk.hex('#FFA500'); // Orange
  if (stress > 0.4) return chalk.yellow;
  return chalk.green;
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
    stress: '😰',
    anxiety: '😟',
    relaxation: '😌',
    contentment: '😌',
    excitement: '🤩'
  };

  return emojiMap[emotion.toLowerCase()] || '🎭';
}
