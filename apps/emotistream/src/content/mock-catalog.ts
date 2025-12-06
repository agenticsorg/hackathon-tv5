/**
 * MockCatalogGenerator - Generates diverse mock content catalog
 */

import { ContentMetadata } from './types';

interface ContentTemplate {
  genres: string[];
  tags: string[];
  minDuration: number;
  maxDuration: number;
}

export class MockCatalogGenerator {
  private templates!: Map<string, ContentTemplate>;
  private movieTitles!: string[];
  private seriesTitles!: string[];
  private documentaryTitles!: string[];
  private musicTitles!: string[];
  private meditationTitles!: string[];
  private shortTitles!: string[];

  constructor() {
    this.initializeTemplates();
    this.initializeTitles();
  }

  /**
   * Generate mock content catalog
   */
  generate(count: number): ContentMetadata[] {
    const catalog: ContentMetadata[] = [];
    const categories: Array<'movie' | 'series' | 'documentary' | 'music' | 'meditation' | 'short'> =
      ['movie', 'series', 'documentary', 'music', 'meditation', 'short'];

    const itemsPerCategory = Math.floor(count / categories.length);
    let idCounter = 1;

    for (const category of categories) {
      const template = this.templates.get(category)!;
      const titles = this.getTitlesForCategory(category);

      for (let i = 0; i < itemsPerCategory; i++) {
        const content: ContentMetadata = {
          contentId: `mock_${category}_${idCounter.toString().padStart(3, '0')}`,
          title: titles[i % titles.length],
          description: this.generateDescription(category),
          platform: 'mock',
          genres: this.randomSample(template.genres, 2, 4),
          category,
          tags: this.randomSample(template.tags, 3, 6),
          duration: this.randomInt(template.minDuration, template.maxDuration)
        };

        catalog.push(content);
        idCounter++;
      }
    }

    // Fill remaining items
    while (catalog.length < count) {
      const category = categories[catalog.length % categories.length];
      const template = this.templates.get(category)!;
      const titles = this.getTitlesForCategory(category);

      catalog.push({
        contentId: `mock_${category}_${idCounter.toString().padStart(3, '0')}`,
        title: titles[idCounter % titles.length],
        description: this.generateDescription(category),
        platform: 'mock',
        genres: this.randomSample(template.genres, 2, 3),
        category,
        tags: this.randomSample(template.tags, 3, 5),
        duration: this.randomInt(template.minDuration, template.maxDuration)
      });

      idCounter++;
    }

    return catalog;
  }

  private initializeTemplates(): void {
    this.templates = new Map([
      ['movie', {
        genres: ['drama', 'comedy', 'thriller', 'romance', 'action', 'sci-fi', 'horror', 'fantasy'],
        tags: ['emotional', 'thought-provoking', 'feel-good', 'intense', 'inspiring', 'dark', 'uplifting'],
        minDuration: 90,
        maxDuration: 180
      }],
      ['series', {
        genres: ['drama', 'comedy', 'crime', 'fantasy', 'mystery', 'sci-fi', 'thriller'],
        tags: ['binge-worthy', 'character-driven', 'plot-twist', 'episodic', 'addictive', 'emotional'],
        minDuration: 30,
        maxDuration: 60
      }],
      ['documentary', {
        genres: ['nature', 'history', 'science', 'biographical', 'social', 'true-crime', 'wildlife'],
        tags: ['educational', 'eye-opening', 'inspiring', 'thought-provoking', 'informative', 'fascinating'],
        minDuration: 45,
        maxDuration: 120
      }],
      ['music', {
        genres: ['classical', 'jazz', 'ambient', 'world', 'electronic', 'instrumental', 'acoustic'],
        tags: ['relaxing', 'energizing', 'meditative', 'uplifting', 'atmospheric', 'soothing', 'inspiring'],
        minDuration: 3,
        maxDuration: 60
      }],
      ['meditation', {
        genres: ['guided', 'ambient', 'nature-sounds', 'mindfulness', 'breathing', 'sleep', 'relaxation'],
        tags: ['calming', 'stress-relief', 'sleep', 'focus', 'breathing', 'peaceful', 'grounding'],
        minDuration: 5,
        maxDuration: 45
      }],
      ['short', {
        genres: ['animation', 'comedy', 'experimental', 'musical', 'documentary', 'drama'],
        tags: ['quick-watch', 'creative', 'fun', 'bite-sized', 'quirky', 'entertaining', 'light'],
        minDuration: 1,
        maxDuration: 15
      }]
    ]);
  }

  private initializeTitles(): void {
    this.movieTitles = [
      'The Journey Within', 'Echoes of Tomorrow', 'Rising Hope', 'Shadows and Light',
      'The Last Dance', 'Whispers in the Wind', 'Finding Home', 'The Quiet Storm',
      'Dreams Unfold', 'Beyond the Horizon', 'The Art of Living', 'Silent Thunder',
      'Moments of Grace', 'The Path Ahead', 'Breaking Free', 'Hearts in Harmony',
      'The Golden Hour', 'Crossing Bridges', 'Uncharted Territory', 'The Final Chapter',
      'A New Beginning', 'The Turning Point', 'Into the Light', 'The Long Road',
      'Whispered Secrets', 'The Great Escape', 'Timeless Love', 'Through the Storm',
      'The Perfect Moment', 'Rising Tide', 'Lost and Found', 'The Simple Life'
    ];

    this.seriesTitles = [
      'The Chronicles', 'Hidden Truths', 'City Lights', 'Dark Waters',
      'The Investigation', 'Family Ties', 'Power Play', 'The Night Shift',
      'Breaking Point', 'The Syndicate', 'Second Chances', 'The Underground',
      'Crown and Glory', 'The Outsiders', 'Parallel Lives', 'The Bureau',
      'Dark Secrets', 'The Network', 'Rising Stars', 'The Compound'
    ];

    this.documentaryTitles = [
      'Our Planet Earth', 'The Human Story', 'Wonders of Nature', 'Ancient Civilizations',
      'The Space Race', 'Ocean Deep', 'Mountain High', 'Forest Spirits',
      'The Climate Crisis', 'Wildlife Warriors', 'Hidden Worlds', 'The Great Migration',
      'Cultures of the World', 'The Innovation Age', 'Art and Soul', 'The Last Frontier',
      'Desert Dreams', 'Polar Extremes', 'River of Life', 'Urban Jungle'
    ];

    this.musicTitles = [
      'Tranquil Waves', 'Morning Light', 'Evening Calm', 'Peaceful Journey',
      'Serene Moments', 'Gentle Breeze', 'Quiet Reflections', 'Soft Harmonies',
      'Ambient Dreams', 'Ethereal Sounds', 'Cosmic Flow', 'Nature Symphony',
      'Midnight Jazz', 'Classical Essence', 'Zen Garden', 'Acoustic Soul'
    ];

    this.meditationTitles = [
      'Deep Relaxation', 'Mindful Breathing', 'Sleep Soundly', 'Stress Relief',
      'Inner Peace', 'Calm Mind', 'Body Scan', 'Loving Kindness',
      'Morning Meditation', 'Evening Wind Down', 'Focus and Clarity', 'Gratitude Practice',
      'Ocean Meditation', 'Forest Bathing', 'Mountain Stillness', 'Sunset Calm'
    ];

    this.shortTitles = [
      'Quick Laugh', 'Animated Wonder', 'Creative Spark', 'Quirky Tale',
      'Mini Adventure', 'Laugh Break', 'Visual Delight', 'Bite-Sized Joy',
      'Fast Fun', 'Instant Smile', 'Quick Escape', 'Micro Story'
    ];
  }

  private getTitlesForCategory(category: string): string[] {
    switch (category) {
      case 'movie': return this.movieTitles;
      case 'series': return this.seriesTitles;
      case 'documentary': return this.documentaryTitles;
      case 'music': return this.musicTitles;
      case 'meditation': return this.meditationTitles;
      case 'short': return this.shortTitles;
      default: return this.movieTitles;
    }
  }

  private generateDescription(category: string): string {
    const descriptions = {
      movie: 'A captivating film that takes you on an emotional journey.',
      series: 'An engaging series that keeps you hooked episode after episode.',
      documentary: 'An enlightening documentary exploring fascinating subjects.',
      music: 'Beautiful music to enhance your mood and state of mind.',
      meditation: 'A guided practice to help you relax and find inner calm.',
      short: 'A delightful short that delivers entertainment in a compact format.'
    };
    return descriptions[category as keyof typeof descriptions] || 'Engaging content.';
  }

  private randomSample<T>(array: T[], min: number, max: number): T[] {
    const count = this.randomInt(min, max);
    const shuffled = [...array].sort(() => Math.random() - 0.5);
    return shuffled.slice(0, Math.min(count, array.length));
  }

  private randomInt(min: number, max: number): number {
    return Math.floor(Math.random() * (max - min + 1)) + min;
  }
}
