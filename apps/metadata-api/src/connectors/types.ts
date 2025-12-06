/**
 * Platform Connector Type Definitions
 *
 * Unified interfaces for streaming platform metadata connectors
 * supporting Netflix IMF, Amazon MEC, and FAST channel formats.
 */

import { MediaMetadata, ValidationResult as BaseValidationResult } from '../types';

/**
 * Supported streaming platforms and channels
 */
export enum Platform {
  // Premium Streaming
  NETFLIX = 'netflix',
  AMAZON = 'amazon',
  HULU = 'hulu',
  DISNEY = 'disney',
  APPLE = 'apple',
  HBO = 'hbo',
  PARAMOUNT = 'paramount',
  PEACOCK = 'peacock',

  // FAST Channels
  FAST_PLUTO = 'fast-pluto',
  FAST_TUBI = 'fast-tubi',
  FAST_ROKU = 'fast-roku',
  FAST_XUMO = 'fast-xumo',
  FAST_SAMSUNG = 'fast-samsung',
  FAST_VIZIO = 'fast-vizio',

  // Custom/Generic
  CUSTOM = 'custom'
}

/**
 * Platform-specific package formats
 */
export type PackageFormat =
  | 'imf'        // Interoperable Master Format (Netflix, Amazon)
  | 'mec'        // Media Entertainment Command (Amazon)
  | 'mrss'       // Media RSS (FAST channels)
  | 'cpix'       // Content Protection Information Exchange
  | 'xml'        // Generic XML
  | 'json';      // Generic JSON

/**
 * Package output structure
 * Union type for different platform-specific package formats
 */
export type PackageOutput =
  | IMFPackage
  | MECPackage
  | MRSSPackage
  | GenericPackage;

/**
 * Netflix/Premium IMF Package
 */
export interface IMFPackage {
  format: 'imf';
  version: string;
  assetMap: {
    id: string;
    annotation?: string;
    creator?: string;
    issueDate: string;
    assets: IMFAsset[];
  };
  packingList: {
    id: string;
    essenceDescriptors: IMFEssenceDescriptor[];
    segmentList: IMFSegment[];
  };
  compositionPlaylist: {
    id: string;
    editRate: string;
    virtualTracks: IMFVirtualTrack[];
  };
  metadata: {
    coreMetadata: MediaMetadata;
    platformSpecific?: Record<string, any>;
  };
}

export interface IMFAsset {
  id: string;
  packingList?: boolean;
  hash?: string;
  size?: number;
  type: string;
}

export interface IMFEssenceDescriptor {
  id: string;
  type: 'video' | 'audio' | 'subtitle' | 'caption';
  codec: string;
  bitrate?: number;
  sampleRate?: number;
  channels?: number;
}

export interface IMFSegment {
  id: string;
  duration: number;
  editRate: string;
  resources: string[];
}

export interface IMFVirtualTrack {
  trackId: string;
  type: 'video' | 'audio' | 'subtitle';
  sequences: Array<{
    resourceId: string;
    entryPoint: number;
    duration: number;
  }>;
}

/**
 * Amazon MEC Package
 */
export interface MECPackage {
  format: 'mec';
  version: string;
  manifest: {
    version: string;
    title: string;
    contentId: string;
    provider: string;
    assets: MECAsset[];
  };
  metadata: {
    title: MediaMetadata;
    technical: MECTechnicalMetadata;
    rights: MECRightsMetadata;
  };
  deliverySpecification: {
    videoSpec: MECVideoSpec;
    audioSpec: MECAudioSpec[];
    subtitleSpec?: MECSubtitleSpec[];
  };
}

export interface MECAsset {
  type: 'video' | 'audio' | 'subtitle' | 'image';
  path: string;
  checksum: string;
  checksumType: 'MD5' | 'SHA256';
  size: number;
}

export interface MECTechnicalMetadata {
  duration: number;
  frameRate: string;
  aspectRatio: string;
  resolution: string;
  colorSpace?: string;
  hdr?: boolean;
}

export interface MECRightsMetadata {
  territories: string[];
  startDate?: string;
  endDate?: string;
  exclusivity?: boolean;
  windowType?: string;
}

export interface MECVideoSpec {
  codec: 'H.264' | 'H.265' | 'VP9' | 'AV1';
  profile: string;
  level: string;
  bitrate: number;
  resolution: string;
  frameRate: string;
  colorPrimaries?: string;
  transferCharacteristics?: string;
}

export interface MECAudioSpec {
  language: string;
  codec: 'AAC' | 'AC3' | 'EAC3' | 'Opus';
  channels: number;
  sampleRate: number;
  bitrate: number;
}

export interface MECSubtitleSpec {
  language: string;
  format: 'SRT' | 'TTML' | 'WebVTT' | 'IMSC1';
  isForced?: boolean;
  isSDH?: boolean;
}

/**
 * FAST Channel MRSS Package
 */
export interface MRSSPackage {
  format: 'mrss';
  version: string;
  channel: {
    title: string;
    description: string;
    link: string;
    language: string;
    copyright?: string;
    managingEditor?: string;
  };
  items: MRSSItem[];
}

export interface MRSSItem {
  guid: string;
  title: string;
  description: string;
  pubDate: string;
  category: string[];
  mediaContent: MRSSMediaContent[];
  mediaThumbnail?: MRSSMediaThumbnail[];
  mediaRating?: MRSSMediaRating;
  mediaCopyright?: string;
  mediaKeywords?: string[];
}

export interface MRSSMediaContent {
  url: string;
  type: string;
  medium: 'video' | 'audio' | 'image';
  duration?: number;
  width?: number;
  height?: number;
  bitrate?: number;
  framerate?: number;
  isDefault?: boolean;
  expression?: 'full' | 'sample' | 'nonstop';
}

export interface MRSSMediaThumbnail {
  url: string;
  width?: number;
  height?: number;
  time?: string;
}

export interface MRSSMediaRating {
  scheme: string;
  content: string;
}

/**
 * Generic package format fallback
 */
export interface GenericPackage {
  format: 'json' | 'xml';
  version: string;
  platform: Platform;
  metadata: MediaMetadata;
  assets: Array<{
    type: string;
    path: string;
    properties: Record<string, any>;
  }>;
  platformSpecific?: Record<string, any>;
}

/**
 * Extended validation result with platform-specific details
 */
export interface PlatformValidationResult extends BaseValidationResult {
  format: PackageFormat;
  schemaVersion?: string;
  technicalChecks?: {
    videoCodec?: boolean;
    audioCodec?: boolean;
    resolution?: boolean;
    bitrate?: boolean;
    duration?: boolean;
  };
  complianceScore?: number; // 0-100
  recommendations?: string[];
}

/**
 * Connector configuration options
 */
export interface ConnectorConfig {
  platform: Platform;
  version?: string;
  strictMode?: boolean;
  validateOnGenerate?: boolean;
  includeOptionalFields?: boolean;
  customMapping?: Record<string, string>;
  credentials?: {
    apiKey?: string;
    secret?: string;
    endpoint?: string;
  };
}

/**
 * Base Platform Connector Interface
 *
 * All platform-specific connectors must implement this interface
 * to ensure consistent validation and package generation.
 *
 * @example
 * ```typescript
 * const connector = ConnectorFactory.getInstance(Platform.NETFLIX);
 * const validation = await connector.validate(metadata);
 *
 * if (validation.valid) {
 *   const package = await connector.generate(metadata);
 *   console.log('Generated IMF package:', package);
 * }
 * ```
 */
export interface PlatformConnector {
  /**
   * Platform identifier
   */
  readonly platform: Platform;

  /**
   * Supported package format(s)
   */
  readonly format: PackageFormat;

  /**
   * Connector version
   */
  readonly version: string;

  /**
   * Validate metadata against platform-specific requirements
   *
   * @param metadata - Media metadata to validate
   * @param config - Optional connector configuration
   * @returns Validation result with errors, warnings, and compliance score
   *
   * @example
   * ```typescript
   * const result = await connector.validate(metadata, {
   *   strictMode: true,
   *   validateOnGenerate: true
   * });
   *
   * if (!result.valid) {
   *   console.error('Validation errors:', result.errors);
   * }
   * ```
   */
  validate(
    metadata: MediaMetadata,
    config?: Partial<ConnectorConfig>
  ): Promise<PlatformValidationResult>;

  /**
   * Generate platform-specific package from metadata
   *
   * @param metadata - Media metadata to package
   * @param config - Optional connector configuration
   * @returns Platform-specific package (IMF, MEC, MRSS, etc.)
   *
   * @example
   * ```typescript
   * const imfPackage = await connector.generate(metadata, {
   *   version: '1.1',
   *   includeOptionalFields: true
   * });
   *
   * // Serialize to XML/JSON
   * const xml = await connector.serialize(imfPackage, 'xml');
   * ```
   */
  generate(
    metadata: MediaMetadata,
    config?: Partial<ConnectorConfig>
  ): Promise<PackageOutput>;

  /**
   * Serialize package to string format (XML, JSON)
   *
   * @param packageData - Generated package output
   * @param format - Serialization format ('xml' or 'json')
   * @returns Serialized package string
   */
  serialize(
    packageData: PackageOutput,
    format: 'xml' | 'json'
  ): Promise<string>;

  /**
   * Parse platform package back to metadata
   *
   * @param packageString - Serialized package string
   * @param format - Package format ('xml' or 'json')
   * @returns Parsed media metadata
   */
  parse(
    packageString: string,
    format: 'xml' | 'json'
  ): Promise<MediaMetadata>;
}

/**
 * Connector capabilities and feature support
 */
export interface ConnectorCapabilities {
  supportsHDR: boolean;
  supportsDolbyVision: boolean;
  supportsDolbyAtmos: boolean;
  supportsMultipleAudioTracks: boolean;
  supportsSubtitles: boolean;
  supportsChapters: boolean;
  supportsTrickPlay: boolean;
  maxResolution: '4K' | '8K' | '1080p' | '720p';
  maxBitrate: number; // Mbps
  supportedVideoCodecs: string[];
  supportedAudioCodecs: string[];
}
