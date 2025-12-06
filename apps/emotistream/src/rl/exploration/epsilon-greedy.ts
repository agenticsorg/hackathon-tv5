export class EpsilonGreedyStrategy {
  private epsilon: number;

  constructor(
    private readonly initialEpsilon: number,
    private readonly minEpsilon: number,
    private readonly decayRate: number
  ) {
    this.epsilon = initialEpsilon;
  }

  shouldExplore(): boolean {
    return Math.random() < this.epsilon;
  }

  selectRandom(actions: string[]): string {
    const randomIndex = Math.floor(Math.random() * actions.length);
    return actions[randomIndex];
  }

  decay(): void {
    this.epsilon = Math.max(this.minEpsilon, this.epsilon * this.decayRate);
  }

  getEpsilon(): number {
    return this.epsilon;
  }
}
