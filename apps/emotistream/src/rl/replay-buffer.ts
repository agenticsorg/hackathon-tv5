import { EmotionalExperience } from './types';

export class ReplayBuffer {
  private experiences: EmotionalExperience[];
  private insertIndex: number;
  private currentSize: number;

  constructor(private readonly maxSize: number = 10000) {
    this.experiences = [];
    this.insertIndex = 0;
    this.currentSize = 0;
  }

  add(experience: EmotionalExperience): void {
    if (this.currentSize < this.maxSize) {
      this.experiences.push(experience);
      this.currentSize++;
    } else {
      this.experiences[this.insertIndex] = experience;
    }

    this.insertIndex = (this.insertIndex + 1) % this.maxSize;
  }

  sample(batchSize: number): EmotionalExperience[] {
    if (this.currentSize < batchSize) {
      return [];
    }

    const sampled: EmotionalExperience[] = [];
    const indices = new Set<number>();

    while (indices.size < batchSize) {
      const randomIndex = Math.floor(Math.random() * this.currentSize);
      if (!indices.has(randomIndex)) {
        indices.add(randomIndex);
        sampled.push(this.experiences[randomIndex]);
      }
    }

    return sampled;
  }

  size(): number {
    return this.currentSize;
  }
}
