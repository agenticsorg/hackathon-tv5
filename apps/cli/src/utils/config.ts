/**
 * Configuration management utilities
 */

import { readFileSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';
import type { HackathonConfig, ToolSelection } from '../types.js';
import { CONFIG_FILE } from '../constants.js';

const DEFAULT_TOOLS: ToolSelection = {
  // AI Assistants
  claudeCode: false,
  geminiCli: false,
  // Orchestration
  claudeFlow: false,
  agenticFlow: false,
  flowNexus: false,
  adk: false,
  // Cloud Platform
  googleCloudCli: false,
  vertexAi: false,
  // Databases
  ruvector: false,
  agentDb: false,
  // Synthesis
  agenticSynth: false,
  strangeLoops: false,
  sparc: false,
  // Python Frameworks
  lionpride: false,
  agenticFramework: false,
  openaiAgents: false
};

export function getConfigPath(dir: string = process.cwd()): string {
  return join(dir, CONFIG_FILE);
}

export function configExists(dir?: string): boolean {
  return existsSync(getConfigPath(dir));
}

export function loadConfig(dir?: string): HackathonConfig | null {
  const configPath = getConfigPath(dir);
  if (!existsSync(configPath)) {
    return null;
  }

  try {
    const content = readFileSync(configPath, 'utf-8');
    return JSON.parse(content) as HackathonConfig;
  } catch {
    return null;
  }
}

export function saveConfig(config: HackathonConfig, dir?: string): void {
  const configPath = getConfigPath(dir);
  writeFileSync(configPath, JSON.stringify(config, null, 2), 'utf-8');
}

export function createDefaultConfig(projectName: string): HackathonConfig {
  return {
    projectName,
    tools: { ...DEFAULT_TOOLS },
    mcpEnabled: false,
    discordLinked: false,
    initialized: false,
    createdAt: new Date().toISOString()
  };
}

export function updateConfig(
  updates: Partial<HackathonConfig>,
  dir?: string
): HackathonConfig {
  const existing = loadConfig(dir) || createDefaultConfig('hackathon-project');
  const updated = { ...existing, ...updates };
  saveConfig(updated, dir);
  return updated;
}
