/**
 * EmotiStream CLI Demo - Main Flow Orchestration
 *
 * Implements the complete emotional recommendation demo loop.
 */

import chalk from 'chalk';
import ora, { Ora } from 'ora';
import { EmotionalState, DesiredState, Recommendation, FeedbackRequest, FeedbackResponse, EmotionalExperience } from '../types/index.js';
import { displayWelcome } from './display/welcome.js';
import { displayEmotionAnalysis, displayDesiredState } from './display/emotion.js';
import { displayRecommendations } from './display/recommendations.js';
import { displayRewardUpdate } from './display/reward.js';
import { displayLearningProgress, displayFinalSummary } from './display/learning.js';
import {
  promptEmotionalInput,
  promptContentSelection,
  promptPostViewingFeedback,
  promptContinue,
  waitForKeypress
} from './prompts.js';
import { MockEmotionDetector } from './mock/emotion-detector.js';
import { MockRecommendationEngine } from './mock/recommendation-engine.js';
import { MockFeedbackProcessor } from './mock/feedback-processor.js';

const DEFAULT_USER_ID = 'demo-user-001';
const MAX_ITERATIONS = 3;

/**
 * Main demo flow orchestrator
 */
export class DemoFlow {
  private userId: string;
  private emotionDetector: MockEmotionDetector;
  private recommendationEngine: MockRecommendationEngine;
  private feedbackProcessor: MockFeedbackProcessor;
  private experiences: EmotionalExperience[] = [];

  constructor() {
    this.userId = DEFAULT_USER_ID;
    this.emotionDetector = new MockEmotionDetector();
    this.recommendationEngine = new MockRecommendationEngine();
    this.feedbackProcessor = new MockFeedbackProcessor();
  }

  /**
   * Run the complete demo flow
   */
  async run(): Promise<void> {
    // Clear terminal and show welcome
    console.clear();
    displayWelcome();
    await waitForKeypress('Press ENTER to start the demonstration...');

    // Main demo loop
    for (let iteration = 1; iteration <= MAX_ITERATIONS; iteration++) {
      console.log(chalk.gray('\n' + '━'.repeat(70) + '\n'));
      console.log(chalk.cyan.bold(`🎬 Session ${iteration} of ${MAX_ITERATIONS}\n`));

      await this.runIteration(iteration);

      // Ask to continue
      if (iteration < MAX_ITERATIONS) {
        const shouldContinue = await promptContinue();
        if (!shouldContinue) {
          console.log(chalk.yellow('\n👋 Thanks for trying EmotiStream! Goodbye!'));
          break;
        }
      }
    }

    // Show final summary
    await this.showFinalSummary();
  }

  /**
   * Run a single iteration of the demo
   */
  private async runIteration(iteration: number): Promise<void> {
    // Step 1: Emotional State Detection
    console.log(chalk.cyan.bold('═══ Step 1: Emotional State Detection ═══\n'));
    const emotionalText = await promptEmotionalInput(iteration);

    const spinner1 = ora('Analyzing your emotional state...').start();
    await this.sleep(800);
    const emotionalState = await this.emotionDetector.analyze(emotionalText);
    spinner1.succeed(chalk.green('✓ Emotional state detected'));

    displayEmotionAnalysis(emotionalState);
    await waitForKeypress();

    // Step 2: Desired State Prediction
    console.log(chalk.cyan.bold('\n═══ Step 2: Predicting Desired State ═══\n'));
    const spinner2 = ora('Calculating optimal emotional trajectory...').start();
    await this.sleep(600);
    const desiredState = this.emotionDetector.predictDesiredState(emotionalState);
    spinner2.succeed(chalk.green('✓ Desired state predicted'));

    displayDesiredState(desiredState);
    await waitForKeypress();

    // Step 3: Generate Recommendations
    console.log(chalk.cyan.bold('\n═══ Step 3: AI-Powered Recommendations ═══\n'));
    const spinner3 = ora('Generating personalized recommendations...').start();
    await this.sleep(700);
    const recommendations = await this.recommendationEngine.getRecommendations(
      emotionalState,
      desiredState,
      this.userId,
      5
    );
    spinner3.succeed(chalk.green('✓ Recommendations generated'));

    displayRecommendations(recommendations, iteration);

    // Step 4: Content Selection
    const selectedContentId = await promptContentSelection(recommendations);
    const selectedContent = recommendations.find(r => r.contentId === selectedContentId)!;

    // Step 5: Simulate Viewing
    console.log(chalk.cyan.bold('\n═══ Step 4: Viewing Experience ═══\n'));
    await this.simulateViewing(selectedContent);

    // Step 6: Post-Viewing Feedback
    console.log(chalk.cyan.bold('\n═══ Step 5: Feedback & Learning ═══\n'));
    const feedbackInput = await promptPostViewingFeedback();

    // Step 7: Process Feedback
    const spinner4 = ora('Processing feedback and updating RL policy...').start();
    await this.sleep(500);

    const postViewingState = this.emotionDetector.analyzePostViewing(feedbackInput);

    const feedbackRequest: FeedbackRequest = {
      userId: this.userId,
      contentId: selectedContent.contentId,
      actualPostState: postViewingState,
      watchDuration: 30, // Mock duration
      completed: true,
      explicitRating: feedbackInput.rating
    };

    const feedbackResponse = await this.feedbackProcessor.processFeedback(
      feedbackRequest,
      emotionalState,
      desiredState
    );

    spinner4.succeed(chalk.green('✓ Policy updated successfully'));

    displayRewardUpdate(feedbackResponse, selectedContent, emotionalState, postViewingState, desiredState);

    // Step 8: Learning Progress
    console.log(chalk.cyan.bold('\n═══ Step 6: Learning Progress ═══\n'));
    await displayLearningProgress(this.userId, iteration, feedbackResponse.learningProgress);
    await waitForKeypress();

    // Store experience
    this.experiences.push({
      userId: this.userId,
      timestamp: Date.now(),
      stateBefore: emotionalState,
      action: selectedContent.contentId,
      stateAfter: postViewingState,
      reward: feedbackResponse.reward,
      desiredState
    });
  }

  /**
   * Simulate content viewing with progress bar
   */
  private async simulateViewing(content: Recommendation): Promise<void> {
    console.log(chalk.white(`📺 Now watching: ${chalk.bold(content.title)}\n`));

    const spinner = ora('').start();
    const steps = 20;

    for (let i = 0; i <= steps; i++) {
      const percent = (i / steps) * 100;
      const filled = '█'.repeat(i);
      const empty = '░'.repeat(steps - i);

      spinner.text = `${chalk.cyan(filled)}${chalk.gray(empty)} ${percent.toFixed(0)}%`;
      await this.sleep(100);
    }

    spinner.succeed(chalk.green('✓ Viewing complete'));
    console.log(chalk.gray('Duration: 30 minutes\n'));
    await this.sleep(500);
  }

  /**
   * Show final summary of the demo session
   */
  private async showFinalSummary(): Promise<void> {
    console.log(chalk.gray('\n' + '━'.repeat(70) + '\n'));
    displayFinalSummary(this.experiences);

    console.log(chalk.cyan.bold('\n🎓 Key Takeaways:\n'));
    console.log(chalk.white('  ✓ Emotion detection analyzes your current emotional state'));
    console.log(chalk.white('  ✓ RL policy learns optimal content recommendations'));
    console.log(chalk.white('  ✓ Feedback updates Q-values for continuous improvement'));
    console.log(chalk.white('  ✓ System balances exploration vs exploitation'));
    console.log(chalk.white('  ✓ Personalized recommendations improve over time\n'));

    console.log(chalk.magenta.bold('Thank you for trying EmotiStream! 🎬✨\n'));
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
