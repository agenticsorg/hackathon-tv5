/**
 * Mapper Unit Tests (TDD - Red Phase)
 */

import { ValenceArousalMapper } from '../../../src/emotion/mappers/valence-arousal';
import { PlutchikMapper } from '../../../src/emotion/mappers/plutchik';
import { StressCalculator } from '../../../src/emotion/mappers/stress';
import { PlutchikEmotion } from '../../../src/emotion/types';

describe('ValenceArousalMapper', () => {
  let mapper: ValenceArousalMapper;

  beforeEach(() => {
    mapper = new ValenceArousalMapper();
  });

  it('should map valid Gemini response', () => {
    // Arrange
    const response = {
      primaryEmotion: 'joy' as PlutchikEmotion,
      valence: 0.8,
      arousal: 0.6,
      stressLevel: 0.3,
      confidence: 0.9,
      reasoning: 'Test',
      rawResponse: {},
    };

    // Act
    const result = mapper.map(response);

    // Assert
    expect(result.valence).toBe(0.8);
    expect(result.arousal).toBe(0.6);
  });

  it('should normalize values outside circumplex', () => {
    // Arrange
    const response = {
      primaryEmotion: 'joy' as PlutchikEmotion,
      valence: 1.2, // Out of range
      arousal: 1.0,
      stressLevel: 0.3,
      confidence: 0.9,
      reasoning: 'Test',
      rawResponse: {},
    };

    // Act
    const result = mapper.map(response);

    // Assert
    const magnitude = Math.sqrt(result.valence ** 2 + result.arousal ** 2);
    expect(magnitude).toBeLessThanOrEqual(1.414); // √2
  });

  it('should handle null/undefined valence with neutral default', () => {
    // Arrange
    const response = {
      primaryEmotion: 'joy' as PlutchikEmotion,
      valence: null as any,
      arousal: 0.5,
      stressLevel: 0.3,
      confidence: 0.9,
      reasoning: 'Test',
      rawResponse: {},
    };

    // Act
    const result = mapper.map(response);

    // Assert
    expect(result.valence).toBe(0.0);
  });

  it('should handle null/undefined arousal with neutral default', () => {
    // Arrange
    const response = {
      primaryEmotion: 'joy' as PlutchikEmotion,
      valence: 0.5,
      arousal: undefined as any,
      stressLevel: 0.3,
      confidence: 0.9,
      reasoning: 'Test',
      rawResponse: {},
    };

    // Act
    const result = mapper.map(response);

    // Assert
    expect(result.arousal).toBe(0.0);
  });
});

describe('PlutchikMapper', () => {
  let mapper: PlutchikMapper;

  beforeEach(() => {
    mapper = new PlutchikMapper();
  });

  it('should generate normalized vector for joy', () => {
    // Act
    const vector = mapper.generateVector('joy', 0.8);

    // Assert
    const sum = Array.from(vector).reduce((a, b) => a + b, 0);
    expect(sum).toBeCloseTo(1.0, 2);
  });

  it('should make primary emotion dominant', () => {
    // Act
    const vector = mapper.generateVector('joy', 0.8);

    // Assert - joy is index 0
    expect(vector[0]).toBeGreaterThan(0.5);
  });

  it('should suppress opposite emotion', () => {
    // Act
    const vector = mapper.generateVector('joy', 0.8);

    // Assert - sadness (opposite of joy) is index 1
    expect(vector[1]).toBe(0.0);
  });

  it('should assign weight to adjacent emotions', () => {
    // Act
    const vector = mapper.generateVector('joy', 0.8);

    // Assert - trust and anticipation are adjacent to joy
    expect(vector[4]).toBeGreaterThan(0.0); // trust
    expect(vector[7]).toBeGreaterThan(0.0); // anticipation
  });

  it('should handle all 8 emotions', () => {
    // Arrange
    const emotions: PlutchikEmotion[] = [
      'joy',
      'sadness',
      'anger',
      'fear',
      'trust',
      'disgust',
      'surprise',
      'anticipation',
    ];

    emotions.forEach(emotion => {
      // Act
      const vector = mapper.generateVector(emotion, 0.7);

      // Assert
      const sum = Array.from(vector).reduce((a, b) => a + b, 0);
      expect(sum).toBeCloseTo(1.0, 2);
    });
  });

  it('should clamp intensity to valid range', () => {
    // Act - intensity > 1.0 should be clamped
    const vector1 = mapper.generateVector('joy', 1.5);
    const vector2 = mapper.generateVector('joy', 1.0);

    // Assert - should produce same result
    expect(Array.from(vector1)).toEqual(Array.from(vector2));
  });
});

describe('StressCalculator', () => {
  let calculator: StressCalculator;

  beforeEach(() => {
    calculator = new StressCalculator();
  });

  it('should calculate high stress for Q2 (negative + high arousal)', () => {
    // Act
    const stress = calculator.calculate(-0.8, 0.7);

    // Assert
    expect(stress).toBeGreaterThan(0.8);
  });

  it('should calculate low stress for Q4 (positive + low arousal)', () => {
    // Act
    const stress = calculator.calculate(0.7, -0.4);

    // Assert
    expect(stress).toBeLessThan(0.2);
  });

  it('should calculate moderate stress for Q1 (positive + high arousal)', () => {
    // Act
    const stress = calculator.calculate(0.8, 0.6);

    // Assert
    expect(stress).toBeGreaterThan(0.2);
    expect(stress).toBeLessThan(0.5);
  });

  it('should calculate moderate stress for Q3 (negative + low arousal)', () => {
    // Act
    const stress = calculator.calculate(-0.6, -0.4);

    // Assert
    expect(stress).toBeGreaterThan(0.4);
    expect(stress).toBeLessThan(0.7);
  });

  it('should boost stress for extreme negative valence', () => {
    // Act
    const stress1 = calculator.calculate(-0.5, 0.5);
    const stress2 = calculator.calculate(-0.9, 0.5);

    // Assert
    expect(stress2).toBeGreaterThan(stress1);
  });

  it('should return value in range [0, 1]', () => {
    // Act - test extreme values
    const stress1 = calculator.calculate(-1.0, 1.0);
    const stress2 = calculator.calculate(1.0, -1.0);
    const stress3 = calculator.calculate(0.0, 0.0);

    // Assert
    expect(stress1).toBeGreaterThanOrEqual(0.0);
    expect(stress1).toBeLessThanOrEqual(1.0);
    expect(stress2).toBeGreaterThanOrEqual(0.0);
    expect(stress2).toBeLessThanOrEqual(1.0);
    expect(stress3).toBeGreaterThanOrEqual(0.0);
    expect(stress3).toBeLessThanOrEqual(1.0);
  });
});
