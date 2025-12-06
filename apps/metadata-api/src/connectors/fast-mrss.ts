/**
 * FAST Platform MRSS (Media RSS) Connector
 *
 * Generates Media RSS 2.0 feeds for Free Ad-Supported Streaming TV platforms
 * including Pluto TV, Tubi, Samsung TV Plus, Roku Channel, and others.
 *
 * Implements the PlatformConnector interface for consistent behavior
 * across all platform-specific connectors.
 *
 * @module FASTMRSSConnector
 * @see https://www.rssboard.org/media-rss
 * @see SMPTE ST 2067 for IMF interoperability
 */

import { MediaMetadata, ValidationError, ValidationWarning } from '../types/index.js';
import {
  PlatformConnector,
  Platform,
  PackageFormat,
  PlatformValidationResult,
  MRSSPackage,
  MRSSItem,
  MRSSMediaThumbnail,
  ConnectorConfig,
  ConnectorCapabilities
} from './types.js';

// ============================================================================
// FAST-SPECIFIC TYPES & INTERFACES
// ============================================================================

/**
 * Linear Schedule Metadata (for FAST channels)
 */
export interface LinearScheduleMetadata {
  channelId: string;
  channelName: string;
  startTime: string; // ISO 8601
  endTime: string;   // ISO 8601
  timezone: string;  // e.g., "America/New_York"
  repeatPattern?: 'daily' | 'weekly' | 'monthly' | 'once';
  priority?: number; // 1-10, higher = more important
}

/**
 * Ad Break Metadata (SCTE-35 markers)
 */
export interface AdBreakMetadata {
  timeOffset: number; // seconds from start
  duration: number;   // seconds
  adType: 'preroll' | 'midroll' | 'postroll';
  scte35?: string;    // SCTE-35 marker data
}

/**
 * FAST Platform Content Specification
 */
export interface FASTContentSpec {
  platform: Platform;
  channelId?: string;
  requiresScheduling: boolean;
  supportedFormats: string[];
  maxBitrate?: number;
  requiresAdMarkers: boolean;
  minimumDuration?: number;    // minutes
  maximumDuration?: number;    // minutes
  thumbnailSpec: {
    minWidth: number;
    minHeight: number;
    aspectRatio: string;
    formats: string[];
  };
}

/**
 * Extended MRSS Item with FAST-specific metadata
 */
export interface FASTMRSSItem extends MRSSItem {
  'fast:schedule'?: LinearScheduleMetadata;
  'fast:adBreaks'?: AdBreakMetadata[];
}

// ============================================================================
// FAST MRSS CONNECTOR IMPLEMENTATION
// ============================================================================

/**
 * FAST MRSS Connector
 *
 * Generates Media RSS 2.0 feeds for FAST platforms with support for:
 * - Linear TV scheduling metadata
 * - Ad break insertion markers (SCTE-35)
 * - Multi-platform validation (Pluto TV, Tubi, Samsung TV Plus, etc.)
 * - EPG (Electronic Program Guide) integration
 *
 * @implements {PlatformConnector}
 *
 * @example
 * ```typescript
 * const connector = new FASTMRSSConnector(Platform.FAST_PLUTO, 'channel-123');
 * const validation = await connector.validate(metadata);
 *
 * if (validation.valid) {
 *   const package = await connector.generate(metadata);
 *   const xml = await connector.serialize(package, 'xml');
 *   console.log('Generated MRSS feed:', xml);
 * }
 * ```
 */
export class FASTMRSSConnector implements PlatformConnector {
  readonly platform: Platform;
  readonly format: PackageFormat = 'mrss';
  readonly version = '2.0';

  private channelId?: string;
  private config: FASTContentSpec;
  private connectorConfig: Partial<ConnectorConfig>;

  /**
   * Create a FAST MRSS connector
   *
   * @param platform - Target FAST platform (default: FAST_PLUTO)
   * @param channelId - Optional channel ID for linear scheduling
   * @param config - Optional connector configuration
   */
  constructor(
    platform: Platform = Platform.FAST_PLUTO,
    channelId?: string,
    config?: Partial<ConnectorConfig>
  ) {
    // Ensure platform is a FAST platform
    if (!this.isFASTPlatform(platform)) {
      throw new Error(`Platform ${platform} is not a FAST platform. Use Platform.FAST_* enums.`);
    }

    this.platform = platform;
    this.channelId = channelId;
    this.config = this.getPlatformSpec(platform);
    this.connectorConfig = {
      strictMode: false,
      validateOnGenerate: true,
      includeOptionalFields: true,
      ...config
    };
  }

  /**
   * Check if platform is a FAST platform
   */
  private isFASTPlatform(platform: Platform): boolean {
    return [
      Platform.FAST_PLUTO,
      Platform.FAST_TUBI,
      Platform.FAST_ROKU,
      Platform.FAST_XUMO,
      Platform.FAST_SAMSUNG,
      Platform.FAST_VIZIO
    ].includes(platform);
  }

  /**
   * Get platform-specific requirements and capabilities
   */
  private getPlatformSpec(platform: Platform): FASTContentSpec {
    const specs: Record<string, FASTContentSpec> = {
      [Platform.FAST_PLUTO]: {
        platform: Platform.FAST_PLUTO,
        requiresScheduling: true,
        supportedFormats: ['mp4', 'hls', 'dash'],
        maxBitrate: 10000,
        requiresAdMarkers: true,
        minimumDuration: 5,
        maximumDuration: 180,
        thumbnailSpec: {
          minWidth: 1920,
          minHeight: 1080,
          aspectRatio: '16:9',
          formats: ['jpg', 'png']
        }
      },
      [Platform.FAST_TUBI]: {
        platform: Platform.FAST_TUBI,
        requiresScheduling: false,
        supportedFormats: ['mp4', 'hls'],
        maxBitrate: 8000,
        requiresAdMarkers: true,
        minimumDuration: 10,
        thumbnailSpec: {
          minWidth: 1920,
          minHeight: 1080,
          aspectRatio: '16:9',
          formats: ['jpg']
        }
      },
      [Platform.FAST_SAMSUNG]: {
        platform: Platform.FAST_SAMSUNG,
        requiresScheduling: true,
        supportedFormats: ['hls', 'dash'],
        maxBitrate: 10000,
        requiresAdMarkers: true,
        minimumDuration: 5,
        thumbnailSpec: {
          minWidth: 1280,
          minHeight: 720,
          aspectRatio: '16:9',
          formats: ['jpg', 'png']
        }
      },
      [Platform.FAST_ROKU]: {
        platform: Platform.FAST_ROKU,
        requiresScheduling: false,
        supportedFormats: ['hls', 'mp4'],
        maxBitrate: 8000,
        requiresAdMarkers: true,
        minimumDuration: 5,
        thumbnailSpec: {
          minWidth: 1920,
          minHeight: 1080,
          aspectRatio: '16:9',
          formats: ['jpg']
        }
      },
      [Platform.FAST_XUMO]: {
        platform: Platform.FAST_XUMO,
        requiresScheduling: true,
        supportedFormats: ['hls', 'dash'],
        requiresAdMarkers: true,
        thumbnailSpec: {
          minWidth: 1280,
          minHeight: 720,
          aspectRatio: '16:9',
          formats: ['jpg']
        }
      },
      [Platform.FAST_VIZIO]: {
        platform: Platform.FAST_VIZIO,
        requiresScheduling: true,
        supportedFormats: ['hls'],
        requiresAdMarkers: true,
        thumbnailSpec: {
          minWidth: 1920,
          minHeight: 1080,
          aspectRatio: '16:9',
          formats: ['jpg']
        }
      }
    };

    return specs[platform] || specs[Platform.FAST_PLUTO];
  }

  /**
   * Validate metadata against FAST platform requirements
   *
   * @param metadata - Media metadata to validate
   * @param config - Optional validation configuration
   * @returns Validation result with platform-specific checks
   */
  async validate(
    metadata: MediaMetadata,
    config?: Partial<ConnectorConfig>
  ): Promise<PlatformValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const strictMode = config?.strictMode ?? this.connectorConfig.strictMode;

    // Required fields validation
    if (!metadata.title || metadata.title.trim().length === 0) {
      errors.push({
        field: 'title',
        message: 'Title is required',
        severity: 'critical',
        platformRequirement: 'All FAST platforms require a title'
      });
    }

    if (!metadata.synopsis && !metadata.description) {
      errors.push({
        field: 'synopsis',
        message: 'Synopsis or description is required',
        severity: 'error',
        platformRequirement: 'FAST platforms require content description'
      });
    }

    if (!metadata.duration || metadata.duration <= 0) {
      errors.push({
        field: 'duration',
        message: 'Duration must be specified and greater than 0',
        severity: 'critical',
        platformRequirement: 'Content duration is required for scheduling'
      });
    }

    // Platform-specific duration validation
    if (this.config.minimumDuration && metadata.duration && metadata.duration < this.config.minimumDuration) {
      errors.push({
        field: 'duration',
        message: `Content duration (${metadata.duration} min) is below minimum (${this.config.minimumDuration} min)`,
        severity: 'error',
        platformRequirement: `${this.platform} requires minimum ${this.config.minimumDuration} minutes`
      });
    }

    if (this.config.maximumDuration && metadata.duration && metadata.duration > this.config.maximumDuration) {
      warnings.push({
        field: 'duration',
        message: `Content duration (${metadata.duration} min) exceeds maximum (${this.config.maximumDuration} min)`,
        recommendation: 'Consider splitting into episodes or reducing length'
      });
    }

    // Genre validation
    if (!metadata.genres || metadata.genres.length === 0) {
      if (strictMode) {
        errors.push({
          field: 'genres',
          message: 'At least one genre is required',
          severity: 'error',
          platformRequirement: 'Genres are required for content categorization'
        });
      } else {
        warnings.push({
          field: 'genres',
          message: 'No genres specified',
          recommendation: 'Add at least one genre for better content discovery'
        });
      }
    }

    // Rating validation
    if (!metadata.rating) {
      warnings.push({
        field: 'rating',
        message: 'Content rating is missing',
        recommendation: 'Add content rating (e.g., TV-PG, TV-14, TV-MA) for better compliance'
      });
    }

    // Scheduling validation
    if (this.config.requiresScheduling && !this.channelId) {
      warnings.push({
        field: 'channelId',
        message: `${this.platform} requires linear channel scheduling`,
        recommendation: 'Provide channelId when initializing connector'
      });
    }

    // Ad marker validation
    if (this.config.requiresAdMarkers) {
      warnings.push({
        field: 'adBreaks',
        message: `${this.platform} requires ad break markers`,
        recommendation: 'Add SCTE-35 markers or ad break timecodes to content'
      });
    }

    // Title length validation
    if (metadata.title && metadata.title.length > 100) {
      warnings.push({
        field: 'title',
        message: 'Title is longer than recommended 100 characters',
        recommendation: 'Shorten title for better display on TV interfaces'
      });
    }

    // Description length validation
    const description = metadata.synopsis || metadata.description || '';
    if (description.length > 500) {
      warnings.push({
        field: 'synopsis',
        message: 'Synopsis is longer than recommended 500 characters',
        recommendation: 'Provide a shorter synopsis for TV display'
      });
    }

    // Technical checks
    const technicalChecks = {
      videoCodec: true, // Would check actual video file metadata
      audioCodec: true,
      resolution: metadata.resolution ? ['1080p', '720p', 'SD'].includes(metadata.resolution) : false,
      bitrate: true,
      duration: metadata.duration ? metadata.duration > 0 : false
    };

    // Calculate compliance score
    const complianceScore = this.calculateCompliance(errors, warnings, technicalChecks);

    return {
      platform: this.platform,
      format: this.format,
      schemaVersion: this.version,
      valid: errors.length === 0,
      errors,
      warnings,
      technicalChecks,
      complianceScore,
      recommendations: this.generateRecommendations(errors, warnings),
      validatedAt: new Date()
    };
  }

  /**
   * Generate MRSS package from metadata
   *
   * @param metadata - Media metadata to transform
   * @param config - Optional generation configuration
   * @returns MRSS package structure
   */
  async generate(
    metadata: MediaMetadata,
    config?: Partial<ConnectorConfig>
  ): Promise<MRSSPackage> {
    // Validate first if enabled
    if (this.connectorConfig.validateOnGenerate || config?.validateOnGenerate) {
      const validation = await this.validate(metadata, config);
      if (!validation.valid && (config?.strictMode || this.connectorConfig.strictMode)) {
        throw new Error(
          `Validation failed: ${validation.errors.map(e => e.message).join(', ')}`
        );
      }
    }

    // Build MRSS package
    const mrssPackage: MRSSPackage = {
      format: 'mrss',
      version: this.version,
      channel: {
        title: `${this.platform.replace('fast-', '').toUpperCase()} - ${this.channelId || 'Feed'}`,
        description: `Media RSS feed for ${this.platform} platform`,
        link: 'https://nexus-ummid.io',
        language: metadata.language || 'en-us',
        copyright: '© Nexus-UMMID',
        managingEditor: 'metadata@nexus-ummid.io'
      },
      items: [this.convertToMRSSItem(metadata)]
    };

    return mrssPackage;
  }

  /**
   * Serialize MRSS package to XML or JSON
   *
   * @param packageData - MRSS package to serialize
   * @param format - Output format ('xml' or 'json')
   * @returns Serialized package string
   */
  async serialize(packageData: MRSSPackage, format: 'xml' | 'json'): Promise<string> {
    if (format === 'json') {
      return JSON.stringify(packageData, null, 2);
    }

    // Convert to MRSS XML
    return this.toMRSSXML(packageData);
  }

  /**
   * Parse MRSS feed back to metadata
   *
   * @param packageString - Serialized MRSS feed
   * @param format - Input format ('xml' or 'json')
   * @returns Parsed media metadata
   */
  async parse(packageString: string, format: 'xml' | 'json'): Promise<MediaMetadata> {
    if (format === 'json') {
      const pkg = JSON.parse(packageString) as MRSSPackage;
      if (pkg.items.length === 0) {
        throw new Error('MRSS package contains no items');
      }
      return this.convertFromMRSSItem(pkg.items[0]);
    }

    // Parse MRSS XML
    return this.parseFromMRSSXML(packageString);
  }

  /**
   * Convert MediaMetadata to MRSS item
   */
  private convertToMRSSItem(metadata: MediaMetadata): FASTMRSSItem {
    const contentUrl = this.generateContentURL(metadata);

    const item: FASTMRSSItem = {
      guid: metadata.id,
      title: metadata.title,
      description: metadata.synopsis || metadata.description || '',
      pubDate: metadata.releaseDate?.toUTCString() || new Date().toUTCString(),
      category: metadata.genres || [],
      mediaContent: [{
        url: contentUrl,
        type: 'video/mp4',
        medium: 'video',
        duration: metadata.duration ? metadata.duration * 60 : undefined,
        width: this.getWidth(metadata.resolution),
        height: this.getHeight(metadata.resolution),
        isDefault: true,
        expression: 'full'
      }],
      mediaKeywords: metadata.keywords
    };

    // Add thumbnails
    if (this.connectorConfig.includeOptionalFields) {
      item.mediaThumbnail = this.generateThumbnails(metadata);
    }

    // Add rating
    if (metadata.rating) {
      item.mediaRating = {
        scheme: 'urn:mpaa',
        content: metadata.rating
      };
    }

    // Add copyright
    if (this.connectorConfig.includeOptionalFields) {
      item.mediaCopyright = '© Nexus-UMMID';
    }

    // Add scheduling metadata for linear platforms
    if (this.config.requiresScheduling && this.channelId) {
      item['fast:schedule'] = this.generateScheduleMetadata(metadata);
    }

    // Add ad break markers if required
    if (this.config.requiresAdMarkers) {
      item['fast:adBreaks'] = this.generateAdBreaks(metadata);
    }

    return item;
  }

  /**
   * Convert MRSS item back to MediaMetadata
   */
  private convertFromMRSSItem(item: MRSSItem): MediaMetadata {
    const metadata: MediaMetadata = {
      id: item.guid,
      title: item.title,
      type: 'movie', // Default to movie, would need to detect from metadata
      synopsis: item.description,
      genres: item.category || [],
      keywords: item.mediaKeywords || [],
      language: 'en',
      releaseDate: new Date(item.pubDate),
      createdAt: new Date(),
      updatedAt: new Date()
    };

    // Extract duration from media content
    if (item.mediaContent && item.mediaContent.length > 0) {
      const content = item.mediaContent[0];
      if (content.duration) {
        metadata.duration = content.duration / 60; // Convert seconds to minutes
      }
      if (content.width && content.height) {
        metadata.resolution = this.getResolution(content.width, content.height);
      }
    }

    // Extract rating
    if (item.mediaRating) {
      metadata.rating = item.mediaRating.content;
    }

    return metadata;
  }

  /**
   * Generate content URL (placeholder implementation)
   */
  private generateContentURL(metadata: MediaMetadata): string {
    return `https://cdn.nexus-ummid.io/content/${metadata.id}/master.m3u8`;
  }

  /**
   * Generate thumbnails
   */
  private generateThumbnails(metadata: MediaMetadata): MRSSMediaThumbnail[] {
    return [
      {
        url: `https://cdn.nexus-ummid.io/thumbnails/${metadata.id}/poster.jpg`,
        width: this.config.thumbnailSpec.minWidth,
        height: this.config.thumbnailSpec.minHeight
      },
      {
        url: `https://cdn.nexus-ummid.io/thumbnails/${metadata.id}/landscape.jpg`,
        width: 1920,
        height: 1080
      }
    ];
  }

  /**
   * Generate linear schedule metadata
   */
  private generateScheduleMetadata(metadata: MediaMetadata): LinearScheduleMetadata {
    const now = new Date();
    const endTime = new Date(now.getTime() + (metadata.duration || 60) * 60000);

    return {
      channelId: this.channelId!,
      channelName: `Channel ${this.channelId}`,
      startTime: now.toISOString(),
      endTime: endTime.toISOString(),
      timezone: 'America/New_York',
      repeatPattern: 'daily',
      priority: 5
    };
  }

  /**
   * Generate ad break markers
   */
  private generateAdBreaks(metadata: MediaMetadata): AdBreakMetadata[] {
    if (!metadata.duration) return [];

    const durationSeconds = metadata.duration * 60;
    const adBreaks: AdBreakMetadata[] = [];

    // Pre-roll ad
    adBreaks.push({
      timeOffset: 0,
      duration: 30,
      adType: 'preroll'
    });

    // Mid-roll ads every 15 minutes for content longer than 20 minutes
    if (durationSeconds > 1200) {
      const numberOfMidrolls = Math.floor(durationSeconds / 900) - 1;
      for (let i = 1; i <= numberOfMidrolls; i++) {
        adBreaks.push({
          timeOffset: i * 900,
          duration: 60,
          adType: 'midroll'
        });
      }
    }

    // Post-roll ad
    adBreaks.push({
      timeOffset: durationSeconds - 30,
      duration: 30,
      adType: 'postroll'
    });

    return adBreaks;
  }

  /**
   * Convert MRSS package to XML string
   */
  private toMRSSXML(pkg: MRSSPackage): string {
    let xml = '<?xml version="1.0" encoding="UTF-8"?>\n';
    xml += '<rss version="2.0" xmlns:media="http://search.yahoo.com/mrss/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:fast="http://nexus-ummid.io/fast/1.0">\n';
    xml += '  <channel>\n';
    xml += `    <title>${this.escapeXml(pkg.channel.title)}</title>\n`;
    xml += `    <link>${this.escapeXml(pkg.channel.link)}</link>\n`;
    xml += `    <description>${this.escapeXml(pkg.channel.description)}</description>\n`;
    xml += `    <language>${pkg.channel.language}</language>\n`;

    if (pkg.channel.copyright) {
      xml += `    <copyright>${this.escapeXml(pkg.channel.copyright)}</copyright>\n`;
    }

    if (pkg.channel.managingEditor) {
      xml += `    <managingEditor>${this.escapeXml(pkg.channel.managingEditor)}</managingEditor>\n`;
    }

    // Add items
    for (const item of pkg.items) {
      xml += this.itemToXML(item as FASTMRSSItem);
    }

    xml += '  </channel>\n';
    xml += '</rss>';

    return xml;
  }

  /**
   * Convert MRSS item to XML
   */
  private itemToXML(item: FASTMRSSItem): string {
    let xml = '    <item>\n';
    xml += `      <guid>${this.escapeXml(item.guid)}</guid>\n`;
    xml += `      <title>${this.escapeXml(item.title)}</title>\n`;
    xml += `      <description><![CDATA[${item.description}]]></description>\n`;
    xml += `      <pubDate>${item.pubDate}</pubDate>\n`;

    // Categories
    for (const category of item.category) {
      xml += `      <category>${this.escapeXml(category)}</category>\n`;
    }

    // Media content
    for (const content of item.mediaContent) {
      xml += `      <media:content url="${this.escapeXml(content.url)}" `;
      xml += `type="${content.type}" medium="${content.medium}"`;
      if (content.duration) xml += ` duration="${content.duration}"`;
      if (content.width) xml += ` width="${content.width}"`;
      if (content.height) xml += ` height="${content.height}"`;
      if (content.isDefault) xml += ` isDefault="true"`;
      xml += ' />\n';
    }

    // Thumbnails
    if (item.mediaThumbnail) {
      for (const thumb of item.mediaThumbnail) {
        xml += `      <media:thumbnail url="${this.escapeXml(thumb.url)}"`;
        if (thumb.width) xml += ` width="${thumb.width}"`;
        if (thumb.height) xml += ` height="${thumb.height}"`;
        xml += ' />\n';
      }
    }

    // Rating
    if (item.mediaRating) {
      xml += `      <media:rating scheme="${item.mediaRating.scheme}">${this.escapeXml(item.mediaRating.content)}</media:rating>\n`;
    }

    // Keywords
    if (item.mediaKeywords && item.mediaKeywords.length > 0) {
      xml += `      <media:keywords>${this.escapeXml(item.mediaKeywords.join(', '))}</media:keywords>\n`;
    }

    // Copyright
    if (item.mediaCopyright) {
      xml += `      <media:copyright>${this.escapeXml(item.mediaCopyright)}</media:copyright>\n`;
    }

    // Schedule
    if (item['fast:schedule']) {
      const schedule = item['fast:schedule'];
      xml += `      <fast:schedule>\n`;
      xml += `        <fast:channelId>${this.escapeXml(schedule.channelId)}</fast:channelId>\n`;
      xml += `        <fast:startTime>${schedule.startTime}</fast:startTime>\n`;
      xml += `        <fast:endTime>${schedule.endTime}</fast:endTime>\n`;
      xml += `        <fast:timezone>${schedule.timezone}</fast:timezone>\n`;
      if (schedule.repeatPattern) {
        xml += `        <fast:repeatPattern>${schedule.repeatPattern}</fast:repeatPattern>\n`;
      }
      xml += `      </fast:schedule>\n`;
    }

    // Ad breaks
    if (item['fast:adBreaks']) {
      xml += `      <fast:adBreaks>\n`;
      for (const adBreak of item['fast:adBreaks']) {
        xml += `        <fast:adBreak>\n`;
        xml += `          <fast:timeOffset>${adBreak.timeOffset}</fast:timeOffset>\n`;
        xml += `          <fast:duration>${adBreak.duration}</fast:duration>\n`;
        xml += `          <fast:adType>${adBreak.adType}</fast:adType>\n`;
        xml += `        </fast:adBreak>\n`;
      }
      xml += `      </fast:adBreaks>\n`;
    }

    xml += '    </item>\n';
    return xml;
  }

  /**
   * Parse MRSS XML to metadata (stub implementation)
   */
  private parseFromMRSSXML(_xml: string): MediaMetadata {
    // In production, use a proper XML parser like fast-xml-parser
    throw new Error('MRSS XML parsing not yet implemented. Use JSON format for parsing.');
  }

  /**
   * Helper methods
   */

  private getWidth(resolution?: '4K' | '1080p' | '720p' | 'SD'): number {
    const widths = { '4K': 3840, '1080p': 1920, '720p': 1280, 'SD': 720 };
    return widths[resolution || '1080p'];
  }

  private getHeight(resolution?: '4K' | '1080p' | '720p' | 'SD'): number {
    const heights = { '4K': 2160, '1080p': 1080, '720p': 720, 'SD': 480 };
    return heights[resolution || '1080p'];
  }

  private getResolution(width: number, height: number): '4K' | '1080p' | '720p' | 'SD' {
    if (width >= 3840 || height >= 2160) return '4K';
    if (width >= 1920 || height >= 1080) return '1080p';
    if (width >= 1280 || height >= 720) return '720p';
    return 'SD';
  }

  private calculateCompliance(errors: ValidationError[], warnings: ValidationWarning[], _checks: any): number {
    const errorWeight = errors.length * 15;
    const warningWeight = warnings.length * 5;
    const technicalWeight = Object.values(_checks).filter(v => !v).length * 10;

    return Math.max(0, 100 - errorWeight - warningWeight - technicalWeight);
  }

  private generateRecommendations(errors: ValidationError[], warnings: ValidationWarning[]): string[] {
    const recommendations: string[] = [];

    if (errors.length > 0) {
      recommendations.push(`Fix ${errors.length} critical error(s) before submission`);
    }

    if (warnings.length > 3) {
      recommendations.push('Address validation warnings to improve content quality');
    }

    if (this.config.requiresScheduling && !this.channelId) {
      recommendations.push('Configure channel ID for linear TV scheduling');
    }

    if (this.config.requiresAdMarkers) {
      recommendations.push('Add SCTE-35 ad markers for better monetization');
    }

    return recommendations;
  }

  private escapeXml(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&apos;');
  }

  /**
   * Get connector capabilities
   */
  static getCapabilities(platform: Platform): ConnectorCapabilities {
    return {
      supportsHDR: false,
      supportsDolbyVision: false,
      supportsDolbyAtmos: false,
      supportsMultipleAudioTracks: true,
      supportsSubtitles: true,
      supportsChapters: false,
      supportsTrickPlay: false,
      maxResolution: '1080p',
      maxBitrate: platform === Platform.FAST_PLUTO || platform === Platform.FAST_SAMSUNG ? 10 : 8,
      supportedVideoCodecs: ['H.264', 'H.265'],
      supportedAudioCodecs: ['AAC', 'AC3']
    };
  }
}

// ============================================================================
// EXPORTS
// ============================================================================

export default FASTMRSSConnector;
