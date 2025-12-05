/**
 * LLM Integration Layer for Enhanced GEO Generation
 * Supports Anthropic Claude and OpenAI GPT models
 */

import type { LLMConfig, Citation, Entity, Quotation } from '../types';

export class LLMIntegration {
  private config: LLMConfig;
  private client: any;

  constructor(config: LLMConfig) {
    this.config = config;
    this.initializeClient();
  }

  private initializeClient(): void {
    if (this.config.provider === 'anthropic') {
      try {
        // Dynamic import to avoid hard dependency
        const Anthropic = require('@anthropic-ai/sdk');
        this.client = new Anthropic.default({
          apiKey: this.config.apiKey
        });
      } catch (error) {
        throw new Error('Anthropic SDK not installed. Run: npm install @anthropic-ai/sdk');
      }
    } else if (this.config.provider === 'openai') {
      try {
        const { OpenAI } = require('openai');
        this.client = new OpenAI({
          apiKey: this.config.apiKey
        });
      } catch (error) {
        throw new Error('OpenAI SDK not installed. Run: npm install openai');
      }
    }
  }

  /**
   * Strip markdown code blocks from LLM response
   */
  private stripMarkdown(text: string): string {
    // Remove ```json and ``` wrappers
    let cleaned = text.trim();

    // Remove opening ```json, ```JSON, or just ```
    cleaned = cleaned.replace(/^```(?:json|JSON)?\s*\n?/m, '');

    // Remove closing ```
    cleaned = cleaned.replace(/\n?```\s*$/m, '');

    return cleaned.trim();
  }

  /**
   * Enhance citations using AI
   */
  async enhanceCitations(
    content: string,
    options?: { generateMissing?: boolean; verifyAccuracy?: boolean }
  ): Promise<Citation[]> {
    const prompt = this.buildCitationPrompt(content, options);
    const response = await this.callLLM(prompt);

    try {
      const cleanedResponse = this.stripMarkdown(response);
      const citations = JSON.parse(cleanedResponse) as Citation[];
      return citations;
    } catch (error) {
      console.warn('Failed to parse AI-generated citations:', error);
      return [];
    }
  }

  /**
   * Extract entities using AI
   */
  async extractEntities(
    content: string,
    options?: { linkToWikidata?: boolean; includeRelationships?: boolean }
  ): Promise<Entity[]> {
    const prompt = this.buildEntityPrompt(content, options);
    const response = await this.callLLM(prompt);

    try {
      const cleanedResponse = this.stripMarkdown(response);
      const entities = JSON.parse(cleanedResponse) as Entity[];
      return entities;
    } catch (error) {
      console.warn('Failed to parse AI-extracted entities:', error);
      return [];
    }
  }

  /**
   * Generate quotations from interviews or content
   */
  async extractQuotations(content: string): Promise<Quotation[]> {
    const prompt = this.buildQuotationPrompt(content);
    const response = await this.callLLM(prompt);

    try {
      const cleanedResponse = this.stripMarkdown(response);
      const quotations = JSON.parse(cleanedResponse) as Quotation[];
      return quotations;
    } catch (error) {
      console.warn('Failed to parse AI-extracted quotations:', error);
      return [];
    }
  }

  /**
   * Analyze content quality and suggest improvements
   */
  async analyzeQuality(content: string): Promise<{
    score: number;
    issues: string[];
    recommendations: string[];
  }> {
    const prompt = `Analyze the following content for quality, readability, and E-E-A-T signals.
Provide a JSON response with: score (0-1), issues (array of problems), and recommendations (array of improvements).

Content:
${content}

Return only valid JSON without any markdown formatting.`;

    const response = await this.callLLM(prompt);

    try {
      const cleanedResponse = this.stripMarkdown(response);
      return JSON.parse(cleanedResponse);
    } catch (error) {
      console.warn('Failed to parse AI quality analysis:', error);
      return {
        score: 0.5,
        issues: ['Failed to analyze content'],
        recommendations: []
      };
    }
  }

  private async callLLM(prompt: string): Promise<string> {
    if (this.config.provider === 'anthropic') {
      const response = await this.client.messages.create({
        model: this.config.model,
        max_tokens: this.config.maxTokens || 4096,
        temperature: this.config.temperature || 0.7,
        messages: [{ role: 'user', content: prompt }]
      });

      return response.content[0].text;
    } else {
      const response = await this.client.chat.completions.create({
        model: this.config.model,
        max_tokens: this.config.maxTokens || 4096,
        temperature: this.config.temperature || 0.7,
        messages: [{ role: 'user', content: prompt }]
      });

      return response.choices[0].message.content || '';
    }
  }

  private buildCitationPrompt(content: string, options?: any): string {
    return `Analyze the following content and extract or generate structured citations in JSON format.
${options?.generateMissing ? 'Also identify claims that need citations and suggest appropriate sources.' : ''}
${options?.verifyAccuracy ? 'Verify the accuracy of existing citations.' : ''}

Return an array of Citation objects with this structure:
{
  "id": "cite:N",
  "source": "source name",
  "type": "academic|industry_report|government|news|book|website",
  "url": "source URL",
  "date": "YYYY-MM-DD",
  "author": "author name",
  "publisher": "publisher name",
  "authority_score": 0.0-1.0
}

Content:
${content}

Return only valid JSON array without any markdown formatting.`;
  }

  private buildEntityPrompt(content: string, options?: any): string {
    return `Extract named entities from the following content and return them as JSON.
${options?.linkToWikidata ? 'Include Wikidata Q-numbers where possible.' : ''}
${options?.includeRelationships ? 'Include relationships between entities.' : ''}

Return an array of Entity objects with this structure:
{
  "id": "entity-id",
  "type": "Person|Organization|Product|Place|Concept",
  "name": "entity name",
  "wikidata_id": "Q123456",
  "schema_type": "schema.org type",
  "properties": {},
  "confidence": 0.0-1.0
}

Content:
${content}

Return only valid JSON array without any markdown formatting.`;
  }

  private buildQuotationPrompt(content: string): string {
    return `Extract direct quotations from the following content and return them as JSON.
Include speaker information and context.

Return an array of Quotation objects with this structure:
{
  "id": "quote:N",
  "text": "the actual quote",
  "speaker": {
    "name": "speaker name",
    "title": "job title",
    "affiliation": "organization",
    "credentials": ["credential1", "credential2"]
  },
  "date": "YYYY-MM-DD",
  "source": "source name",
  "type": "expert_opinion|case_study|research_finding"
}

Content:
${content}

Return only valid JSON array without any markdown formatting.`;
  }
}
