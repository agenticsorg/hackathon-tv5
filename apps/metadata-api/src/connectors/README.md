# Platform Connectors - Architecture Documentation

## Overview

The Platform Connectors module provides a unified interface for generating and validating metadata packages across multiple streaming platforms including Netflix, Amazon Prime Video, and FAST channels (Pluto TV, Tubi, Roku, etc.).

## Architecture Design

### Design Patterns

1. **Factory Pattern** - `ConnectorFactory` provides centralized connector instantiation
2. **Strategy Pattern** - Each platform implements the `PlatformConnector` interface
3. **Singleton Pattern** - Connector instances are cached for performance
4. **Interface Segregation** - Clear separation between validation, generation, and serialization

### Core Components

```
connectors/
├── types.ts              # All interfaces, types, and enums
├── index.ts              # Factory and unified API
├── netflix-imf.ts        # Netflix IMF connector (to be implemented)
├── amazon-mec.ts         # Amazon MEC connector (to be implemented)
├── fast-mrss.ts          # FAST MRSS connector (to be implemented)
└── README.md            # This file
```

## Type System

### Platform Enum

Supports 16 streaming platforms:

**Premium Platforms:**
- Netflix, Amazon, Hulu, Disney+, Apple TV+, HBO Max, Paramount+, Peacock

**FAST Channels:**
- Pluto TV, Tubi, Roku Channel, Xumo, Samsung TV Plus, Vizio WatchFree+

**Custom:**
- Generic/Custom platforms

### Package Formats

| Platform | Format | Description |
|----------|--------|-------------|
| Netflix | IMF | Interoperable Master Format (SMPTE ST 2067) |
| Amazon | MEC | Media Entertainment Command |
| Hulu | IMF | Interoperable Master Format |
| Disney+ | IMF | Interoperable Master Format |
| Apple TV+ | IMF | Interoperable Master Format |
| HBO Max | IMF | Interoperable Master Format |
| Paramount+ | IMF | Interoperable Master Format |
| Peacock | IMF | Interoperable Master Format |
| FAST Channels | MRSS | Media RSS 2.0 |
| Custom | JSON/XML | Generic formats |

### Package Structures

#### IMF Package (Netflix, Disney+, Apple TV+)

```typescript
interface IMFPackage {
  format: 'imf';
  version: string;
  assetMap: {
    id: string;
    annotation?: string;
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
  };
}
```

#### MEC Package (Amazon Prime Video)

```typescript
interface MECPackage {
  format: 'mec';
  version: string;
  manifest: {
    version: string;
    title: string;
    contentId: string;
    provider: string;
    assets: MECAsset[];
  };
  deliverySpecification: {
    videoSpec: MECVideoSpec;
    audioSpec: MECAudioSpec[];
    subtitleSpec?: MECSubtitleSpec[];
  };
}
```

#### MRSS Package (FAST Channels)

```typescript
interface MRSSPackage {
  format: 'mrss';
  version: string;
  channel: {
    title: string;
    description: string;
    link: string;
  };
  items: MRSSItem[];
}
```

## PlatformConnector Interface

All connectors must implement:

```typescript
interface PlatformConnector {
  readonly platform: Platform;
  readonly format: PackageFormat;
  readonly version: string;

  validate(
    metadata: MediaMetadata,
    config?: Partial<ConnectorConfig>
  ): Promise<PlatformValidationResult>;

  generate(
    metadata: MediaMetadata,
    config?: Partial<ConnectorConfig>
  ): Promise<PackageOutput>;

  serialize(
    packageData: PackageOutput,
    format: 'xml' | 'json'
  ): Promise<string>;

  parse(
    packageString: string,
    format: 'xml' | 'json'
  ): Promise<MediaMetadata>;
}
```

## Usage Examples

### Basic Usage

```typescript
import { ConnectorFactory, Platform } from './connectors';

// Get Netflix connector
const netflix = ConnectorFactory.getInstance(Platform.NETFLIX);

// Validate metadata
const validation = await netflix.validate(metadata);

if (validation.valid) {
  // Generate IMF package
  const imfPackage = await netflix.generate(metadata);

  // Serialize to XML
  const xml = await netflix.serialize(imfPackage, 'xml');

  // Save to file
  fs.writeFileSync('netflix-package.xml', xml);
}
```

### Multi-Platform Validation

```typescript
import { validateMultiPlatform, Platform } from './connectors';

const results = await validateMultiPlatform(metadata, [
  Platform.NETFLIX,
  Platform.AMAZON,
  Platform.DISNEY,
  Platform.FAST_PLUTO
]);

for (const [platform, result] of results) {
  console.log(`${platform}: ${result.valid ? 'PASS' : 'FAIL'}`);

  if (!result.valid) {
    console.error('Errors:', result.errors);
  }

  if (result.warnings.length > 0) {
    console.warn('Warnings:', result.warnings);
  }
}
```

### Multi-Platform Package Generation

```typescript
import { generateMultiPlatform, Platform } from './connectors';

const packages = await generateMultiPlatform(metadata, [
  Platform.NETFLIX,
  Platform.AMAZON,
  Platform.FAST_PLUTO
]);

for (const [platform, pkg] of packages) {
  const connector = ConnectorFactory.getInstance(platform);
  const serialized = await connector.serialize(pkg, 'xml');

  fs.writeFileSync(`packages/${platform}.xml`, serialized);
}
```

### Custom Configuration

```typescript
const connector = ConnectorFactory.getInstance(Platform.AMAZON, {
  strictMode: true,
  validateOnGenerate: true,
  includeOptionalFields: true,
  version: '2.0',
  credentials: {
    apiKey: process.env.AMAZON_API_KEY,
    endpoint: 'https://mec.amazon.com/v2'
  }
});

const package = await connector.generate(metadata);
```

### Platform Capabilities

```typescript
import { ConnectorFactory, Platform } from './connectors';

// Check if platform supports HDR
const capabilities = ConnectorFactory.getCapabilities(Platform.NETFLIX);

if (capabilities.supportsHDR) {
  console.log('Netflix supports HDR content');
  console.log('Max resolution:', capabilities.maxResolution);
  console.log('Max bitrate:', capabilities.maxBitrate, 'Mbps');
}

// Check supported codecs
console.log('Video codecs:', capabilities.supportedVideoCodecs);
console.log('Audio codecs:', capabilities.supportedAudioCodecs);
```

## Implementation Guide

### Creating a New Connector

1. **Create connector file** (e.g., `netflix-imf.ts`)

```typescript
import {
  PlatformConnector,
  Platform,
  PackageFormat,
  MediaMetadata,
  PlatformValidationResult,
  IMFPackage,
  ConnectorConfig
} from './types';

export class NetflixIMFConnector implements PlatformConnector {
  readonly platform = Platform.NETFLIX;
  readonly format: PackageFormat = 'imf';
  readonly version = '1.1';

  private config: Partial<ConnectorConfig>;

  constructor(config?: Partial<ConnectorConfig>) {
    this.config = {
      strictMode: false,
      validateOnGenerate: true,
      ...config
    };
  }

  async validate(
    metadata: MediaMetadata,
    config?: Partial<ConnectorConfig>
  ): Promise<PlatformValidationResult> {
    const errors = [];
    const warnings = [];

    // Netflix-specific validation rules
    if (!metadata.eidr) {
      errors.push({
        field: 'eidr',
        message: 'EIDR is required for Netflix',
        severity: 'critical' as const,
        platformRequirement: 'Netflix requires EIDR for all content'
      });
    }

    if (!metadata.rating) {
      errors.push({
        field: 'rating',
        message: 'Content rating is required',
        severity: 'error' as const
      });
    }

    // Check technical requirements
    const technicalChecks = {
      videoCodec: true,
      audioCodec: true,
      resolution: metadata.resolution ? ['4K', '1080p'].includes(metadata.resolution) : false,
      bitrate: true,
      duration: metadata.duration ? metadata.duration > 0 : false
    };

    const complianceScore = this.calculateCompliance(errors, warnings, technicalChecks);

    return {
      platform: this.platform,
      format: this.format,
      valid: errors.length === 0,
      errors,
      warnings,
      technicalChecks,
      complianceScore,
      validatedAt: new Date()
    };
  }

  async generate(
    metadata: MediaMetadata,
    config?: Partial<ConnectorConfig>
  ): Promise<IMFPackage> {
    // Validate first if enabled
    if (this.config.validateOnGenerate || config?.validateOnGenerate) {
      const validation = await this.validate(metadata, config);
      if (!validation.valid) {
        throw new Error(`Validation failed: ${validation.errors.map(e => e.message).join(', ')}`);
      }
    }

    // Generate IMF package structure
    const imfPackage: IMFPackage = {
      format: 'imf',
      version: this.version,
      assetMap: {
        id: `urn:uuid:${metadata.id}`,
        annotation: metadata.title,
        creator: 'Nexus-UMMID',
        issueDate: new Date().toISOString(),
        assets: []
      },
      packingList: {
        id: `urn:uuid:${metadata.id}-pl`,
        essenceDescriptors: [],
        segmentList: []
      },
      compositionPlaylist: {
        id: `urn:uuid:${metadata.id}-cpl`,
        editRate: '24000/1001',
        virtualTracks: []
      },
      metadata: {
        coreMetadata: metadata
      }
    };

    return imfPackage;
  }

  async serialize(packageData: IMFPackage, format: 'xml' | 'json'): Promise<string> {
    if (format === 'json') {
      return JSON.stringify(packageData, null, 2);
    }

    // Convert to IMF XML (SMPTE ST 2067)
    return this.toIMFXML(packageData);
  }

  async parse(packageString: string, format: 'xml' | 'json'): Promise<MediaMetadata> {
    // Parse IMF package back to metadata
    if (format === 'json') {
      const pkg = JSON.parse(packageString) as IMFPackage;
      return pkg.metadata.coreMetadata;
    }

    // Parse IMF XML
    return this.parseIMFXML(packageString);
  }

  private calculateCompliance(errors: any[], warnings: any[], checks: any): number {
    const errorWeight = errors.length * 15;
    const warningWeight = warnings.length * 5;
    const technicalWeight = Object.values(checks).filter(v => !v).length * 10;

    return Math.max(0, 100 - errorWeight - warningWeight - technicalWeight);
  }

  private toIMFXML(packageData: IMFPackage): string {
    // Implement IMF XML serialization (SMPTE ST 2067-2, 2067-3)
    throw new Error('IMF XML serialization not yet implemented');
  }

  private parseIMFXML(xml: string): MediaMetadata {
    // Implement IMF XML parsing
    throw new Error('IMF XML parsing not yet implemented');
  }
}
```

2. **Register connector in `index.ts`**

```typescript
import { NetflixIMFConnector } from './netflix-imf';

ConnectorFactory.register(Platform.NETFLIX, NetflixIMFConnector);
```

## Validation Rules by Platform

### Netflix (IMF)

**Required Fields:**
- EIDR ID
- Title
- Content rating
- Duration
- Language
- Synopsis (min 50 characters)
- Genres (at least 1)

**Technical Requirements:**
- Video: H.264/H.265, up to 4K, max 25 Mbps
- Audio: AAC/AC3/EAC3, Dolby Atmos supported
- Subtitles: TTML/IMSC1 format

### Amazon (MEC)

**Required Fields:**
- Content ID
- Title
- Provider name
- Content rating
- Territories
- Duration

**Technical Requirements:**
- Video: H.264/H.265, up to 4K, max 20 Mbps
- Audio: AAC/AC3/EAC3
- MD5/SHA256 checksums for all assets

### FAST Channels (MRSS)

**Required Fields:**
- GUID
- Title
- Description
- Publication date
- Media content URL
- Category/Genre

**Technical Requirements:**
- Video: H.264, up to 1080p, max 8-10 Mbps
- Audio: AAC
- No HDR/Dolby support

## Testing

```typescript
import { ConnectorFactory, Platform } from './connectors';

describe('Platform Connectors', () => {
  it('should create Netflix connector', () => {
    const connector = ConnectorFactory.getInstance(Platform.NETFLIX);
    expect(connector.platform).toBe(Platform.NETFLIX);
    expect(connector.format).toBe('imf');
  });

  it('should validate metadata', async () => {
    const connector = ConnectorFactory.getInstance(Platform.NETFLIX);
    const result = await connector.validate(sampleMetadata);

    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it('should generate IMF package', async () => {
    const connector = ConnectorFactory.getInstance(Platform.NETFLIX);
    const package = await connector.generate(sampleMetadata);

    expect(package.format).toBe('imf');
    expect(package.assetMap).toBeDefined();
  });
});
```

## Next Steps

1. **Implement Netflix IMF Connector** (`netflix-imf.ts`)
   - IMF XML generation (SMPTE ST 2067-2, 2067-3)
   - Asset map, packing list, composition playlist
   - EIDR integration

2. **Implement Amazon MEC Connector** (`amazon-mec.ts`)
   - MEC XML/JSON manifest generation
   - Checksum calculation
   - Delivery specification formatting

3. **Implement FAST MRSS Connector** (`fast-mrss.ts`)
   - Media RSS 2.0 feed generation
   - Multi-channel support
   - Ad insertion markers

4. **Add Serialization Libraries**
   - XML builder (xml2js, fast-xml-parser)
   - JSON schema validation (ajv)
   - SMPTE IMF validators

5. **Integration Testing**
   - Platform-specific validation tests
   - Round-trip serialization tests
   - Performance benchmarks

## References

- [SMPTE ST 2067 (IMF)](https://www.smpte.org/standards/document-index/st-2067)
- [Amazon MEC Specification](https://developer.amazon.com/docs/video-submission/mec-spec.html)
- [Media RSS 2.0 Specification](http://www.rssboard.org/media-rss)
- [EIDR Registry](https://www.eidr.org/)
- [MovieLabs Digital Distribution Framework](https://movielabs.com/md/)

## License

MIT - Nexus-UMMID Metadata API
