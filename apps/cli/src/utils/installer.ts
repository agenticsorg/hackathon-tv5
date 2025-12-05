/**
 * Tool installation utilities
 */

import { execa } from 'execa';
import ora from 'ora';
import type { Tool, InstallProgress } from '../types.js';
import { logger } from './logger.js';

// Allowed command prefixes for security
const ALLOWED_COMMAND_PREFIXES = [
  'npx',
  'npm',
  'pip',
  'pip3',
  'python',
  'python3',
  'node',
  'git',
  'curl',
  'wget'
] as const;

// Active child processes for cleanup
const activeProcesses = new Set<ReturnType<typeof execa>>();

// Cleanup handler to kill child processes on exit
function setupCleanupHandlers(): void {
  const cleanup = async () => {
    for (const process of activeProcesses) {
      try {
        process.kill('SIGTERM');
        await new Promise(resolve => setTimeout(resolve, 1000));
        if (!process.killed) {
          process.kill('SIGKILL');
        }
      } catch {
        // Process may already be dead
      }
    }
    activeProcesses.clear();
  };

  process.on('SIGINT', async () => {
    await cleanup();
    process.exit(130);
  });

  process.on('SIGTERM', async () => {
    await cleanup();
    process.exit(143);
  });

  process.on('exit', () => {
    for (const proc of activeProcesses) {
      try {
        proc.kill('SIGKILL');
      } catch {
        // Ignore errors during exit
      }
    }
  });
}

// Setup cleanup handlers once
setupCleanupHandlers();

/**
 * Safely checks if a tool is installed by running its verify command
 * Uses execa with shell: false to prevent command injection
 * @param tool - The tool to check
 * @returns Promise resolving to true if installed, false otherwise
 */
export async function checkToolInstalled(tool: Tool): Promise<boolean> {
  try {
    const command = tool.verifyCommand.trim();

    if (!command) {
      return false;
    }

    // Parse command into executable and arguments
    const parts = command.split(/\s+/);
    const cmd = parts[0];
    const args = parts.slice(1);

    // Validate the base command is safe (no path traversal, etc.)
    if (cmd.includes('..') || cmd.includes('/') || cmd.includes('\\')) {
      // Only allow simple command names, not paths
      // Exception: allow absolute paths to known safe locations
      if (!cmd.startsWith('/usr/') && !cmd.startsWith('/bin/')) {
        return false;
      }
    }

    // Use execa with shell: false to prevent command injection
    const result = await execa(cmd, args, {
      shell: false,  // CRITICAL: Prevents shell injection
      reject: false, // Don't throw on non-zero exit
      timeout: 30000, // 30 second timeout
      stdio: 'pipe',
    });

    return result.exitCode === 0;
  } catch {
    return false;
  }
}

export async function installTool(tool: Tool): Promise<InstallProgress> {
  const spinner = ora(`Installing ${tool.displayName}...`).start();

  try {
    // Handle different install command types
    const command = tool.installCommand;

    if (command.startsWith('npx ')) {
      // For npx commands, just run them directly
      await runCommand(command);
    } else if (command.startsWith('npm install')) {
      await runCommand(command);
    } else if (command.startsWith('pip install')) {
      await runCommand(command);
    } else if (command.startsWith('curl')) {
      // Special handling for curl-based installs (like gcloud)
      spinner.info(`${tool.displayName} requires manual installation`);
      spinner.stop();
      logger.info(`Run: ${command}`);
      logger.link('Installation guide', tool.docUrl);
      return {
        tool: tool.name,
        status: 'skipped',
        message: 'Requires manual installation'
      };
    } else {
      await runCommand(command);
    }

    // Verify installation
    const installed = await checkToolInstalled(tool);

    if (installed) {
      spinner.succeed(`${tool.displayName} installed successfully`);
      return { tool: tool.name, status: 'success' };
    } else {
      spinner.warn(`${tool.displayName} may need additional setup`);
      return {
        tool: tool.name,
        status: 'success',
        message: 'Installed but verification pending'
      };
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    spinner.fail(`Failed to install ${tool.displayName}`);
    logger.error(message);
    return { tool: tool.name, status: 'failed', message };
  }
}

/**
 * Validates that a command is safe to execute
 * @param command - The command string to validate
 * @throws Error if command is potentially unsafe
 */
function validateCommand(command: string): void {
  const trimmedCommand = command.trim();

  if (!trimmedCommand) {
    throw new Error('Command cannot be empty');
  }

  // Extract the base command (first word)
  const baseCommand = trimmedCommand.split(/\s+/)[0];

  // Check if command starts with an allowed prefix
  const isAllowed = ALLOWED_COMMAND_PREFIXES.some(prefix =>
    baseCommand === prefix || baseCommand.startsWith(`${prefix}/`)
  );

  if (!isAllowed) {
    throw new Error(
      `Command "${baseCommand}" is not allowed. ` +
      `Allowed commands: ${ALLOWED_COMMAND_PREFIXES.join(', ')}`
    );
  }

  // Check for dangerous shell metacharacters in suspicious positions
  const dangerousPatterns = [
    /[;&|`$(){}[\]<>]/g, // Shell metacharacters
    /\$\{/g,              // Variable substitution
    /\$\(/g,              // Command substitution
  ];

  for (const pattern of dangerousPatterns) {
    if (pattern.test(command)) {
      throw new Error(
        `Command contains potentially unsafe characters: ${command}`
      );
    }
  }
}

/**
 * Safely executes a command using execa (no shell injection)
 * @param command - The command string to execute
 * @returns Promise resolving to stdout output
 */
export async function runCommand(command: string): Promise<string> {
  // Validate command before execution
  validateCommand(command);

  // Parse command into executable and arguments
  const parts = command.trim().split(/\s+/);
  const cmd = parts[0];
  const args = parts.slice(1);

  try {
    // Use execa for safe command execution (no shell injection)
    const childProcess = execa(cmd, args, {
      shell: false,  // CRITICAL: Disable shell to prevent injection
      stdio: 'pipe',
      reject: false, // Don't throw on non-zero exit
      timeout: 300000, // 5 minute timeout
      killSignal: 'SIGTERM',
    });

    // Track active process for cleanup
    activeProcesses.add(childProcess);

    const result = await childProcess;

    // Remove from active processes
    activeProcesses.delete(childProcess);

    if (result.exitCode === 0) {
      return result.stdout;
    } else {
      throw new Error(
        result.stderr ||
        result.stdout ||
        `Command exited with code ${result.exitCode}`
      );
    }
  } catch (error) {
    if (error instanceof Error) {
      throw error;
    }
    throw new Error(`Command execution failed: ${String(error)}`);
  }
}

/**
 * Helper to safely check if a command exists
 * Uses execa with shell: false for safety
 */
async function commandExists(cmd: string, args: string[]): Promise<boolean> {
  try {
    const result = await execa(cmd, args, {
      shell: false,
      reject: false,
      timeout: 10000,
      stdio: 'pipe',
    });
    return result.exitCode === 0;
  } catch {
    return false;
  }
}

export async function checkPrerequisites(): Promise<{
  node: boolean;
  npm: boolean;
  python: boolean;
  pip: boolean;
  git: boolean;
}> {
  // Run all checks in parallel for better performance
  const [node, npm, python3, python, pip3, pip, git] = await Promise.all([
    commandExists('node', ['--version']),
    commandExists('npm', ['--version']),
    commandExists('python3', ['--version']),
    commandExists('python', ['--version']),
    commandExists('pip3', ['--version']),
    commandExists('pip', ['--version']),
    commandExists('git', ['--version']),
  ]);

  return {
    node,
    npm,
    python: python3 || python,
    pip: pip3 || pip,
    git
  };
}
