/**
 * Doctor Command - Comprehensive ARW health check
 */

import chalk from 'chalk';
import { runDoctorCheck } from '../validate.js';

export async function doctorCommand(url, options = {}) {
  const { platform, timeout = 8000, json = false } = options;

  console.log(chalk.bold.blue('\n🔍 ARW Doctor - Comprehensive Health Check\n'));
  console.log(chalk.gray(`URL: ${url}`));
  console.log(chalk.gray(`Timeout: ${timeout}ms`));
  if (platform) {
    console.log(chalk.gray(`Platform: ${platform}`));
  }
  console.log('');

  try {
    const report = await runDoctorCheck(url, { timeout, platform });

    if (json) {
      // JSON output for CI/CD
      console.log(JSON.stringify(report, null, 2));
      return report;
    }

    // Human-readable output
    printDoctorReport(report);

    // Exit code based on errors
    const hasErrors = report.summary.errors > 0;
    process.exitCode = hasErrors ? 1 : 0;

    return report;
  } catch (error) {
    console.error(chalk.red(`\n✗ Doctor check failed: ${error.message}`));
    process.exitCode = 1;
    throw error;
  }
}

function printDoctorReport(report) {
  const { sections, summary } = report;

  // Summary
  console.log(chalk.bold('\n📊 Summary'));
  console.log(chalk.gray('─'.repeat(50)));
  console.log(`Total Checks: ${summary.totalChecks}`);
  console.log(chalk.green(`Passed: ${summary.passed}`));
  console.log(chalk.yellow(`Warnings: ${summary.warnings}`));
  console.log(chalk.red(`Errors: ${summary.errors}`));

  // Calculate health score
  const healthScore = summary.totalChecks > 0
    ? Math.round((summary.passed / summary.totalChecks) * 100)
    : 0;

  const scoreColor = healthScore >= 90 ? chalk.green :
                     healthScore >= 70 ? chalk.yellow : chalk.red;
  console.log(`\nHealth Score: ${scoreColor(healthScore + '%')}`);

  // Core validation
  if (sections.core) {
    printSection('Core Validation', sections.core);
  }

  // Format validation
  if (sections.formats) {
    printSection('Format Validation (YAML + JSON)', sections.formats);
  }

  // Well-known
  if (sections.wellKnown) {
    printSection('Well-Known Manifest', sections.wellKnown);
  }

  // Robots.txt
  if (sections.robots) {
    printSection('Robots.txt Hints', sections.robots);
  }

  // Compatibility
  if (sections.compatibility) {
    printCompatibilitySection(sections.compatibility);
  }

  // Platform-specific
  if (sections.platform) {
    printSection(`Platform Checks (${sections.platform.platform})`, sections.platform);
  }

  // Recommendations
  printRecommendations(sections);
}

function printSection(title, section) {
  console.log(chalk.bold(`\n${title}`));
  console.log(chalk.gray('─'.repeat(50)));

  if (section.passed && section.passed.length > 0) {
    console.log(chalk.green('\n✓ Passed:'));
    section.passed.forEach(p => console.log(chalk.green(`  • ${p}`)));
  }

  if (section.warnings && section.warnings.length > 0) {
    console.log(chalk.yellow('\n⚠ Warnings:'));
    section.warnings.forEach(w => console.log(chalk.yellow(`  • ${w}`)));
  }

  if (section.errors && section.errors.length > 0) {
    console.log(chalk.red('\n✗ Errors:'));
    section.errors.forEach(e => console.log(chalk.red(`  • ${e}`)));
  }
}

function printCompatibilitySection(section) {
  console.log(chalk.bold('\nAgent Compatibility Tests'));
  console.log(chalk.gray('─'.repeat(50)));

  if (section.agents) {
    for (const [agentType, result] of Object.entries(section.agents)) {
      const icon = result.accessible ? chalk.green('✓') : chalk.red('✗');
      const status = result.accessible ? chalk.green('Accessible') : chalk.red('Failed');
      console.log(`\n${icon} ${formatAgentName(agentType)}: ${status}`);
      console.log(chalk.gray(`  Method: ${result.method}`));
      console.log(chalk.gray(`  ${result.message}`));
    }
  }
}

function formatAgentName(name) {
  return name
    .split(/(?=[A-Z])/)
    .join(' ')
    .replace(/^\w/, c => c.toUpperCase());
}

function printRecommendations(sections) {
  console.log(chalk.bold('\n💡 Recommendations'));
  console.log(chalk.gray('─'.repeat(50)));

  const recommendations = [];

  // Check if JSON is missing
  if (sections.formats?.warnings?.some(w => w.includes('Missing llms.json'))) {
    recommendations.push('Create llms.json for better agent compatibility');
    recommendations.push('  → Run: arw convert llms.txt llms.json');
  }

  // Check if well-known is missing
  if (sections.wellKnown?.warnings?.some(w => w.includes('Missing'))) {
    recommendations.push('Add /.well-known/arw-manifest.json for improved discovery');
  }

  // Check if robots.txt needs updating
  if (sections.robots?.warnings?.length > 0) {
    recommendations.push('Add ARW hints to robots.txt');
    recommendations.push('  → Add comment: # Agent-Ready Web: /llms.txt');
  }

  // Check compatibility issues
  if (sections.compatibility?.agents) {
    const failedAgents = Object.values(sections.compatibility.agents)
      .filter(a => !a.accessible);
    if (failedAgents.length > 0) {
      recommendations.push('Some agents cannot access your ARW implementation');
      recommendations.push('  → Review format and header configuration');
    }
  }

  if (recommendations.length === 0) {
    console.log(chalk.green('  All looks good! No recommendations at this time.'));
  } else {
    recommendations.forEach(r => console.log(chalk.cyan(`  • ${r}`)));
  }

  console.log('');
}
