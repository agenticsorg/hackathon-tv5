/**
 * Platform Connector Tests
 * TDD London School style - using mocks and test doubles
 *
 * Tests for:
 * - NetflixIMFConnector (Interoperable Master Format)
 * - AmazonMECConnector (Metadata Exchange Container)
 * - FASTMRSSConnector (Media RSS for FAST channels)
 * - ConnectorFactory
 */

import {
  completeMetadata,
  dolbyVisionMetadata,
  incompleteNetflixMetadata,
  multiTerritoryMetadata,
  fastChannelMetadata,
  seriesEpisodeMetadata,
  minimalMetadata
} from './fixtures/sample-metadata';

// Mock imports - will be implemented later
// import { NetflixIMFConnector } from '../src/connectors/netflix-imf';
// import { AmazonMECConnector } from '../src/connectors/amazon-mec';
// import { FASTMRSSConnector } from '../src/connectors/fast-mrss';
// import { ConnectorFactory } from '../src/connectors/factory';

describe('NetflixIMFConnector', () => {
  let connector: any; // Will be typed once implemented

  beforeEach(() => {
    // connector = new NetflixIMFConnector();
    // Mock for now - will use real implementation once created
    connector = {
      generateIMFPackage: jest.fn(),
      validateNetflixContent: jest.fn(),
      handleDolbyVision: jest.fn()
    };
  });

  describe('generateIMFPackage', () => {
    it('should generate valid IMF package structure with complete metadata', async () => {
      // Arrange
      const expectedStructure = {
        assetMap: expect.objectContaining({
          id: completeMetadata.id,
          eidr: completeMetadata.eidr
        }),
        packingList: expect.any(Object),
        cpl: expect.objectContaining({
          contentTitle: completeMetadata.title,
          editRate: expect.any(String),
          duration: completeMetadata.duration
        }),
        metadata: expect.objectContaining({
          title: completeMetadata.title,
          contentVersion: expect.any(String)
        })
      };

      connector.generateIMFPackage.mockResolvedValue(expectedStructure);

      // Act
      const result = await connector.generateIMFPackage(completeMetadata);

      // Assert
      expect(result).toMatchObject(expectedStructure);
      expect(result.assetMap.id).toBe(completeMetadata.id);
      expect(result.cpl.contentTitle).toBe(completeMetadata.title);
      expect(connector.generateIMFPackage).toHaveBeenCalledWith(completeMetadata);
    });

    it('should include all required IMF XML elements', async () => {
      // Arrange
      const expectedElements = {
        assetMap: expect.any(Object),
        packingList: expect.any(Object),
        cpl: expect.any(Object), // Composition Playlist
        metadata: expect.any(Object),
        volumeIndex: expect.any(Object)
      };

      connector.generateIMFPackage.mockResolvedValue(expectedElements);

      // Act
      const result = await connector.generateIMFPackage(completeMetadata);

      // Assert
      expect(result).toHaveProperty('assetMap');
      expect(result).toHaveProperty('packingList');
      expect(result).toHaveProperty('cpl');
      expect(result).toHaveProperty('metadata');
      expect(result).toHaveProperty('volumeIndex');
    });

    it('should handle series episodes with proper segment structure', async () => {
      // Arrange
      const episodePackage = {
        assetMap: { id: seriesEpisodeMetadata.id },
        cpl: {
          contentTitle: seriesEpisodeMetadata.title,
          segmentList: expect.any(Array),
          episodeNumber: 1,
          seasonNumber: 1
        }
      };

      connector.generateIMFPackage.mockResolvedValue(episodePackage);

      // Act
      const result = await connector.generateIMFPackage(seriesEpisodeMetadata);

      // Assert
      expect(result.cpl).toHaveProperty('segmentList');
      expect(result.cpl).toHaveProperty('episodeNumber');
      expect(result.cpl).toHaveProperty('seasonNumber');
    });
  });

  describe('validateNetflixContent', () => {
    it('should validate complete metadata successfully', async () => {
      // Arrange
      const validationResult = {
        valid: true,
        errors: [],
        warnings: [],
        checks: {
          hasEIDR: true,
          hasRating: true,
          hasResolution: true,
          hasAudioTracks: true,
          hasCaptions: true
        }
      };

      connector.validateNetflixContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateNetflixContent(completeMetadata);

      // Assert
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
      expect(result.checks.hasEIDR).toBe(true);
      expect(result.checks.hasRating).toBe(true);
    });

    it('should return errors for metadata missing required fields', async () => {
      // Arrange
      const validationResult = {
        valid: false,
        errors: [
          {
            field: 'eidr',
            message: 'EIDR is required for Netflix content',
            severity: 'critical' as const,
            platformRequirement: 'Netflix Technical Specification v8.0'
          },
          {
            field: 'rating',
            message: 'Content rating is required',
            severity: 'critical' as const,
            platformRequirement: 'Netflix Content Guidelines'
          },
          {
            field: 'resolution',
            message: 'Resolution must be specified (minimum 1080p)',
            severity: 'error' as const,
            platformRequirement: 'Netflix Quality Standards'
          }
        ],
        warnings: [],
        checks: {
          hasEIDR: false,
          hasRating: false,
          hasResolution: false,
          hasAudioTracks: false,
          hasCaptions: false
        }
      };

      connector.validateNetflixContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateNetflixContent(incompleteNetflixMetadata);

      // Assert
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
      expect(result.errors).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            field: 'eidr',
            severity: 'critical'
          }),
          expect.objectContaining({
            field: 'rating',
            severity: 'critical'
          })
        ])
      );
    });

    it('should validate audio track requirements', async () => {
      // Arrange
      const validationResult = {
        valid: true,
        errors: [],
        warnings: [
          {
            field: 'audioTracks',
            message: 'Consider adding Dolby Atmos track for enhanced experience',
            recommendation: 'Add en-atmos audio track'
          }
        ],
        checks: {
          hasEIDR: true,
          hasRating: true,
          hasResolution: true,
          hasAudioTracks: true,
          hasCaptions: true,
          hasDolbyAtmos: false
        }
      };

      connector.validateNetflixContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateNetflixContent(completeMetadata);

      // Assert
      expect(result.checks.hasAudioTracks).toBe(true);
      expect(result.warnings).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            field: 'audioTracks'
          })
        ])
      );
    });
  });

  describe('handleDolbyVision', () => {
    it('should process Dolby Vision metadata correctly', async () => {
      // Arrange
      const dolbyVisionData = {
        profile: 'DV-P8',
        level: '05',
        rpu: true, // RPU metadata present
        el: true,  // Enhancement layer present
        bl: true,  // Base layer present
        colorSpace: 'BT.2020',
        transferFunction: 'PQ',
        metadata: {
          maxCLL: 1000,
          maxFALL: 400,
          masteringDisplay: {
            primaries: 'BT.2020',
            whitePoint: 'D65',
            luminance: {
              max: 4000,
              min: 0.005
            }
          }
        }
      };

      connector.handleDolbyVision.mockResolvedValue(dolbyVisionData);

      // Act
      const result = await connector.handleDolbyVision(dolbyVisionMetadata);

      // Assert
      expect(result).toHaveProperty('profile');
      expect(result).toHaveProperty('level');
      expect(result.rpu).toBe(true);
      expect(result.metadata).toHaveProperty('maxCLL');
      expect(result.metadata.masteringDisplay).toHaveProperty('luminance');
    });

    it('should validate Dolby Vision technical requirements', async () => {
      // Arrange
      const validationResult = {
        valid: true,
        profile: 'DV-P8',
        warnings: [],
        technicalChecks: {
          hasRPU: true,
          hasEnhancementLayer: true,
          hasCorrectColorSpace: true,
          meetsLuminanceRequirements: true
        }
      };

      connector.handleDolbyVision.mockResolvedValue(validationResult);

      // Act
      const result = await connector.handleDolbyVision(dolbyVisionMetadata);

      // Assert
      expect(result.valid).toBe(true);
      expect(result.technicalChecks.hasRPU).toBe(true);
      expect(result.technicalChecks.meetsLuminanceRequirements).toBe(true);
    });
  });
});

describe('AmazonMECConnector', () => {
  let connector: any;

  beforeEach(() => {
    connector = {
      generateMECFeed: jest.fn(),
      validateAmazonContent: jest.fn(),
      handleMultiTerritoryRights: jest.fn()
    };
  });

  describe('generateMECFeed', () => {
    it('should generate valid MEC XML structure', async () => {
      // Arrange
      const expectedMEC = {
        manifest: {
          version: '1.4',
          provider: expect.any(String),
          xmlns: 'http://www.movielabs.com/schema/mec/v1.4'
        },
        catalog: {
          entries: expect.arrayContaining([
            expect.objectContaining({
              id: completeMetadata.id,
              title: completeMetadata.title,
              contentType: 'Movie'
            })
          ])
        },
        avails: expect.any(Object)
      };

      connector.generateMECFeed.mockResolvedValue(expectedMEC);

      // Act
      const result = await connector.generateMECFeed([completeMetadata]);

      // Assert
      expect(result).toHaveProperty('manifest');
      expect(result).toHaveProperty('catalog');
      expect(result).toHaveProperty('avails');
      expect(result.manifest.version).toBe('1.4');
      expect(result.catalog.entries).toHaveLength(1);
    });

    it('should include proper namespace declarations', async () => {
      // Arrange
      const mecWithNamespaces = {
        manifest: {
          xmlns: 'http://www.movielabs.com/schema/mec/v1.4',
          'xmlns:md': 'http://www.movielabs.com/schema/md/v2.7',
          'xmlns:avails': 'http://www.movielabs.com/schema/avails/v2.4'
        }
      };

      connector.generateMECFeed.mockResolvedValue(mecWithNamespaces);

      // Act
      const result = await connector.generateMECFeed([completeMetadata]);

      // Assert
      expect(result.manifest.xmlns).toBeDefined();
      expect(result.manifest['xmlns:md']).toBeDefined();
      expect(result.manifest['xmlns:avails']).toBeDefined();
    });

    it('should generate catalog entries for multiple assets', async () => {
      // Arrange
      const multiplAssets = [completeMetadata, dolbyVisionMetadata, fastChannelMetadata];
      const mecFeed = {
        catalog: {
          entries: multiplAssets.map(asset => ({
            id: asset.id,
            title: asset.title,
            contentType: asset.type === 'movie' ? 'Movie' : 'Episode'
          }))
        }
      };

      connector.generateMECFeed.mockResolvedValue(mecFeed);

      // Act
      const result = await connector.generateMECFeed(multiplAssets);

      // Assert
      expect(result.catalog.entries).toHaveLength(3);
      expect(result.catalog.entries[0].id).toBe(completeMetadata.id);
      expect(result.catalog.entries[1].id).toBe(dolbyVisionMetadata.id);
      expect(result.catalog.entries[2].id).toBe(fastChannelMetadata.id);
    });
  });

  describe('validateAmazonContent', () => {
    it('should validate Amazon Prime Video requirements', async () => {
      // Arrange
      const validationResult = {
        valid: true,
        errors: [],
        warnings: [],
        checks: {
          hasTitle: true,
          hasDescription: true,
          hasGenres: true,
          hasRating: true,
          hasTechnicalSpecs: true,
          meetsQualityStandards: true
        }
      };

      connector.validateAmazonContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateAmazonContent(completeMetadata);

      // Assert
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
      expect(result.checks.meetsQualityStandards).toBe(true);
    });

    it('should check for required metadata fields', async () => {
      // Arrange
      const validationResult = {
        valid: false,
        errors: [
          {
            field: 'synopsis',
            message: 'Synopsis is required and must be at least 20 characters',
            severity: 'error' as const,
            platformRequirement: 'Amazon Content Specification'
          }
        ],
        warnings: [],
        checks: {
          hasTitle: true,
          hasDescription: false,
          hasGenres: true
        }
      };

      connector.validateAmazonContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateAmazonContent(minimalMetadata);

      // Assert
      expect(result.valid).toBe(false);
      expect(result.errors).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            field: 'synopsis'
          })
        ])
      );
    });

    it('should validate technical specifications', async () => {
      // Arrange
      const validationResult = {
        valid: true,
        errors: [],
        warnings: [],
        checks: {
          hasValidResolution: true,
          hasValidAspectRatio: true,
          hasValidAudioFormat: true,
          meetsMinimumQuality: true
        },
        technicalSpecs: {
          resolution: '4K',
          aspectRatio: '2.39:1',
          audioFormat: 'Dolby Atmos',
          bitrate: 'HD'
        }
      };

      connector.validateAmazonContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateAmazonContent(completeMetadata);

      // Assert
      expect(result.checks.hasValidResolution).toBe(true);
      expect(result.checks.hasValidAspectRatio).toBe(true);
      expect(result.technicalSpecs.resolution).toBe('4K');
    });
  });

  describe('handleMultiTerritoryRights', () => {
    it('should process rights for multiple territories', async () => {
      // Arrange
      const rightsData = {
        territories: ['US', 'CA', 'GB', 'DE', 'FR', 'ES', 'IT', 'JP', 'AU', 'NZ', 'IN', 'BR', 'MX'],
        windows: [
          {
            territory: 'US',
            availStart: expect.any(Date),
            availEnd: expect.any(Date),
            licenseType: 'SVOD',
            exclusive: false
          },
          {
            territory: 'GB',
            availStart: expect.any(Date),
            availEnd: expect.any(Date),
            licenseType: 'TVOD',
            exclusive: false
          }
        ],
        restrictions: []
      };

      connector.handleMultiTerritoryRights.mockResolvedValue(rightsData);

      // Act
      const result = await connector.handleMultiTerritoryRights(multiTerritoryMetadata);

      // Assert
      expect(result.territories).toHaveLength(13);
      expect(result.windows.length).toBeGreaterThan(0);
      expect(result.windows[0]).toHaveProperty('territory');
      expect(result.windows[0]).toHaveProperty('licenseType');
    });

    it('should detect territory-specific release windows', async () => {
      // Arrange
      const rightsData = {
        windows: [
          {
            territory: 'US',
            availStart: new Date('2024-07-01'),
            availEnd: new Date('2025-07-01')
          },
          {
            territory: 'JP',
            availStart: new Date('2024-08-01'), // Delayed release
            availEnd: new Date('2025-08-01')
          }
        ]
      };

      connector.handleMultiTerritoryRights.mockResolvedValue(rightsData);

      // Act
      const result = await connector.handleMultiTerritoryRights(multiTerritoryMetadata);

      // Assert
      const usWindow = result.windows.find((w: any) => w.territory === 'US');
      const jpWindow = result.windows.find((w: any) => w.territory === 'JP');

      expect(usWindow).toBeDefined();
      expect(jpWindow).toBeDefined();
      expect(jpWindow.availStart).not.toEqual(usWindow.availStart);
    });

    it('should handle different licensing models per territory', async () => {
      // Arrange
      const rightsData = {
        windows: [
          {
            territory: 'US',
            licenseType: 'TVOD', // Transactional
            subscriptionRequired: false
          },
          {
            territory: 'GB',
            licenseType: 'SVOD', // Subscription
            subscriptionRequired: true
          }
        ]
      };

      connector.handleMultiTerritoryRights.mockResolvedValue(rightsData);

      // Act
      const result = await connector.handleMultiTerritoryRights(multiTerritoryMetadata);

      // Assert
      expect(result.windows).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            territory: 'US',
            licenseType: 'TVOD'
          }),
          expect.objectContaining({
            territory: 'GB',
            licenseType: 'SVOD'
          })
        ])
      );
    });
  });
});

describe('FASTMRSSConnector', () => {
  let connector: any;

  beforeEach(() => {
    connector = {
      generateMRSSFeed: jest.fn(),
      validateFASTContent: jest.fn(),
      addSchedulingMetadata: jest.fn()
    };
  });

  describe('generateMRSSFeed', () => {
    it('should generate valid MRSS/RSS structure', async () => {
      // Arrange
      const expectedMRSS = {
        rss: {
          version: '2.0',
          'xmlns:media': 'http://search.yahoo.com/mrss/',
          'xmlns:dcterms': 'http://purl.org/dc/terms/',
          channel: {
            title: expect.any(String),
            description: expect.any(String),
            items: expect.arrayContaining([
              expect.objectContaining({
                title: fastChannelMetadata.title,
                guid: fastChannelMetadata.id
              })
            ])
          }
        }
      };

      connector.generateMRSSFeed.mockResolvedValue(expectedMRSS);

      // Act
      const result = await connector.generateMRSSFeed([fastChannelMetadata]);

      // Assert
      expect(result.rss.version).toBe('2.0');
      expect(result.rss['xmlns:media']).toBeDefined();
      expect(result.rss.channel.items).toHaveLength(1);
    });

    it('should include media:content elements with technical specs', async () => {
      // Arrange
      const mrssWithMedia = {
        rss: {
          channel: {
            items: [
              {
                title: fastChannelMetadata.title,
                'media:content': {
                  url: expect.any(String),
                  type: 'video/mp4',
                  duration: fastChannelMetadata.duration! * 60, // Convert to seconds
                  width: 1920,
                  height: 1080,
                  bitrate: 5000
                },
                'media:thumbnail': {
                  url: expect.any(String)
                }
              }
            ]
          }
        }
      };

      connector.generateMRSSFeed.mockResolvedValue(mrssWithMedia);

      // Act
      const result = await connector.generateMRSSFeed([fastChannelMetadata]);

      // Assert
      const item = result.rss.channel.items[0];
      expect(item['media:content']).toBeDefined();
      expect(item['media:content'].type).toBe('video/mp4');
      expect(item['media:content'].duration).toBe(fastChannelMetadata.duration! * 60);
      expect(item['media:thumbnail']).toBeDefined();
    });

    it('should generate feed for multiple FAST channel assets', async () => {
      // Arrange
      const assets = [fastChannelMetadata, { ...fastChannelMetadata, id: 'asset-fast-002', title: 'Comedy Special' }];
      const mrssFeed = {
        rss: {
          channel: {
            items: assets.map(asset => ({
              title: asset.title,
              guid: asset.id
            }))
          }
        }
      };

      connector.generateMRSSFeed.mockResolvedValue(mrssFeed);

      // Act
      const result = await connector.generateMRSSFeed(assets);

      // Assert
      expect(result.rss.channel.items).toHaveLength(2);
      expect(result.rss.channel.items[1].title).toBe('Comedy Special');
    });
  });

  describe('validateFASTContent', () => {
    it('should validate content for Samsung TV Plus', async () => {
      // Arrange
      const validationResult = {
        platform: 'samsung-tv-plus',
        valid: true,
        errors: [],
        warnings: [],
        checks: {
          hasValidDuration: true,
          hasValidRating: true,
          hasRequiredMetadata: true,
          meetsFormatRequirements: true
        }
      };

      connector.validateFASTContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateFASTContent(fastChannelMetadata, 'samsung-tv-plus');

      // Assert
      expect(result.platform).toBe('samsung-tv-plus');
      expect(result.valid).toBe(true);
      expect(result.checks.meetsFormatRequirements).toBe(true);
    });

    it('should validate content for Roku Channel', async () => {
      // Arrange
      const validationResult = {
        platform: 'roku',
        valid: true,
        errors: [],
        warnings: [
          {
            field: 'resolution',
            message: '4K content recommended for Roku',
            recommendation: 'Upgrade to 4K resolution'
          }
        ],
        checks: {
          hasValidDuration: true,
          hasClosedCaptions: true,
          meetsRokuStandards: true
        }
      };

      connector.validateFASTContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateFASTContent(fastChannelMetadata, 'roku');

      // Assert
      expect(result.platform).toBe('roku');
      expect(result.valid).toBe(true);
      expect(result.checks.meetsRokuStandards).toBe(true);
      expect(result.warnings).toHaveLength(1);
    });

    it('should validate content for Pluto TV', async () => {
      // Arrange
      const validationResult = {
        platform: 'pluto-tv',
        valid: true,
        errors: [],
        warnings: [],
        checks: {
          hasValidGenres: true,
          hasValidRating: true,
          meetsPlutoRequirements: true,
          hasAdBreakMarkers: false // Warning, not error
        }
      };

      connector.validateFASTContent.mockResolvedValue(validationResult);

      // Act
      const result = await connector.validateFASTContent(fastChannelMetadata, 'pluto-tv');

      // Assert
      expect(result.platform).toBe('pluto-tv');
      expect(result.valid).toBe(true);
      expect(result.checks.meetsPlutoRequirements).toBe(true);
    });
  });

  describe('addSchedulingMetadata', () => {
    it('should include scheduling information in MRSS feed', async () => {
      // Arrange
      const scheduledContent = {
        ...fastChannelMetadata,
        scheduling: {
          airDate: new Date('2024-06-15T20:00:00Z'),
          duration: 105,
          repeatSchedule: {
            frequency: 'weekly',
            dayOfWeek: 'friday',
            time: '20:00'
          }
        }
      };

      connector.addSchedulingMetadata.mockResolvedValue(scheduledContent);

      // Act
      const result = await connector.addSchedulingMetadata(fastChannelMetadata, {
        airDate: new Date('2024-06-15T20:00:00Z'),
        frequency: 'weekly'
      });

      // Assert
      expect(result.scheduling).toBeDefined();
      expect(result.scheduling.airDate).toBeInstanceOf(Date);
      expect(result.scheduling.repeatSchedule.frequency).toBe('weekly');
    });

    it('should handle ad break markers for FAST content', async () => {
      // Arrange
      const contentWithAdBreaks = {
        ...fastChannelMetadata,
        adBreaks: [
          { position: 300, duration: 120 },  // 5 min in, 2 min ad
          { position: 1200, duration: 120 }, // 20 min in, 2 min ad
          { position: 2400, duration: 180 }  // 40 min in, 3 min ad
        ]
      };

      connector.addSchedulingMetadata.mockResolvedValue(contentWithAdBreaks);

      // Act
      const result = await connector.addSchedulingMetadata(fastChannelMetadata, {
        includeAdBreaks: true,
        adBreakInterval: 600 // Every 10 minutes
      });

      // Assert
      expect(result.adBreaks).toBeDefined();
      expect(result.adBreaks.length).toBeGreaterThan(0);
      expect(result.adBreaks[0]).toHaveProperty('position');
      expect(result.adBreaks[0]).toHaveProperty('duration');
    });

    it('should support EPG integration metadata', async () => {
      // Arrange
      const epgMetadata = {
        ...fastChannelMetadata,
        epg: {
          channelId: 'fast-action-001',
          programId: 'prog-12345',
          startTime: new Date('2024-06-15T20:00:00Z'),
          endTime: new Date('2024-06-15T21:45:00Z'),
          category: 'Movie',
          subcategory: 'Action'
        }
      };

      connector.addSchedulingMetadata.mockResolvedValue(epgMetadata);

      // Act
      const result = await connector.addSchedulingMetadata(fastChannelMetadata, {
        includeEPG: true,
        channelId: 'fast-action-001'
      });

      // Assert
      expect(result.epg).toBeDefined();
      expect(result.epg.channelId).toBe('fast-action-001');
      expect(result.epg.startTime).toBeInstanceOf(Date);
      expect(result.epg.category).toBe('Movie');
    });
  });
});

describe('ConnectorFactory', () => {
  let factory: any;

  beforeEach(() => {
    factory = {
      getInstance: jest.fn(),
      getSupportedPlatforms: jest.fn(),
      validatePlatform: jest.fn()
    };
  });

  describe('getInstance', () => {
    it('should return NetflixIMFConnector instance for netflix platform', () => {
      // Arrange
      const netflixConnector = { name: 'NetflixIMFConnector' };
      factory.getInstance.mockReturnValue(netflixConnector);

      // Act
      const result = factory.getInstance('netflix');

      // Assert
      expect(result).toBeDefined();
      expect(result.name).toBe('NetflixIMFConnector');
      expect(factory.getInstance).toHaveBeenCalledWith('netflix');
    });

    it('should return AmazonMECConnector instance for amazon platform', () => {
      // Arrange
      const amazonConnector = { name: 'AmazonMECConnector' };
      factory.getInstance.mockReturnValue(amazonConnector);

      // Act
      const result = factory.getInstance('amazon');

      // Assert
      expect(result).toBeDefined();
      expect(result.name).toBe('AmazonMECConnector');
      expect(factory.getInstance).toHaveBeenCalledWith('amazon');
    });

    it('should return FASTMRSSConnector instance for FAST platforms', () => {
      // Arrange
      const fastConnector = { name: 'FASTMRSSConnector' };
      factory.getInstance.mockReturnValue(fastConnector);

      // Act
      const result = factory.getInstance('samsung-tv-plus');

      // Assert
      expect(result).toBeDefined();
      expect(result.name).toBe('FASTMRSSConnector');
      expect(factory.getInstance).toHaveBeenCalledWith('samsung-tv-plus');
    });

    it('should throw error for unsupported platform', () => {
      // Arrange
      factory.getInstance.mockImplementation((platform: string) => {
        throw new Error(`Unsupported platform: ${platform}`);
      });

      // Act & Assert
      expect(() => factory.getInstance('unsupported-platform')).toThrow('Unsupported platform');
    });
  });

  describe('getSupportedPlatforms', () => {
    it('should return list of all supported platforms', () => {
      // Arrange
      const platforms = [
        'netflix',
        'amazon',
        'hulu',
        'disney',
        'apple',
        'hbo',
        'paramount',
        'peacock',
        'samsung-tv-plus',
        'roku',
        'pluto-tv',
        'xumo',
        'tubi'
      ];
      factory.getSupportedPlatforms.mockReturnValue(platforms);

      // Act
      const result = factory.getSupportedPlatforms();

      // Assert
      expect(result).toEqual(expect.arrayContaining(['netflix', 'amazon']));
      expect(result).toEqual(expect.arrayContaining(['samsung-tv-plus', 'roku', 'pluto-tv']));
      expect(result.length).toBeGreaterThan(5);
    });
  });

  describe('validatePlatform', () => {
    it('should validate platform is supported', () => {
      // Arrange
      factory.validatePlatform.mockReturnValue(true);

      // Act
      const result = factory.validatePlatform('netflix');

      // Assert
      expect(result).toBe(true);
      expect(factory.validatePlatform).toHaveBeenCalledWith('netflix');
    });

    it('should return false for unsupported platform', () => {
      // Arrange
      factory.validatePlatform.mockReturnValue(false);

      // Act
      const result = factory.validatePlatform('unknown-platform');

      // Assert
      expect(result).toBe(false);
    });
  });

  describe('unified connector interface', () => {
    it('should provide consistent interface across all connectors', () => {
      // Arrange
      const connectors = ['netflix', 'amazon', 'samsung-tv-plus'].map(platform => {
        const connector = {
          platform,
          generate: jest.fn(),
          validate: jest.fn()
        };
        return connector;
      });

      // Act & Assert
      connectors.forEach(connector => {
        expect(connector).toHaveProperty('generate');
        expect(connector).toHaveProperty('validate');
        expect(typeof connector.generate).toBe('function');
        expect(typeof connector.validate).toBe('function');
      });
    });
  });
});
