/**
 * EmotiStream CLI - Inquirer Prompts
 *
 * Interactive prompts for user input throughout the demo.
 */

import inquirer from 'inquirer';
import chalk from 'chalk';
import { Recommendation } from '../types/index.js';

/**
 * Emotional input examples for different iterations
 */
const EMOTIONAL_EXAMPLES = [
  "I'm feeling stressed and overwhelmed from work today",
  "I feel a bit lonely and could use some uplifting content",
  "I'm feeling anxious and need something to calm my nerves"
];

/**
 * Prompt for emotional state input
 */
export async function promptEmotionalInput(iteration: number): Promise<string> {
  console.log(chalk.gray('Tell us how you\'re feeling right now. Be as descriptive as you like.\n'));

  const exampleIndex = (iteration - 1) % EMOTIONAL_EXAMPLES.length;
  const example = chalk.gray(`Example: "${EMOTIONAL_EXAMPLES[exampleIndex]}"`);

  const { emotionalText } = await inquirer.prompt([
    {
      type: 'input',
      name: 'emotionalText',
      message: 'How are you feeling?',
      default: EMOTIONAL_EXAMPLES[exampleIndex],
      validate: (input: string) => {
        if (!input || input.trim().length < 10) {
          return 'Please provide at least 10 characters describing your emotions';
        }
        return true;
      }
    }
  ]);

  return emotionalText;
}

/**
 * Prompt for content selection from recommendations
 */
export async function promptContentSelection(recommendations: Recommendation[]): Promise<string> {
  console.log(chalk.cyan.bold('\n📌 Select content to watch:\n'));

  const choices = recommendations.map((rec, index) => ({
    name: `${index + 1}. ${rec.title} ${chalk.gray(`(Q: ${rec.qValue.toFixed(3)}, Sim: ${rec.similarityScore.toFixed(3)})`)}`,
    value: rec.contentId,
    short: rec.title
  }));

  const { selectedContentId } = await inquirer.prompt([
    {
      type: 'list',
      name: 'selectedContentId',
      message: 'Choose content:',
      choices,
      pageSize: 10
    }
  ]);

  return selectedContentId;
}

/**
 * Post-viewing feedback input types
 */
export interface PostViewingFeedback {
  text?: string;
  rating?: number;
  emoji?: string;
}

/**
 * Prompt for post-viewing feedback
 */
export async function promptPostViewingFeedback(): Promise<PostViewingFeedback> {
  console.log(chalk.gray('Now that you\'ve finished watching, how do you feel?\n'));

  const { feedbackType } = await inquirer.prompt([
    {
      type: 'list',
      name: 'feedbackType',
      message: 'Choose feedback method:',
      choices: [
        { name: '💬 Text feedback (most accurate)', value: 'text' },
        { name: '⭐ Star rating (1-5)', value: 'rating' },
        { name: '😊 Emoji feedback (quick)', value: 'emoji' }
      ]
    }
  ]);

  if (feedbackType === 'text') {
    const { text } = await inquirer.prompt([
      {
        type: 'input',
        name: 'text',
        message: 'Describe how you feel now:',
        default: 'I feel much more relaxed and calm now',
        validate: (input: string) => {
          if (!input || input.trim().length < 5) {
            return 'Please provide at least 5 characters';
          }
          return true;
        }
      }
    ]);
    return { text };
  } else if (feedbackType === 'rating') {
    const { rating } = await inquirer.prompt([
      {
        type: 'list',
        name: 'rating',
        message: 'Rate your experience:',
        choices: [
          { name: '⭐⭐⭐⭐⭐ (5) - Excellent', value: 5 },
          { name: '⭐⭐⭐⭐ (4) - Good', value: 4 },
          { name: '⭐⭐⭐ (3) - Okay', value: 3 },
          { name: '⭐⭐ (2) - Poor', value: 2 },
          { name: '⭐ (1) - Very Poor', value: 1 }
        ]
      }
    ]);
    return { rating };
  } else {
    const { emoji } = await inquirer.prompt([
      {
        type: 'list',
        name: 'emoji',
        message: 'How do you feel?',
        choices: [
          { name: '😊 Happy', value: '😊' },
          { name: '😌 Relaxed', value: '😌' },
          { name: '😐 Neutral', value: '😐' },
          { name: '😢 Sad', value: '😢' },
          { name: '😡 Angry', value: '😡' },
          { name: '😴 Sleepy', value: '😴' }
        ]
      }
    ]);
    return { emoji };
  }
}

/**
 * Prompt to continue or exit
 */
export async function promptContinue(): Promise<boolean> {
  const { shouldContinue } = await inquirer.prompt([
    {
      type: 'confirm',
      name: 'shouldContinue',
      message: chalk.cyan('Would you like to continue with another recommendation?'),
      default: true
    }
  ]);

  return shouldContinue;
}

/**
 * Wait for user to press ENTER
 */
export async function waitForKeypress(message: string = 'Press ENTER to continue...'): Promise<void> {
  await inquirer.prompt([
    {
      type: 'input',
      name: 'continue',
      message: chalk.gray(message),
      prefix: '',
      transformer: () => '' // Hide input
    }
  ]);
}
