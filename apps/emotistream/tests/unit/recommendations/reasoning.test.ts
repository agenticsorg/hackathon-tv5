import { ReasoningGenerator } from '../../../src/recommendations/reasoning';

describe('ReasoningGenerator', () => {
  let generator: ReasoningGenerator;

  beforeEach(() => {
    generator = new ReasoningGenerator();
  });

  describe('generate', () => {
    it('should generate reasoning for stressed user', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: -0.3,
        arousal: 0.6,
        stressLevel: 0.8,
        dominance: 0.0,
        rawMetrics: {},
      };

      const desiredState = {
        valence: 0.5,
        arousal: -0.4,
      };

      const profile = {
        contentId: 'content1',
        title: 'Nature Documentary',
        platform: 'Netflix',
        valenceDelta: 0.8,
        arousalDelta: -1.0,
        stressReduction: 0.7,
        duration: 50,
        genres: ['Documentary', 'Nature'],
        embedding: new Float32Array(1536),
      };

      const qValue = 0.85;
      const isExploration = false;

      // Act
      const reasoning = generator.generate(
        currentState,
        desiredState,
        profile,
        qValue,
        isExploration
      );

      // Assert
      expect(reasoning).toContain('feeling');
      expect(reasoning).toContain('stressed');
      expect(reasoning).toContain('calm');
      expect(reasoning).toContain('relax');
      expect(reasoning).toContain('stress relief');
      expect(reasoning.length).toBeGreaterThan(50);
    });

    it('should include exploration marker when isExploration is true', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: 0.0,
        arousal: 0.0,
        stressLevel: 0.5,
        dominance: 0.0,
        rawMetrics: {},
      };

      const desiredState = {
        valence: 0.5,
        arousal: 0.3,
      };

      const profile = {
        contentId: 'content1',
        title: 'Action Movie',
        platform: 'Netflix',
        valenceDelta: 0.5,
        arousalDelta: 0.5,
        stressReduction: 0.2,
        duration: 120,
        genres: ['Action'],
        embedding: new Float32Array(1536),
      };

      const qValue = 0.5;
      const isExploration = true;

      // Act
      const reasoning = generator.generate(
        currentState,
        desiredState,
        profile,
        qValue,
        isExploration
      );

      // Assert
      expect(reasoning).toContain('New discovery');
    });

    it('should mention high confidence for high Q-value', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: 0.0,
        arousal: 0.0,
        stressLevel: 0.5,
        dominance: 0.0,
        rawMetrics: {},
      };

      const desiredState = {
        valence: 0.5,
        arousal: 0.3,
      };

      const profile = {
        contentId: 'content1',
        title: 'Comedy Show',
        platform: 'Netflix',
        valenceDelta: 0.6,
        arousalDelta: 0.4,
        stressReduction: 0.3,
        duration: 30,
        genres: ['Comedy'],
        embedding: new Float32Array(1536),
      };

      const qValue = 0.85;
      const isExploration = false;

      // Act
      const reasoning = generator.generate(
        currentState,
        desiredState,
        profile,
        qValue,
        isExploration
      );

      // Assert
      expect(reasoning).toContain('loved this content');
    });

    it('should mention experimental pick for low Q-value', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: 0.0,
        arousal: 0.0,
        stressLevel: 0.5,
        dominance: 0.0,
        rawMetrics: {},
      };

      const desiredState = {
        valence: 0.5,
        arousal: 0.3,
      };

      const profile = {
        contentId: 'content1',
        title: 'Indie Film',
        platform: 'YouTube',
        valenceDelta: 0.4,
        arousalDelta: 0.2,
        stressReduction: 0.2,
        duration: 90,
        genres: ['Indie'],
        embedding: new Float32Array(1536),
      };

      const qValue = 0.2;
      const isExploration = false;

      // Act
      const reasoning = generator.generate(
        currentState,
        desiredState,
        profile,
        qValue,
        isExploration
      );

      // Assert
      expect(reasoning).toContain('experimental pick');
    });

    it('should describe mood improvement for positive valence delta', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: -0.5,
        arousal: 0.0,
        stressLevel: 0.4,
        dominance: 0.0,
        rawMetrics: {},
      };

      const desiredState = {
        valence: 0.5,
        arousal: 0.0,
      };

      const profile = {
        contentId: 'content1',
        title: 'Uplifting Drama',
        platform: 'Netflix',
        valenceDelta: 0.7,
        arousalDelta: 0.0,
        stressReduction: 0.1,
        duration: 45,
        genres: ['Drama'],
        embedding: new Float32Array(1536),
      };

      const qValue = 0.6;
      const isExploration = false;

      // Act
      const reasoning = generator.generate(
        currentState,
        desiredState,
        profile,
        qValue,
        isExploration
      );

      // Assert
      expect(reasoning).toContain('improve your mood');
    });

    it('should describe relaxation for negative arousal delta', () => {
      // Arrange
      const currentState = {
        id: 'state1',
        userId: 'user123',
        timestamp: Date.now(),
        valence: 0.0,
        arousal: 0.7,
        stressLevel: 0.6,
        dominance: 0.0,
        rawMetrics: {},
      };

      const desiredState = {
        valence: 0.3,
        arousal: -0.3,
      };

      const profile = {
        contentId: 'content1',
        title: 'Meditation Video',
        platform: 'YouTube',
        valenceDelta: 0.3,
        arousalDelta: -0.8,
        stressReduction: 0.5,
        duration: 20,
        genres: ['Wellness'],
        embedding: new Float32Array(1536),
      };

      const qValue = 0.7;
      const isExploration = false;

      // Act
      const reasoning = generator.generate(
        currentState,
        desiredState,
        profile,
        qValue,
        isExploration
      );

      // Assert
      expect(reasoning).toContain('relax');
    });
  });
});
