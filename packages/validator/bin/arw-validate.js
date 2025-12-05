
#!/usr/bin/env node
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import chalk from 'chalk';
import { validateDomain } from '../src/validate.js';

const argv = yargs(hideBin(process.argv))
  .usage('$0 <url>', 'Validate an ARW implementation', (y) => {
    return y.positional('url', {
      type: 'string',
      describe: 'Base URL to validate, e.g. https://example.com'
    }).option('timeout', {
      type: 'number',
      default: 8000,
      describe: 'HTTP timeout in ms'
    }).demandCommand(1);
  })
  .help()
  .argv;

const url = argv._[0];
validateDomain(url, { timeout: argv.timeout })
  .then((report) => {
    const ok = report.errors.length === 0;
    console.log('\n' + chalk.bold('ARW Validation Report'));
    console.log('Domain:', chalk.cyan(url));
    console.log('Profile:', report.profile || 'unknown');
    console.log('Level:', report.level || 'undetected');
    console.log('Checks passed:', chalk.green(report.passed.length));
    console.log('Checks failed:', report.errors.length ? chalk.red(report.errors.length) : chalk.green('0'));
    if (report.passed.length) {
      console.log('\n' + chalk.green('✔ Passed'));
      report.passed.forEach(p => console.log('  •', p));
    }
    if (report.errors.length) {
      console.log('\n' + chalk.red('✖ Errors'));
      report.errors.forEach(e => console.log('  •', e));
      process.exitCode = 1;
    } else {
      console.log('\n' + chalk.green('All good!'));
    }
  })
  .catch((e) => {
    console.error(chalk.red('Validation failed:'), e.message || e);
    process.exitCode = 1;
  });
