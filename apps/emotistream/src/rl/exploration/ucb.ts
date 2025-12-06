export class UCBCalculator {
  constructor(private readonly c: number = 2.0) {}

  calculate(qValue: number, visitCount: number, totalVisits: number): number {
    if (visitCount === 0) {
      return Infinity;
    }

    const explorationBonus = this.c * Math.sqrt(Math.log(totalVisits) / visitCount);
    return qValue + explorationBonus;
  }
}
