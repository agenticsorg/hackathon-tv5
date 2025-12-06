/**
 * Netflix IMF (Interoperable Master Format) Connector
 *
 * Generates Netflix-compliant IMF packages with Dolby Vision support
 * and validates content against Netflix delivery specifications.
 *
 * @module connectors/netflix-imf
 */

import { MediaMetadata, ValidationResult, ValidationError, ValidationWarning } from '../types/index.js';

/**
 * Platform Connector Interface
 * Base interface for all platform-specific connectors
 */
export interface PlatformConnector {
  platformName: string;
  version: string;

  /**
   * Validates metadata against platform specifications
   */
  validate(metadata: MediaMetadata): ValidationResult;

  /**
   * Generates platform-specific delivery package
   */
  generatePackage(metadata: MediaMetadata): any;
}

/**
 * IMF Package Structure
 * Represents a complete Netflix IMF delivery package
 */
export interface IMFPackage {
  /** Composition Playlist - defines the timeline and structure */
  compositionPlaylist: CompositionPlaylist;

  /** Asset Map - inventory of all files in the package */
  assetMap: AssetMap;

  /** Packing List - cryptographic checksums for all assets */
  packingList: PackingList;

  /** Dolby Vision metadata sidecar (if applicable) */
  dolbyVisionMetadata?: DolbyVisionMetadata;

  /** Package UUID */
  packageId: string;

  /** Creation timestamp */
  createdAt: Date;
}

/**
 * Composition Playlist (CPL)
 * Defines the editorial structure and timeline
 */
export interface CompositionPlaylist {
  id: string;
  annotation: string;
  issueDate: string;
  creator: string;
  contentTitle: string;
  contentKind: 'feature' | 'trailer' | 'episode' | 'advertisement';

  /** Segments containing video, audio, subtitle tracks */
  segmentList: Segment[];

  /** Editorial rate (typically 24, 25, or 30 fps) */
  editRate: {
    numerator: number;
    denominator: number;
  };
}

/**
 * Segment in Composition Playlist
 */
export interface Segment {
  id: string;

  /** Video track reference */
  videoTrack?: TrackFileResource;

  /** Audio track references */
  audioTracks: TrackFileResource[];

  /** Subtitle track references */
  subtitleTracks?: TrackFileResource[];

  /** Segment duration in edit units */
  duration: number;
}

/**
 * Track File Resource
 */
export interface TrackFileResource {
  id: string;
  trackId: string;
  sourceEncoding: string;
  hash: string;
  hashAlgorithm: 'SHA-1' | 'SHA-256';
  duration: number;
  editRate: {
    numerator: number;
    denominator: number;
  };
}

/**
 * Asset Map (ASSETMAP)
 * Inventory of all files in the package
 */
export interface AssetMap {
  id: string;
  creator: string;
  issueDate: string;

  /** List of all assets */
  assets: Asset[];
}

/**
 * Asset Entry
 */
export interface Asset {
  id: string;
  packingList: boolean;
  chunkList: ChunkEntry[];
}

/**
 * Chunk Entry - physical file reference
 */
export interface ChunkEntry {
  path: string;
  volumeIndex: number;
  offset: number;
  length: number;
}

/**
 * Packing List (PKL)
 * Cryptographic integrity verification
 */
export interface PackingList {
  id: string;
  issueDate: string;
  creator: string;

  /** Assets with checksums */
  assets: PackingListAsset[];
}

/**
 * Packing List Asset
 */
export interface PackingListAsset {
  id: string;
  hash: string;
  hashAlgorithm: 'SHA-1' | 'SHA-256';
  size: number;
  type: string;
  originalFileName: string;
}

/**
 * Dolby Vision Metadata
 * HDR10+ and Dolby Vision Profile 5/8 support
 */
export interface DolbyVisionMetadata {
  /** Dolby Vision profile (5 for broadcast, 8 for OTT/streaming) */
  profile: 5 | 8;

  /** Dolby Vision level */
  level: number;

  /** RPU (Reference Processing Unit) metadata */
  rpuData?: string;

  /** HDR10 compatibility */
  hdr10Compatible: boolean;

  /** Color space */
  colorSpace: 'BT.2020' | 'P3-D65';

  /** Transfer function */
  transferFunction: 'PQ' | 'HLG';

  /** Mastering display metadata */
  masteringDisplay: {
    displayPrimaries: {
      red: { x: number; y: number };
      green: { x: number; y: number };
      blue: { x: number; y: number };
      whitePoint: { x: number; y: number };
    };
    luminance: {
      max: number; // cd/m²
      min: number; // cd/m²
    };
  };

  /** Content light level */
  contentLight: {
    maxCLL: number; // Maximum Content Light Level (cd/m²)
    maxFALL: number; // Maximum Frame Average Light Level (cd/m²)
  };
}

/**
 * Netflix Content Specification
 * Platform-specific validation rules
 */
export interface NetflixContentSpec {
  /** Title requirements */
  title: {
    minLength: number;
    maxLength: number;
    allowedCharacters: RegExp;
  };

  /** Synopsis requirements */
  synopsis: {
    minLength: number;
    maxLength: number;
  };

  /** Required metadata fields */
  requiredFields: string[];

  /** Technical requirements */
  technical: {
    minResolution: '1080p' | '4K';
    requiredAudioFormats: string[];
    requiredSubtitleFormats: string[];
    minBitrate: number; // in Mbps
    maxBitrate: number;
  };

  /** Rating requirements */
  allowedRatings: string[];

  /** Genre taxonomy */
  allowedGenres: string[];

  /** Dolby Vision requirements */
  dolbyVision: {
    required: boolean;
    profiles: number[];
    hdr10Fallback: boolean;
  };
}

/**
 * Netflix IMF Connector Implementation
 */
export class NetflixIMFConnector implements PlatformConnector {
  public readonly platformName = 'Netflix';
  public readonly version = '1.0.0';

  /**
   * Netflix content specification
   */
  private readonly netflixSpec: NetflixContentSpec = {
    title: {
      minLength: 1,
      maxLength: 250,
      allowedCharacters: /^[a-zA-Z0-9\s\-':!?.&]+$/
    },
    synopsis: {
      minLength: 50,
      maxLength: 500
    },
    requiredFields: [
      'title',
      'type',
      'synopsis',
      'genres',
      'language',
      'rating',
      'duration'
    ],
    technical: {
      minResolution: '4K',
      requiredAudioFormats: ['5.1', 'Atmos'],
      requiredSubtitleFormats: ['SRT', 'WebVTT'],
      minBitrate: 15,
      maxBitrate: 35
    },
    allowedRatings: ['G', 'PG', 'PG-13', 'R', 'NC-17', 'TV-Y', 'TV-Y7', 'TV-G', 'TV-PG', 'TV-14', 'TV-MA'],
    allowedGenres: [
      'Action',
      'Adventure',
      'Animation',
      'Comedy',
      'Crime',
      'Documentary',
      'Drama',
      'Family',
      'Fantasy',
      'Horror',
      'Mystery',
      'Romance',
      'Sci-Fi',
      'Thriller',
      'Western'
    ],
    dolbyVision: {
      required: true,
      profiles: [5, 8],
      hdr10Fallback: true
    }
  };

  /**
   * Validates media metadata against Netflix content specifications
   *
   * @param metadata - Media metadata to validate
   * @returns Validation result with errors and warnings
   */
  public validate(metadata: MediaMetadata): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Validate required fields
    for (const field of this.netflixSpec.requiredFields) {
      if (!metadata[field as keyof MediaMetadata]) {
        errors.push({
          field,
          message: `Required field '${field}' is missing`,
          severity: 'critical',
          platformRequirement: 'Netflix IMF Content Specification v3.0'
        });
      }
    }

    // Validate title
    if (metadata.title) {
      if (metadata.title.length < this.netflixSpec.title.minLength) {
        errors.push({
          field: 'title',
          message: `Title must be at least ${this.netflixSpec.title.minLength} characters`,
          severity: 'error',
          platformRequirement: 'Netflix Title Guidelines'
        });
      }

      if (metadata.title.length > this.netflixSpec.title.maxLength) {
        errors.push({
          field: 'title',
          message: `Title must not exceed ${this.netflixSpec.title.maxLength} characters`,
          severity: 'error',
          platformRequirement: 'Netflix Title Guidelines'
        });
      }

      if (!this.netflixSpec.title.allowedCharacters.test(metadata.title)) {
        warnings.push({
          field: 'title',
          message: 'Title contains special characters that may not display correctly',
          recommendation: 'Use only alphanumeric characters and standard punctuation'
        });
      }
    }

    // Validate synopsis
    if (metadata.synopsis) {
      if (metadata.synopsis.length < this.netflixSpec.synopsis.minLength) {
        warnings.push({
          field: 'synopsis',
          message: `Synopsis should be at least ${this.netflixSpec.synopsis.minLength} characters for better discovery`,
          recommendation: 'Provide a more detailed synopsis'
        });
      }

      if (metadata.synopsis.length > this.netflixSpec.synopsis.maxLength) {
        errors.push({
          field: 'synopsis',
          message: `Synopsis must not exceed ${this.netflixSpec.synopsis.maxLength} characters`,
          severity: 'error',
          platformRequirement: 'Netflix Synopsis Guidelines'
        });
      }
    }

    // Validate rating
    if (metadata.rating && this.netflixSpec.allowedRatings.indexOf(metadata.rating) === -1) {
      errors.push({
        field: 'rating',
        message: `Invalid rating '${metadata.rating}'. Allowed: ${this.netflixSpec.allowedRatings.join(', ')}`,
        severity: 'error',
        platformRequirement: 'Netflix Content Rating System'
      });
    }

    // Validate genres
    if (metadata.genres && metadata.genres.length > 0) {
      const invalidGenres = metadata.genres.filter(
        genre => this.netflixSpec.allowedGenres.indexOf(genre) === -1
      );

      if (invalidGenres.length > 0) {
        warnings.push({
          field: 'genres',
          message: `Unknown genres: ${invalidGenres.join(', ')}`,
          recommendation: `Use Netflix standard genres: ${this.netflixSpec.allowedGenres.join(', ')}`
        });
      }
    }

    // Validate resolution
    if (metadata.resolution && metadata.resolution !== '4K') {
      errors.push({
        field: 'resolution',
        message: 'Netflix requires 4K (UHD) resolution for new content',
        severity: 'critical',
        platformRequirement: 'Netflix Technical Specifications v7.0'
      });
    }

    // Validate Dolby Vision
    const hasDolbyVision = this.checkDolbyVisionSupport(metadata);
    if (!hasDolbyVision && this.netflixSpec.dolbyVision.required) {
      errors.push({
        field: 'technical',
        message: 'Dolby Vision metadata is required for Netflix 4K content',
        severity: 'critical',
        platformRequirement: 'Netflix HDR Specification'
      });
    }

    // Validate duration
    if (metadata.duration) {
      if (metadata.type === 'movie' && metadata.duration < 60) {
        warnings.push({
          field: 'duration',
          message: 'Feature films typically exceed 60 minutes',
          recommendation: 'Verify content type classification'
        });
      }

      if (metadata.type === 'episode' && metadata.duration > 90) {
        warnings.push({
          field: 'duration',
          message: 'Episodes typically do not exceed 90 minutes',
          recommendation: 'Consider classifying as a movie or special'
        });
      }
    }

    return {
      platform: this.platformName,
      valid: errors.length === 0,
      errors,
      warnings,
      validatedAt: new Date()
    };
  }

  /**
   * Validates content specifically for Netflix requirements
   *
   * @param metadata - Media metadata to validate
   * @returns Validation result
   */
  public validateNetflixContent(metadata: MediaMetadata): ValidationResult {
    return this.validate(metadata);
  }

  /**
   * Generates Netflix IMF package from metadata
   *
   * @param metadata - Media metadata
   * @returns Complete IMF package structure
   */
  public generatePackage(metadata: MediaMetadata): IMFPackage {
    return this.generateIMFPackage(metadata);
  }

  /**
   * Generates a complete Netflix IMF package
   *
   * @param metadata - Media metadata
   * @returns IMF package with CPL, ASSETMAP, PKL, and Dolby Vision metadata
   */
  public generateIMFPackage(metadata: MediaMetadata): IMFPackage {
    const packageId = this.generateUUID();
    const cplId = this.generateUUID();
    const assetMapId = this.generateUUID();
    const pklId = this.generateUUID();

    // Generate Composition Playlist
    const compositionPlaylist: CompositionPlaylist = {
      id: cplId,
      annotation: `Netflix IMF Package for ${metadata.title}`,
      issueDate: new Date().toISOString(),
      creator: 'Nexus-UMMID Platform',
      contentTitle: metadata.title,
      contentKind: this.mapContentKind(metadata.type),
      editRate: {
        numerator: 24,
        denominator: 1
      },
      segmentList: this.generateSegments(metadata)
    };

    // Generate Asset Map
    const assetMap: AssetMap = {
      id: assetMapId,
      creator: 'Nexus-UMMID Platform',
      issueDate: new Date().toISOString(),
      assets: this.generateAssetMapEntries(metadata, cplId, pklId)
    };

    // Generate Packing List
    const packingList: PackingList = {
      id: pklId,
      issueDate: new Date().toISOString(),
      creator: 'Nexus-UMMID Platform',
      assets: this.generatePackingListAssets(metadata)
    };

    // Generate Dolby Vision metadata if applicable
    const dolbyVisionMetadata = this.generateDolbyVisionMetadata(metadata);

    return {
      packageId,
      compositionPlaylist,
      assetMap,
      packingList,
      dolbyVisionMetadata,
      createdAt: new Date()
    };
  }

  /**
   * Checks if metadata includes Dolby Vision support
   *
   * @param metadata - Media metadata
   * @returns True if Dolby Vision is supported
   */
  private checkDolbyVisionSupport(metadata: MediaMetadata): boolean {
    // In a real implementation, this would check for Dolby Vision XML sidecar
    // or metadata fields indicating HDR support
    return metadata.resolution === '4K';
  }

  /**
   * Maps media type to IMF content kind
   *
   * @param type - Media type
   * @returns IMF content kind
   */
  private mapContentKind(type: string): 'feature' | 'trailer' | 'episode' | 'advertisement' {
    const mapping: Record<string, 'feature' | 'trailer' | 'episode' | 'advertisement'> = {
      'movie': 'feature',
      'documentary': 'feature',
      'short': 'trailer',
      'episode': 'episode',
      'series': 'episode'
    };

    return mapping[type] || 'feature';
  }

  /**
   * Generates segment list for composition playlist
   *
   * @param metadata - Media metadata
   * @returns Array of segments
   */
  private generateSegments(metadata: MediaMetadata): Segment[] {
    const duration = (metadata.duration || 90) * 60 * 24; // Convert minutes to frames (24fps)

    return [{
      id: this.generateUUID(),
      videoTrack: {
        id: this.generateUUID(),
        trackId: 'video-track-1',
        sourceEncoding: 'urn:smpte:ul:060e2b34.04010105.0e090502.01000000',
        hash: this.generateHash(`${metadata.id}-video`),
        hashAlgorithm: 'SHA-256',
        duration,
        editRate: {
          numerator: 24,
          denominator: 1
        }
      },
      audioTracks: [
        {
          id: this.generateUUID(),
          trackId: 'audio-track-1',
          sourceEncoding: 'urn:smpte:ul:060e2b34.04010105.0e090502.02000000',
          hash: this.generateHash(`${metadata.id}-audio-main`),
          hashAlgorithm: 'SHA-256',
          duration,
          editRate: {
            numerator: 24,
            denominator: 1
          }
        }
      ],
      subtitleTracks: (metadata.subtitles || []).map((lang, index) => ({
        id: this.generateUUID(),
        trackId: `subtitle-track-${index + 1}`,
        sourceEncoding: 'urn:smpte:ul:060e2b34.04010105.0e090502.03000000',
        hash: this.generateHash(`${metadata.id}-subtitle-${lang}`),
        hashAlgorithm: 'SHA-256',
        duration,
        editRate: {
          numerator: 24,
          denominator: 1
        }
      })),
      duration
    }];
  }

  /**
   * Generates asset map entries
   *
   * @param metadata - Media metadata
   * @param cplId - Composition Playlist ID
   * @param pklId - Packing List ID
   * @returns Array of assets
   */
  private generateAssetMapEntries(metadata: MediaMetadata, cplId: string, pklId: string): Asset[] {
    return [
      {
        id: cplId,
        packingList: false,
        chunkList: [{
          path: `CPL_${cplId}.xml`,
          volumeIndex: 1,
          offset: 0,
          length: 0
        }]
      },
      {
        id: pklId,
        packingList: true,
        chunkList: [{
          path: `PKL_${pklId}.xml`,
          volumeIndex: 1,
          offset: 0,
          length: 0
        }]
      },
      {
        id: this.generateUUID(),
        packingList: false,
        chunkList: [{
          path: `${metadata.id}_video_4k.mxf`,
          volumeIndex: 1,
          offset: 0,
          length: 0
        }]
      },
      {
        id: this.generateUUID(),
        packingList: false,
        chunkList: [{
          path: `${metadata.id}_audio_atmos.mxf`,
          volumeIndex: 1,
          offset: 0,
          length: 0
        }]
      }
    ];
  }

  /**
   * Generates packing list assets with checksums
   *
   * @param metadata - Media metadata
   * @returns Array of packing list assets
   */
  private generatePackingListAssets(metadata: MediaMetadata): PackingListAsset[] {
    return [
      {
        id: this.generateUUID(),
        hash: this.generateHash(`${metadata.id}-video-mxf`),
        hashAlgorithm: 'SHA-256',
        size: 0,
        type: 'application/mxf',
        originalFileName: `${metadata.id}_video_4k.mxf`
      },
      {
        id: this.generateUUID(),
        hash: this.generateHash(`${metadata.id}-audio-mxf`),
        hashAlgorithm: 'SHA-256',
        size: 0,
        type: 'application/mxf',
        originalFileName: `${metadata.id}_audio_atmos.mxf`
      }
    ];
  }

  /**
   * Generates Dolby Vision metadata for HDR content
   *
   * @param metadata - Media metadata
   * @returns Dolby Vision metadata or undefined
   */
  private generateDolbyVisionMetadata(metadata: MediaMetadata): DolbyVisionMetadata | undefined {
    if (metadata.resolution !== '4K') {
      return undefined;
    }

    return {
      profile: 8, // Profile 8 for OTT/streaming
      level: 5,
      hdr10Compatible: true,
      colorSpace: 'BT.2020',
      transferFunction: 'PQ',
      masteringDisplay: {
        displayPrimaries: {
          red: { x: 0.708, y: 0.292 },
          green: { x: 0.170, y: 0.797 },
          blue: { x: 0.131, y: 0.046 },
          whitePoint: { x: 0.3127, y: 0.3290 }
        },
        luminance: {
          max: 4000, // 4000 cd/m² (nits)
          min: 0.0001 // 0.0001 cd/m²
        }
      },
      contentLight: {
        maxCLL: 4000, // Maximum Content Light Level
        maxFALL: 400  // Maximum Frame Average Light Level
      }
    };
  }

  /**
   * Generates a UUID v4
   *
   * @returns UUID string
   */
  private generateUUID(): string {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = Math.random() * 16 | 0;
      const v = c === 'x' ? r : (r & 0x3 | 0x8);
      return v.toString(16);
    });
  }

  /**
   * Generates a SHA-256 hash placeholder
   *
   * @param input - Input string
   * @returns Hash string
   */
  private generateHash(input: string): string {
    // In production, this would use crypto.createHash('sha256')
    // Using a simple hash placeholder for now
    let hash = 0;
    for (let i = 0; i < input.length; i++) {
      const char = input.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    const hexHash = Math.abs(hash).toString(16);
    // Pad to 64 characters
    const padding = new Array(64 - hexHash.length + 1).join('0');
    const paddedHash = padding + hexHash;
    return `sha256-${paddedHash}`;
  }
}

/**
 * Factory function to create a Netflix IMF connector instance
 *
 * @returns NetflixIMFConnector instance
 */
export function createNetflixIMFConnector(): NetflixIMFConnector {
  return new NetflixIMFConnector();
}

/**
 * Default export
 */
export default NetflixIMFConnector;
