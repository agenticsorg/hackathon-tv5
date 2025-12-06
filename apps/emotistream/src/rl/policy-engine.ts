import { QTable } from './q-table';
import { RewardCalculator } from './reward-calculator';
import { EpsilonGreedyStrategy } from './exploration/epsilon-greedy';
import { UCBCalculator } from './exploration/ucb';
import { ReplayBuffer } from './replay-buffer';
import {
  EmotionalState,
  DesiredState,
  ActionSelection,
  EmotionalExperience,
  PolicyUpdate
} from './types';

export class RLPolicyEngine {
  private readonly learningRate = 0.1;
  private readonly discountFactor = 0.95;
  private readonly ucbCalculator: UCBCalculator;
  private readonly replayBuffer: ReplayBuffer;

  constructor(
    private readonly qTable: QTable,
    private readonly rewardCalculator: RewardCalculator,
    private readonly explorationStrategy: EpsilonGreedyStrategy
  ) {
    this.ucbCalculator = new UCBCalculator(2.0);
    this.replayBuffer = new ReplayBuffer(10000);
  }

  async selectAction(
    userId: string,
    state: EmotionalState,
    desired: DesiredState,
    availableContent: string[]
  ): Promise<ActionSelection> {
    const stateHash = this.hashState(state);

    if (this.explorationStrategy.shouldExplore()) {
      return this.explore(userId, stateHash, availableContent);
    } else {
      return this.exploit(userId, stateHash, availableContent);
    }
  }

  async updatePolicy(userId: string, experience: EmotionalExperience): Promise<PolicyUpdate> {
    const currentStateHash = this.hashState(experience.stateBefore);
    const nextStateHash = this.hashState(experience.stateAfter);

    const entry = await this.qTable.get(currentStateHash, experience.contentId);
    const currentQ = entry?.qValue || 0.0;

    const nextStateActions = await this.qTable.getStateActions(nextStateHash);
    const maxNextQ = nextStateActions.length > 0
      ? Math.max(...nextStateActions.map(e => e.qValue))
      : 0.0;

    const tdTarget = experience.reward + this.discountFactor * maxNextQ;
    const tdError = tdTarget - currentQ;
    const newQ = currentQ + this.learningRate * tdError;

    await this.qTable.updateQValue(currentStateHash, experience.contentId, newQ);

    this.explorationStrategy.decay();

    this.replayBuffer.add(experience);

    const updatedEntry = await this.qTable.get(currentStateHash, experience.contentId);

    return {
      stateHash: currentStateHash,
      contentId: experience.contentId,
      oldQValue: currentQ,
      newQValue: newQ,
      tdError,
      reward: experience.reward,
      visitCount: updatedEntry?.visitCount || 1
    };
  }

  async getQValue(userId: string, stateHash: string, contentId: string): Promise<number> {
    const entry = await this.qTable.get(stateHash, contentId);
    return entry?.qValue || 0.0;
  }

  private async exploit(
    userId: string,
    stateHash: string,
    availableContent: string[]
  ): Promise<ActionSelection> {
    let maxQ = -Infinity;
    let bestContentId = availableContent[0];
    let bestVisitCount = 0;

    for (const contentId of availableContent) {
      const entry = await this.qTable.get(stateHash, contentId);
      const qValue = entry?.qValue || 0.0;

      if (qValue > maxQ) {
        maxQ = qValue;
        bestContentId = contentId;
        bestVisitCount = entry?.visitCount || 0;
      }
    }

    const confidence = bestVisitCount > 0
      ? 1.0 - Math.exp(-bestVisitCount / 10.0)
      : 0.0;

    return {
      contentId: bestContentId,
      qValue: maxQ,
      isExploration: false,
      explorationBonus: 0.0,
      confidence,
      stateHash
    };
  }

  private async explore(
    userId: string,
    stateHash: string,
    availableContent: string[]
  ): Promise<ActionSelection> {
    const stateActions = await this.qTable.getStateActions(stateHash);
    const totalVisits = stateActions.reduce((sum, e) => sum + e.visitCount, 0);

    if (totalVisits === 0) {
      const randomContent = this.explorationStrategy.selectRandom(availableContent);
      return {
        contentId: randomContent,
        qValue: 0.0,
        isExploration: true,
        explorationBonus: Infinity,
        confidence: 0.0,
        stateHash
      };
    }

    let maxUCB = -Infinity;
    let bestContentId = availableContent[0];
    let bestQValue = 0.0;
    let bestBonus = 0.0;
    let bestVisitCount = 0;

    for (const contentId of availableContent) {
      const entry = await this.qTable.get(stateHash, contentId);
      const qValue = entry?.qValue || 0.0;
      const visitCount = entry?.visitCount || 0;

      const ucbValue = this.ucbCalculator.calculate(qValue, visitCount, totalVisits);

      if (ucbValue > maxUCB) {
        maxUCB = ucbValue;
        bestContentId = contentId;
        bestQValue = qValue;
        bestBonus = ucbValue - qValue;
        bestVisitCount = visitCount;
      }
    }

    const confidence = bestVisitCount > 0
      ? 1.0 - Math.exp(-bestVisitCount / 10.0)
      : 0.0;

    return {
      contentId: bestContentId,
      qValue: bestQValue,
      isExploration: true,
      explorationBonus: bestBonus,
      confidence,
      stateHash
    };
  }

  private hashState(state: EmotionalState): string {
    const valenceBucket = Math.floor((state.valence + 1.0) / 0.4);
    const vBucket = Math.max(0, Math.min(4, valenceBucket));

    const arousalBucket = Math.floor((state.arousal + 1.0) / 0.4);
    const aBucket = Math.max(0, Math.min(4, arousalBucket));

    const stressBucket = Math.floor(state.stress / 0.34);
    const sBucket = Math.max(0, Math.min(2, stressBucket));

    return `${vBucket}:${aBucket}:${sBucket}`;
  }
}
