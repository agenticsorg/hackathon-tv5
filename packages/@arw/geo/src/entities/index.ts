/**
 * ARW-GEO-5: Entity Enrichment
 * Provides +30-40% visibility improvement through entity linking
 */

import type { Entity, EntityType, EntityRelationship, EntityGraph } from '../types';

export class EntityEnrichment {
  /**
   * Extract entities from markdown content
   */
  extractFromMarkdown(content: string): Entity[] {
    const entities: Entity[] = [];

    // Match entities blocks
    const entityPattern = /```entities\s*\n(.*?)\n```/gs;
    const matches = Array.from(content.matchAll(entityPattern));

    for (const match of matches) {
      try {
        const entityArray = JSON.parse(match[1]) as Entity[];
        entities.push(...entityArray);
      } catch (error) {
        console.warn('Failed to parse entities:', error);
      }
    }

    return entities;
  }

  /**
   * Extract entity mentions from text using regex patterns
   * (Simplified version - in production would use NLP library)
   */
  extractFromText(text: string): Array<{
    text: string;
    type: EntityType;
    position: number;
  }> {
    const entities: Array<{ text: string; type: EntityType; position: number }> = [];

    // Pattern for organizations (capitalized consecutive words)
    const orgPattern = /\b([A-Z][a-z]+(?: [A-Z][a-z]+)* (?:Inc|Corp|Corporation|LLC|Ltd|Company))\b/g;
    const orgMatches = Array.from(text.matchAll(orgPattern));
    for (const match of orgMatches) {
      entities.push({
        text: match[1],
        type: 'Organization',
        position: match.index || 0
      });
    }

    // Pattern for products (all caps or capitalized with version numbers)
    const productPattern = /\b([A-Z][A-Za-z]+(?: [A-Z][A-Za-z]+)?(?: \d+(?:\.\d+)?)?)\b/g;
    const productMatches = Array.from(text.matchAll(productPattern));
    for (const match of productMatches) {
      if (!entities.some(e => e.text === match[1])) {
        entities.push({
          text: match[1],
          type: 'Product',
          position: match.index || 0
        });
      }
    }

    return entities;
  }

  /**
   * Link entity to Wikidata (mock implementation)
   */
  async linkToWikidata(_entityName: string, _entityType: EntityType): Promise<string | null> {
    // In production, this would call the Wikidata API
    // For now, return a mock ID
    return `Q${Math.floor(Math.random() * 1000000)}`;
  }

  /**
   * Validate entity structure
   */
  validate(entity: Entity): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!entity.id) errors.push('Missing entity ID');
    if (!entity.type) errors.push('Missing entity type');
    if (!entity.name) errors.push('Missing entity name');
    if (!entity.schema_type) errors.push('Missing schema type');

    // Validate confidence score
    if (entity.confidence !== undefined) {
      if (entity.confidence < 0 || entity.confidence > 1) {
        errors.push('Confidence must be between 0 and 1');
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  /**
   * Build entity graph from entities and relationships
   */
  buildGraph(entities: Entity[], relationships: EntityRelationship[]): EntityGraph {
    return {
      entities,
      relationships
    };
  }

  /**
   * Convert entity to Schema.org
   */
  toSchemaOrg(entity: Entity): object {
    const baseSchema: any = {
      '@id': entity.wikidata_id ? `http://www.wikidata.org/entity/${entity.wikidata_id}` : undefined,
      '@type': entity.type,
      name: entity.name,
      sameAs: entity.dbpedia_url,
      description: entity.description,
      ...entity.properties
    };

    return baseSchema;
  }
}
