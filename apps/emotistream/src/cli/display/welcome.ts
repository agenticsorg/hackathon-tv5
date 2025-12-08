/**
 * EmotiStream CLI - Welcome Screen
 */

import chalk from 'chalk';

/**
 * Display welcome banner with ASCII art
 */
export function displayWelcome(): void {
  const banner = `
${chalk.cyan('╔═══════════════════════════════════════════════════════════════════╗')}
${chalk.cyan('║')}                                                                   ${chalk.cyan('║')}
${chalk.cyan('║')}        ${chalk.magenta.bold('EmotiStream')} ${chalk.white.bold('Nexus')} ${chalk.gray('- AI-Powered Emotional Wellness')}        ${chalk.cyan('║')}
${chalk.cyan('║')}                                                                   ${chalk.cyan('║')}
${chalk.cyan('║')}              ${chalk.yellow('🎬')}  ${chalk.white('Emotion-Aware Content Recommendations')}  ${chalk.yellow('🎬')}            ${chalk.cyan('║')}
${chalk.cyan('║')}                                                                   ${chalk.cyan('║')}
${chalk.cyan('╚═══════════════════════════════════════════════════════════════════╝')}

${chalk.white.bold('Welcome to the EmotiStream Interactive Demo!')}

${chalk.gray('This demonstration showcases how our AI-powered recommendation system')}
${chalk.gray('uses reinforcement learning to match content with your emotional state.')}

${chalk.cyan.bold('How it works:')}
  ${chalk.white('1.')} ${chalk.green('Detect')} your current emotional state
  ${chalk.white('2.')} ${chalk.green('Predict')} your desired emotional target
  ${chalk.white('3.')} ${chalk.green('Recommend')} personalized content using RL policy
  ${chalk.white('4.')} ${chalk.green('Learn')} from your feedback to improve future recommendations

${chalk.yellow.bold('Technology Stack:')}
  ${chalk.white('•')} ${chalk.gray('Google Gemini for emotion detection')}
  ${chalk.white('•')} ${chalk.gray('Q-Learning with ε-greedy exploration')}
  ${chalk.white('•')} ${chalk.gray('Vector similarity search (RuVector)')}
  ${chalk.white('•')} ${chalk.gray('Multi-factor reward calculation')}

${chalk.gray('─'.repeat(70))}
`;

  console.log(banner);
}

/**
 * Display thank you message
 */
export function displayThankYou(): void {
  console.log(`
${chalk.cyan('╔═══════════════════════════════════════════════════════════════════╗')}
${chalk.cyan('║')}                                                                   ${chalk.cyan('║')}
${chalk.cyan('║')}                   ${chalk.magenta.bold('Thank You for Trying')}                           ${chalk.cyan('║')}
${chalk.cyan('║')}                      ${chalk.white.bold('EmotiStream Nexus!')}                           ${chalk.cyan('║')}
${chalk.cyan('║')}                                                                   ${chalk.cyan('║')}
${chalk.cyan('║')}          ${chalk.yellow('🌟')}  ${chalk.gray('Your feedback helps us improve')}  ${chalk.yellow('🌟')}              ${chalk.cyan('║')}
${chalk.cyan('║')}                                                                   ${chalk.cyan('║')}
${chalk.cyan('╚═══════════════════════════════════════════════════════════════════╝')}
`);
}
