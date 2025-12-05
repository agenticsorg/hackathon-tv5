/**
 * Convert Command - Convert YAML manifest to JSON
 */

import fs from 'node:fs';
import yaml from 'js-yaml';
import chalk from 'chalk';

export async function convertCommand(inputFile, outputFile, options = {}) {
  try {
    // Read YAML file
    console.log(chalk.blue(`Reading YAML from: ${inputFile}`));
    const yamlContent = fs.readFileSync(inputFile, 'utf8');

    // Parse YAML
    const data = yaml.load(yamlContent);
    console.log(chalk.green('✓ YAML parsed successfully'));

    // Convert to JSON
    const jsonContent = JSON.stringify(data, null, 2);

    // Write JSON file
    if (outputFile) {
      fs.writeFileSync(outputFile, jsonContent, 'utf8');
      console.log(chalk.green(`✓ JSON written to: ${outputFile}`));
    } else {
      // Output to stdout
      console.log(jsonContent);
    }

    // Validate structure
    if (data.version && data.site && data.policies) {
      console.log(chalk.green('✓ Valid ARW manifest structure'));
    } else {
      console.log(chalk.yellow('⚠ Warning: Manifest may be missing required fields'));
    }

    return { success: true, data };
  } catch (error) {
    console.error(chalk.red(`✗ Conversion failed: ${error.message}`));
    return { success: false, error };
  }
}
