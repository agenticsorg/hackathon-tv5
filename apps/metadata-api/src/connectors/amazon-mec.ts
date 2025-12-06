/**
 * Amazon MEC (Media Entertainment Core) Connector
 *
 * Implements the Amazon Prime Video metadata delivery specification
 * using the EMA Avails format (Entertainment Merchant's Association).
 *
 * @see https://movielabs.com/md/avails/
 * @see https://partnerhub.amazon.com/prime-video
 */

import { MediaMetadata, ValidationResult, ValidationError, ValidationWarning } from '../types/index.js';
import { BasePlatformConnector } from './base.js';

/**
 * MEC Feed Structure
 *
 * Represents the XML structure for Amazon MEC delivery
 * Based on EMA Avails v2.5 specification
 */
export interface MECFeed {
  avails: {
    xmlns: string;
    'xmlns:xsi': string;
    'xsi:schemaLocation': string;
    version: string;
    asset: MECAsset[];
  };
}

/**
 * MEC Asset Structure
 */
export interface MECAsset {
  assetMetadata: MECAssetMetadata;
  transaction: MECTransaction[];
}

/**
 * MEC Asset Metadata
 */
export interface MECAssetMetadata {
  basicMetadata: {
    contentID: string; // EIDR or proprietary ID
    titleDisplayUnlimited: string;
    titleInternalAlias?: string;
    releaseYear?: number;
    releaseDate?: string;
    workType: 'Movie' | 'Episode' | 'Season' | 'Series' | 'Short';
    runLength?: string; // PT1H30M format
    rating?: {
      system: string; // e.g., "MPAA", "TV"
      value: string; // e.g., "PG-13", "TV-MA"
    }[];
  };
  contentDescription?: {
    synopsis?: {
      language: string;
      text: string;
      length: number;
    }[];
    genre?: string[];
    keyword?: string[];
  };
  peopleMetadata?: {
    director?: string[];
    actor?: {
      name: string;
      characterName?: string;
      billingOrder?: number;
    }[];
    producer?: string[];
    writer?: string[];
  };
  technicalMetadata?: {
    language: string[];
    subtitleLanguage?: string[];
    audioConfiguration?: string; // e.g., "5.1", "Atmos"
    videoFormat?: string; // e.g., "HD", "UHD", "4K"
    aspectRatio?: string;
  };
}

/**
 * MEC Transaction (Rights and Terms)
 */
export interface MECTransaction {
  transactionID: string;
  territory: {
    country: string[]; // ISO 3166-1 alpha-3 codes
  };
  startDate: string; // ISO 8601
  endDate?: string; // ISO 8601
  licenseType: 'EST' | 'VOD' | 'SVOD' | 'AVOD' | 'TVOD' | 'FOD';
  exclusivity: 'Exclusive' | 'Non-Exclusive';
  priceType?: string;
  priceValue?: number;
  priceCurrency?: string;
}

/**
 * Amazon Content Specification
 *
 * Amazon Prime Video specific content requirements
 */
export interface AmazonContentSpec {
  // Video Requirements
  minResolution: '1080p' | '4K';
  requiredFormats: string[];
  preferredFormats: string[];

  // Audio Requirements
  minAudioChannels: string;
  supportedAudioCodecs: string[];

  // Metadata Requirements
  requiredFields: string[];
  maxSynopsisLength: number;
  minSynopsisLength: number;
  maxKeywords: number;

  // Rights Requirements
  exclusivityRequired: boolean;
  minLicenseDuration: number; // in days
}

/**
 * Prime Video Requirements
 *
 * Platform-specific validation rules
 */
export interface PrimeVideoRequirements {
  technical: {
    videoCodec: string[];
    audioCodec: string[];
    containerFormat: string[];
    minBitrate: number;
    maxBitrate: number;
  };
  content: {
    requiredRatings: string[];
    prohibitedContent: string[];
    requireClosedCaptions: boolean;
    requireAudioDescription: boolean;
  };
  metadata: {
    requireEIDR: boolean;
    requireOriginalLanguage: boolean;
    maxGenres: number;
    requireDirector: boolean;
    requireCast: boolean;
    minCastMembers: number;
  };
}

/**
 * Amazon MEC Connector
 *
 * Generates Amazon Prime Video compatible metadata feeds
 * in EMA Avails XML format with MEC extensions
 */
export class AmazonMECConnector extends BasePlatformConnector {
  readonly platform = 'amazon';

  /**
   * Amazon Content Specifications
   */
  private readonly contentSpec: AmazonContentSpec = {
    minResolution: '1080p',
    requiredFormats: ['MP4', 'ProRes'],
    preferredFormats: ['IMF', 'ProRes 4444'],
    minAudioChannels: '5.1',
    supportedAudioCodecs: ['AAC', 'Dolby Digital', 'Dolby Atmos', 'Dolby Digital Plus'],
    requiredFields: ['title', 'eidr', 'releaseDate', 'duration', 'synopsis', 'rating', 'language'],
    maxSynopsisLength: 500,
    minSynopsisLength: 50,
    maxKeywords: 20,
    exclusivityRequired: false,
    minLicenseDuration: 30,
  };

  /**
   * Prime Video Technical Requirements
   */
  private readonly primeVideoRequirements: PrimeVideoRequirements = {
    technical: {
      videoCodec: ['H.264', 'H.265', 'ProRes'],
      audioCodec: ['AAC', 'AC3', 'EAC3', 'Dolby Atmos'],
      containerFormat: ['MP4', 'MOV', 'IMF'],
      minBitrate: 5000000, // 5 Mbps
      maxBitrate: 25000000, // 25 Mbps
    },
    content: {
      requiredRatings: ['MPAA', 'TV'],
      prohibitedContent: [],
      requireClosedCaptions: true,
      requireAudioDescription: false,
    },
    metadata: {
      requireEIDR: true,
      requireOriginalLanguage: true,
      maxGenres: 3,
      requireDirector: true,
      requireCast: true,
      minCastMembers: 3,
    },
  };

  /**
   * Generate MEC Feed from Media Metadata
   *
   * @param metadata - Source media metadata
   * @returns MEC feed structure ready for XML serialization
   */
  async generateFeed(metadata: MediaMetadata): Promise<MECFeed> {
    // Validate before generating
    const validation = await this.validate(metadata);
    if (!validation.valid) {
      throw new Error(`Cannot generate MEC feed: ${validation.errors.map(e => e.message).join(', ')}`);
    }

    const mecFeed: MECFeed = {
      avails: {
        xmlns: 'http://www.movielabs.com/schema/avails/v2.5/avails',
        'xmlns:xsi': 'http://www.w3.org/2001/XMLSchema-instance',
        'xsi:schemaLocation': 'http://www.movielabs.com/schema/avails/v2.5/avails http://www.movielabs.com/schema/avails/v2.5/avails.xsd',
        version: '2.5',
        asset: [
          {
            assetMetadata: this.buildAssetMetadata(metadata),
            transaction: this.buildTransactions(metadata),
          },
        ],
      },
    };

    return mecFeed;
  }

  /**
   * Validate Amazon Content Requirements
   *
   * @param metadata - Media metadata to validate
   * @returns Validation result with errors and warnings
   */
  async validate(metadata: MediaMetadata): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Required fields validation
    if (!this.hasRequiredField(metadata.title, 'title')) {
      errors.push({
        field: 'title',
        message: 'Title is required for Amazon MEC delivery',
        severity: 'critical',
        platformRequirement: 'Amazon Prime Video requires titleDisplayUnlimited',
      });
    }

    if (this.primeVideoRequirements.metadata.requireEIDR && !metadata.eidr) {
      errors.push({
        field: 'eidr',
        message: 'EIDR is required for Amazon Prime Video',
        severity: 'critical',
        platformRequirement: 'Amazon requires EIDR or equivalent unique identifier',
      });
    }

    // Content metadata validation
    if (!metadata.synopsis || metadata.synopsis.length < this.contentSpec.minSynopsisLength) {
      errors.push({
        field: 'synopsis',
        message: `Synopsis must be at least ${this.contentSpec.minSynopsisLength} characters`,
        severity: 'error',
        platformRequirement: `Amazon requires descriptive synopsis (${this.contentSpec.minSynopsisLength}-${this.contentSpec.maxSynopsisLength} chars)`,
      });
    }

    if (metadata.synopsis && metadata.synopsis.length > this.contentSpec.maxSynopsisLength) {
      warnings.push({
        field: 'synopsis',
        message: `Synopsis exceeds ${this.contentSpec.maxSynopsisLength} characters, will be truncated`,
        recommendation: 'Reduce synopsis length for optimal display',
      });
    }

    // Rating validation
    if (!metadata.rating) {
      errors.push({
        field: 'rating',
        message: 'Content rating is required',
        severity: 'critical',
        platformRequirement: 'Amazon requires MPAA or TV rating',
      });
    }

    // Genre validation
    if (!metadata.genres || metadata.genres.length === 0) {
      warnings.push({
        field: 'genres',
        message: 'No genres specified',
        recommendation: 'Add at least one genre for better discoverability',
      });
    }

    if (metadata.genres && metadata.genres.length > this.primeVideoRequirements.metadata.maxGenres) {
      warnings.push({
        field: 'genres',
        message: `Maximum ${this.primeVideoRequirements.metadata.maxGenres} genres recommended`,
        recommendation: 'Select the most relevant genres',
      });
    }

    // Cast and crew validation
    if (this.primeVideoRequirements.metadata.requireDirector && (!metadata.director || metadata.director.length === 0)) {
      warnings.push({
        field: 'director',
        message: 'Director information missing',
        recommendation: 'Add director information for complete metadata',
      });
    }

    if (this.primeVideoRequirements.metadata.requireCast) {
      if (!metadata.cast || metadata.cast.length === 0) {
        warnings.push({
          field: 'cast',
          message: 'No cast information provided',
          recommendation: `Add at least ${this.primeVideoRequirements.metadata.minCastMembers} cast members`,
        });
      } else if (metadata.cast.length < this.primeVideoRequirements.metadata.minCastMembers) {
        warnings.push({
          field: 'cast',
          message: `Only ${metadata.cast.length} cast members provided`,
          recommendation: `Recommend at least ${this.primeVideoRequirements.metadata.minCastMembers} cast members`,
        });
      }
    }

    // Technical metadata validation
    if (!metadata.language) {
      errors.push({
        field: 'language',
        message: 'Primary language is required',
        severity: 'critical',
        platformRequirement: 'Amazon requires ISO 639-2 language code',
      });
    }

    if (!metadata.duration) {
      errors.push({
        field: 'duration',
        message: 'Runtime duration is required',
        severity: 'error',
        platformRequirement: 'Amazon requires runtime in ISO 8601 duration format',
      });
    }

    // Resolution validation
    if (metadata.resolution) {
      const allowedResolutions = ['4K', '1080p', '720p'];
      if (!allowedResolutions.includes(metadata.resolution)) {
        warnings.push({
          field: 'resolution',
          message: `Resolution '${metadata.resolution}' may not meet Amazon requirements`,
          recommendation: 'Amazon prefers 4K or 1080p content',
        });
      }
    } else {
      warnings.push({
        field: 'resolution',
        message: 'Resolution not specified',
        recommendation: 'Specify video resolution for technical compliance',
      });
    }

    // Rights validation
    if (metadata.platforms) {
      const amazonPlatform = metadata.platforms.find(p => p.platform === 'amazon');
      if (amazonPlatform) {
        if (!amazonPlatform.availableFrom) {
          errors.push({
            field: 'platforms.availableFrom',
            message: 'Start date required for Amazon availability window',
            severity: 'error',
            platformRequirement: 'Amazon requires valid start date for licensing',
          });
        }

        if (amazonPlatform.availableFrom && amazonPlatform.availableUntil) {
          const duration = new Date(amazonPlatform.availableUntil).getTime() - new Date(amazonPlatform.availableFrom).getTime();
          const durationDays = duration / (1000 * 60 * 60 * 24);

          if (durationDays < this.contentSpec.minLicenseDuration) {
            warnings.push({
              field: 'platforms.availableUntil',
              message: `License duration (${Math.round(durationDays)} days) is less than recommended minimum`,
              recommendation: `Amazon recommends minimum ${this.contentSpec.minLicenseDuration} days license duration`,
            });
          }
        }
      }
    }

    // Territory validation
    if (!metadata.territories || metadata.territories.length === 0) {
      warnings.push({
        field: 'territories',
        message: 'No territories specified',
        recommendation: 'Specify target territories for rights management',
      });
    }

    return {
      platform: this.platform,
      valid: errors.length === 0,
      errors,
      warnings,
      validatedAt: new Date(),
    };
  }

  /**
   * Export MEC Feed to XML
   *
   * @param feed - MEC feed structure
   * @returns XML string representation
   */
  exportFeed(feed: MECFeed): string {
    return this.generateMECXML(feed);
  }

  /**
   * Build Asset Metadata
   *
   * @private
   * @param metadata - Source media metadata
   * @returns MEC asset metadata structure
   */
  private buildAssetMetadata(metadata: MediaMetadata): MECAssetMetadata {
    const assetMetadata: MECAssetMetadata = {
      basicMetadata: {
        contentID: metadata.eidr || metadata.id,
        titleDisplayUnlimited: metadata.title,
        titleInternalAlias: metadata.id,
        releaseYear: metadata.releaseDate ? new Date(metadata.releaseDate).getFullYear() : undefined,
        releaseDate: metadata.releaseDate ? this.formatDate(metadata.releaseDate) : undefined,
        workType: this.mapWorkType(metadata.type),
        runLength: metadata.duration ? this.formatDuration(metadata.duration) : undefined,
      },
    };

    // Add rating if available
    if (metadata.rating) {
      assetMetadata.basicMetadata.rating = [
        {
          system: metadata.rating.startsWith('TV-') ? 'TV' : 'MPAA',
          value: metadata.rating,
        },
      ];
    }

    // Add content description
    if (metadata.synopsis || metadata.genres || metadata.keywords) {
      assetMetadata.contentDescription = {};

      if (metadata.synopsis) {
        assetMetadata.contentDescription.synopsis = [
          {
            language: metadata.language || 'en',
            text: metadata.synopsis.substring(0, this.contentSpec.maxSynopsisLength),
            length: Math.min(metadata.synopsis.length, this.contentSpec.maxSynopsisLength),
          },
        ];
      }

      if (metadata.genres && metadata.genres.length > 0) {
        assetMetadata.contentDescription.genre = metadata.genres.slice(0, this.primeVideoRequirements.metadata.maxGenres);
      }

      if (metadata.keywords && metadata.keywords.length > 0) {
        assetMetadata.contentDescription.keyword = metadata.keywords.slice(0, this.contentSpec.maxKeywords);
      }
    }

    // Add people metadata
    if (metadata.director || metadata.cast || metadata.producers || metadata.writers) {
      assetMetadata.peopleMetadata = {};

      if (metadata.director && metadata.director.length > 0) {
        assetMetadata.peopleMetadata.director = metadata.director;
      }

      if (metadata.cast && metadata.cast.length > 0) {
        assetMetadata.peopleMetadata.actor = metadata.cast.map((castMember, index) => ({
          name: castMember.name,
          characterName: castMember.characterName,
          billingOrder: castMember.order || index + 1,
        }));
      }

      if (metadata.producers && metadata.producers.length > 0) {
        assetMetadata.peopleMetadata.producer = metadata.producers;
      }

      if (metadata.writers && metadata.writers.length > 0) {
        assetMetadata.peopleMetadata.writer = metadata.writers;
      }
    }

    // Add technical metadata
    assetMetadata.technicalMetadata = {
      language: [metadata.language],
    };

    if (metadata.subtitles && metadata.subtitles.length > 0) {
      assetMetadata.technicalMetadata.subtitleLanguage = metadata.subtitles;
    }

    if (metadata.audioTracks && metadata.audioTracks.length > 0) {
      assetMetadata.technicalMetadata.audioConfiguration = this.contentSpec.minAudioChannels;
    }

    if (metadata.resolution) {
      assetMetadata.technicalMetadata.videoFormat = this.mapResolution(metadata.resolution);
    }

    if (metadata.aspectRatio) {
      assetMetadata.technicalMetadata.aspectRatio = metadata.aspectRatio;
    }

    return assetMetadata;
  }

  /**
   * Build Transactions (Rights and Terms)
   *
   * @private
   * @param metadata - Source media metadata
   * @returns Array of MEC transactions
   */
  private buildTransactions(metadata: MediaMetadata): MECTransaction[] {
    const transactions: MECTransaction[] = [];

    if (!metadata.platforms || !metadata.territories) {
      return transactions;
    }

    const amazonPlatform = metadata.platforms.find(p => p.platform === 'amazon');
    if (!amazonPlatform) {
      return transactions;
    }

    // Create transaction for each territory
    metadata.territories.forEach((territory, index) => {
      const transaction: MECTransaction = {
        transactionID: `TXN-${metadata.id}-${territory}-${index}`,
        territory: {
          country: [this.convertToISO3166Alpha3(territory)],
        },
        startDate: amazonPlatform.availableFrom ? this.formatDate(amazonPlatform.availableFrom) : this.formatDate(new Date()),
        endDate: amazonPlatform.availableUntil ? this.formatDate(amazonPlatform.availableUntil) : undefined,
        licenseType: amazonPlatform.subscriptionRequired ? 'SVOD' : 'AVOD',
        exclusivity: 'Non-Exclusive', // Default to non-exclusive
      };

      transactions.push(transaction);
    });

    return transactions;
  }

  /**
   * Map Nexus-UMMID content type to MEC WorkType
   *
   * @private
   * @param type - Nexus content type
   * @returns MEC WorkType
   */
  private mapWorkType(type: string): 'Movie' | 'Episode' | 'Season' | 'Series' | 'Short' {
    const typeMap: Record<string, 'Movie' | 'Episode' | 'Season' | 'Series' | 'Short'> = {
      movie: 'Movie',
      series: 'Series',
      episode: 'Episode',
      documentary: 'Movie',
      short: 'Short',
    };

    return typeMap[type] || 'Movie';
  }

  /**
   * Format duration to ISO 8601 duration format
   *
   * @private
   * @param minutes - Duration in minutes
   * @returns ISO 8601 duration string (e.g., "PT1H30M")
   */
  private formatDuration(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;

    if (hours > 0) {
      return `PT${hours}H${mins}M`;
    }
    return `PT${mins}M`;
  }

  /**
   * Map resolution to MEC video format
   *
   * @private
   * @param resolution - Resolution string
   * @returns MEC video format
   */
  private mapResolution(resolution: string): string {
    const resolutionMap: Record<string, string> = {
      '4K': 'UHD',
      '1080p': 'HD',
      '720p': 'HD',
      SD: 'SD',
    };

    return resolutionMap[resolution] || 'HD';
  }

  /**
   * Convert ISO 3166-1 alpha-2 to alpha-3 country code
   *
   * @private
   * @param alpha2 - Two-letter country code (e.g., "US")
   * @returns Three-letter country code (e.g., "USA")
   */
  private convertToISO3166Alpha3(alpha2: string): string {
    // Simplified mapping - in production, use a complete ISO 3166 library
    const countryMap: Record<string, string> = {
      US: 'USA',
      GB: 'GBR',
      CA: 'CAN',
      FR: 'FRA',
      DE: 'DEU',
      JP: 'JPN',
      AU: 'AUS',
      BR: 'BRA',
      IN: 'IND',
      MX: 'MEX',
      ES: 'ESP',
      IT: 'ITA',
      // Add more as needed
    };

    return countryMap[alpha2.toUpperCase()] || alpha2.toUpperCase();
  }

  /**
   * Generate MEC XML from feed structure
   *
   * @private
   * @param feed - MEC feed structure
   * @returns XML string
   */
  private generateMECXML(feed: MECFeed): string {
    const { avails } = feed;

    let xml = '<?xml version="1.0" encoding="UTF-8"?>\n';
    xml += `<Avails xmlns="${avails.xmlns}" `;
    xml += `xmlns:xsi="${avails['xmlns:xsi']}" `;
    xml += `xsi:schemaLocation="${avails['xsi:schemaLocation']}" `;
    xml += `version="${avails.version}">\n`;

    for (const asset of avails.asset) {
      xml += '  <Asset>\n';
      xml += this.generateAssetMetadataXML(asset.assetMetadata);

      for (const transaction of asset.transaction) {
        xml += this.generateTransactionXML(transaction);
      }

      xml += '  </Asset>\n';
    }

    xml += '</Avails>';

    return xml;
  }

  /**
   * Generate Asset Metadata XML
   *
   * @private
   * @param metadata - Asset metadata structure
   * @returns XML string fragment
   */
  private generateAssetMetadataXML(metadata: MECAssetMetadata): string {
    let xml = '    <AssetMetadata>\n';
    xml += '      <BasicMetadata>\n';

    const { basicMetadata } = metadata;
    xml += `        <ContentID>${this.escapeXML(basicMetadata.contentID)}</ContentID>\n`;
    xml += `        <TitleDisplayUnlimited>${this.escapeXML(basicMetadata.titleDisplayUnlimited)}</TitleDisplayUnlimited>\n`;

    if (basicMetadata.titleInternalAlias) {
      xml += `        <TitleInternalAlias>${this.escapeXML(basicMetadata.titleInternalAlias)}</TitleInternalAlias>\n`;
    }

    if (basicMetadata.releaseYear) {
      xml += `        <ReleaseYear>${basicMetadata.releaseYear}</ReleaseYear>\n`;
    }

    if (basicMetadata.releaseDate) {
      xml += `        <ReleaseDate>${basicMetadata.releaseDate}</ReleaseDate>\n`;
    }

    xml += `        <WorkType>${basicMetadata.workType}</WorkType>\n`;

    if (basicMetadata.runLength) {
      xml += `        <RunLength>${basicMetadata.runLength}</RunLength>\n`;
    }

    if (basicMetadata.rating) {
      for (const rating of basicMetadata.rating) {
        xml += `        <Rating>\n`;
        xml += `          <System>${this.escapeXML(rating.system)}</System>\n`;
        xml += `          <Value>${this.escapeXML(rating.value)}</Value>\n`;
        xml += `        </Rating>\n`;
      }
    }

    xml += '      </BasicMetadata>\n';

    // Content Description
    if (metadata.contentDescription) {
      xml += '      <ContentDescription>\n';

      if (metadata.contentDescription.synopsis) {
        for (const synopsis of metadata.contentDescription.synopsis) {
          xml += `        <Synopsis language="${synopsis.language}" length="${synopsis.length}">\n`;
          xml += `          ${this.escapeXML(synopsis.text)}\n`;
          xml += `        </Synopsis>\n`;
        }
      }

      if (metadata.contentDescription.genre) {
        for (const genre of metadata.contentDescription.genre) {
          xml += `        <Genre>${this.escapeXML(genre)}</Genre>\n`;
        }
      }

      if (metadata.contentDescription.keyword) {
        for (const keyword of metadata.contentDescription.keyword) {
          xml += `        <Keyword>${this.escapeXML(keyword)}</Keyword>\n`;
        }
      }

      xml += '      </ContentDescription>\n';
    }

    // People Metadata
    if (metadata.peopleMetadata) {
      xml += '      <PeopleMetadata>\n';

      if (metadata.peopleMetadata.director) {
        for (const director of metadata.peopleMetadata.director) {
          xml += `        <Director>${this.escapeXML(director)}</Director>\n`;
        }
      }

      if (metadata.peopleMetadata.actor) {
        for (const actor of metadata.peopleMetadata.actor) {
          xml += `        <Actor>\n`;
          xml += `          <Name>${this.escapeXML(actor.name)}</Name>\n`;
          if (actor.characterName) {
            xml += `          <CharacterName>${this.escapeXML(actor.characterName)}</CharacterName>\n`;
          }
          if (actor.billingOrder) {
            xml += `          <BillingOrder>${actor.billingOrder}</BillingOrder>\n`;
          }
          xml += `        </Actor>\n`;
        }
      }

      if (metadata.peopleMetadata.producer) {
        for (const producer of metadata.peopleMetadata.producer) {
          xml += `        <Producer>${this.escapeXML(producer)}</Producer>\n`;
        }
      }

      if (metadata.peopleMetadata.writer) {
        for (const writer of metadata.peopleMetadata.writer) {
          xml += `        <Writer>${this.escapeXML(writer)}</Writer>\n`;
        }
      }

      xml += '      </PeopleMetadata>\n';
    }

    // Technical Metadata
    if (metadata.technicalMetadata) {
      xml += '      <TechnicalMetadata>\n';

      for (const lang of metadata.technicalMetadata.language) {
        xml += `        <Language>${this.escapeXML(lang)}</Language>\n`;
      }

      if (metadata.technicalMetadata.subtitleLanguage) {
        for (const lang of metadata.technicalMetadata.subtitleLanguage) {
          xml += `        <SubtitleLanguage>${this.escapeXML(lang)}</SubtitleLanguage>\n`;
        }
      }

      if (metadata.technicalMetadata.audioConfiguration) {
        xml += `        <AudioConfiguration>${this.escapeXML(metadata.technicalMetadata.audioConfiguration)}</AudioConfiguration>\n`;
      }

      if (metadata.technicalMetadata.videoFormat) {
        xml += `        <VideoFormat>${this.escapeXML(metadata.technicalMetadata.videoFormat)}</VideoFormat>\n`;
      }

      if (metadata.technicalMetadata.aspectRatio) {
        xml += `        <AspectRatio>${this.escapeXML(metadata.technicalMetadata.aspectRatio)}</AspectRatio>\n`;
      }

      xml += '      </TechnicalMetadata>\n';
    }

    xml += '    </AssetMetadata>\n';

    return xml;
  }

  /**
   * Generate Transaction XML
   *
   * @private
   * @param transaction - Transaction structure
   * @returns XML string fragment
   */
  private generateTransactionXML(transaction: MECTransaction): string {
    let xml = '    <Transaction>\n';
    xml += `      <TransactionID>${this.escapeXML(transaction.transactionID)}</TransactionID>\n`;
    xml += '      <Territory>\n';

    for (const country of transaction.territory.country) {
      xml += `        <Country>${this.escapeXML(country)}</Country>\n`;
    }

    xml += '      </Territory>\n';
    xml += `      <Start>${transaction.startDate}</Start>\n`;

    if (transaction.endDate) {
      xml += `      <End>${transaction.endDate}</End>\n`;
    }

    xml += `      <LicenseType>${transaction.licenseType}</LicenseType>\n`;
    xml += `      <Exclusivity>${transaction.exclusivity}</Exclusivity>\n`;

    if (transaction.priceType && transaction.priceValue && transaction.priceCurrency) {
      xml += '      <Price>\n';
      xml += `        <PriceType>${this.escapeXML(transaction.priceType)}</PriceType>\n`;
      xml += `        <PriceValue>${transaction.priceValue}</PriceValue>\n`;
      xml += `        <Currency>${this.escapeXML(transaction.priceCurrency)}</Currency>\n`;
      xml += '      </Price>\n';
    }

    xml += '    </Transaction>\n';

    return xml;
  }

  /**
   * Escape XML special characters
   *
   * @private
   * @param text - Text to escape
   * @returns Escaped text safe for XML
   */
  private escapeXML(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&apos;');
  }
}
