/**
 * Tests for AST Parser
 */

import { describe, it, expect } from 'vitest';
import { ASTParser } from '../../src/generator/ast-parser';
import { promises as fs } from 'fs';
import path from 'path';
import os from 'os';

describe('ASTParser', () => {
  const parser = new ASTParser();

  async function createTempFile(content: string): Promise<string> {
    const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), 'arw-test-'));
    const filePath = path.join(tmpDir, 'test.tsx');
    await fs.writeFile(filePath, content, 'utf-8');
    return filePath;
  }

  async function cleanup(filePath: string): Promise<void> {
    const dir = path.dirname(filePath);
    await fs.rm(dir, { recursive: true, force: true });
  }

  it('should extract arwConfig from named export', async () => {
    const content = `
      export const arwConfig = {
        priority: 'high',
        purpose: 'test',
        title: 'Test Page'
      };
    `;

    const filePath = await createTempFile(content);
    const config = await parser.extractARWConfig(filePath);
    await cleanup(filePath);

    expect(config).not.toBeNull();
    expect(config?.priority).toBe('high');
    expect(config?.purpose).toBe('test');
    expect(config?.title).toBe('Test Page');
  });

  it('should extract arwConfig with chunks', async () => {
    const content = `
      export const arwConfig = {
        priority: 'normal',
        chunks: [
          { id: 'intro', heading: 'Introduction' },
          { id: 'content', heading: 'Main Content' }
        ]
      };
    `;

    const filePath = await createTempFile(content);
    const config = await parser.extractARWConfig(filePath);
    await cleanup(filePath);

    expect(config).not.toBeNull();
    expect(config?.chunks).toHaveLength(2);
    expect(config?.chunks?.[0].id).toBe('intro');
    expect(config?.chunks?.[1].heading).toBe('Main Content');
  });

  it('should return null for file without arwConfig', async () => {
    const content = `
      export default function Page() {
        return <div>Hello</div>;
      }
    `;

    const filePath = await createTempFile(content);
    const config = await parser.extractARWConfig(filePath);
    await cleanup(filePath);

    expect(config).toBeNull();
  });

  it('should handle TypeScript syntax', async () => {
    const content = `
      interface PageConfig {
        priority: string;
      }

      export const arwConfig: PageConfig = {
        priority: 'high'
      };
    `;

    const filePath = await createTempFile(content);
    const config = await parser.extractARWConfig(filePath);
    await cleanup(filePath);

    expect(config).not.toBeNull();
    expect(config?.priority).toBe('high');
  });
});
