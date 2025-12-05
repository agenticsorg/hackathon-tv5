/**
 * AST Parser Module
 * Extracts arwConfig exports from TypeScript/JavaScript files
 */

import { parse } from '@babel/parser';
import traverse from '@babel/traverse';
import * as t from '@babel/types';
import { promises as fs } from 'fs';

export interface ARWConfigChunk {
  id: string;
  heading?: string;
  description?: string;
  url_fragment?: string;
}

export interface ARWConfig {
  priority?: 'low' | 'normal' | 'high';
  purpose?: string;
  chunks?: ARWConfigChunk[];
  title?: string;
  description?: string;
}

export class ASTParser {
  /**
   * Extract arwConfig export from a source file
   */
  async extractARWConfig(filePath: string): Promise<ARWConfig | null> {
    try {
      const code = await fs.readFile(filePath, 'utf-8');

      // Parse the file as TypeScript/JSX
      const ast = parse(code, {
        sourceType: 'module',
        plugins: ['typescript', 'jsx']
      });

      let arwConfig: ARWConfig | null = null;

      // Traverse the AST to find arwConfig export
      traverse(ast, {
        ExportNamedDeclaration: (path) => {
          const declaration = path.node.declaration;

          // Handle: export const arwConfig = {...}
          if (t.isVariableDeclaration(declaration)) {
            for (const decl of declaration.declarations) {
              if (
                t.isIdentifier(decl.id) &&
                decl.id.name === 'arwConfig' &&
                decl.init &&
                t.isObjectExpression(decl.init)
              ) {
                arwConfig = this.parseConfigObject(decl.init);
              }
            }
          }
        }
      });

      return arwConfig;
    } catch (error) {
      console.error(`[ARW] Error parsing ${filePath}:`, error);
      return null;
    }
  }

  /**
   * Parse an AST object expression into ARWConfig
   */
  private parseConfigObject(node: t.ObjectExpression): ARWConfig {
    const config: ARWConfig = {};

    for (const prop of node.properties) {
      if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
        const key = prop.key.name as keyof ARWConfig;
        const value = this.evaluateNode(prop.value);

        if (value !== null && value !== undefined) {
          (config as any)[key] = value;
        }
      }
    }

    return config;
  }

  /**
   * Safely evaluate AST nodes to extract literal values
   */
  private evaluateNode(node: t.Node): any {
    if (t.isStringLiteral(node)) {
      return node.value;
    }

    if (t.isNumericLiteral(node)) {
      return node.value;
    }

    if (t.isBooleanLiteral(node)) {
      return node.value;
    }

    if (t.isNullLiteral(node)) {
      return null;
    }

    if (t.isArrayExpression(node)) {
      return node.elements
        .map(el => el ? this.evaluateNode(el) : null)
        .filter(v => v !== null);
    }

    if (t.isObjectExpression(node)) {
      return this.parseConfigObject(node);
    }

    // For complex expressions, return null
    return null;
  }
}
