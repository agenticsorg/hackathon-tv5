/**
 * Test Compatibility Command - Tests actual agent accessibility
 */

import chalk from 'chalk';
import { testCompatibility } from '../validators/compatibility-tester.js';
import { fetchText } from '../validate.js';

export async function testCompatibilityCommand(url, options = {}) {
  const { timeout = 8000, json = false } = options;

  if (!json) {
    console.log(chalk.bold.blue('\n🧪 ARW Compatibility Testing\n'));
    console.log(chalk.gray(`URL: ${url}`));
    console.log(chalk.gray(`Testing agent accessibility...\n`));
  }

  try {
    const results = await testCompatibility(url, fetchText, { timeout });

    if (json) {
      console.log(JSON.stringify(results, null, 2));
      return results;
    }

    // Print results
    console.log(chalk.bold('Agent Compatibility Results'));
    console.log(chalk.gray('─'.repeat(60)));

    let totalTests = 0;
    let passedTests = 0;

    for (const [agentType, result] of Object.entries(results.agents || {})) {
      totalTests++;
      const icon = result.accessible ? chalk.green('✓') : chalk.red('✗');
      const status = result.accessible ? chalk.green('PASS') : chalk.red('FAIL');

      console.log(`\n${icon} ${formatAgentName(agentType)}`);
      console.log(chalk.gray(`  Method: ${result.method}`));
      console.log(`  Status: ${status}`);
      console.log(chalk.gray(`  ${result.message}`));

      if (result.accessible) passedTests++;
    }

    // Summary
    console.log(chalk.bold('\n📊 Summary'));
    console.log(chalk.gray('─'.repeat(60)));
    console.log(`Tests Run: ${totalTests}`);
    console.log(chalk.green(`Passed: ${passedTests}`));
    console.log(chalk.red(`Failed: ${totalTests - passedTests}`));

    const percentage = totalTests > 0 ? Math.round((passedTests / totalTests) * 100) : 0;
    const percentColor = percentage >= 90 ? chalk.green :
                        percentage >= 70 ? chalk.yellow : chalk.red;
    console.log(`Compatibility: ${percentColor(percentage + '%')}`);

    if (results.warnings && results.warnings.length > 0) {
      console.log(chalk.yellow('\n⚠ Warnings:'));
      results.warnings.forEach(w => console.log(chalk.yellow(`  • ${w}`)));
    }

    if (results.errors && results.errors.length > 0) {
      console.log(chalk.red('\n✗ Errors:'));
      results.errors.forEach(e => console.log(chalk.red(`  • ${e}`)));
    }

    console.log('');

    // Exit code
    process.exitCode = passedTests === totalTests ? 0 : 1;

    return results;
  } catch (error) {
    console.error(chalk.red(`\n✗ Compatibility test failed: ${error.message}`));
    process.exitCode = 1;
    throw error;
  }
}

function formatAgentName(name) {
  return name
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, str => str.toUpperCase())
    .trim();
}
