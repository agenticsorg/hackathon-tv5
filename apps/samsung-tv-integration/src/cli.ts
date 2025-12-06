#!/usr/bin/env node
import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { prompt } from 'enquirer';
import { discoverTVs } from './lib/discovery.js';
import { createTVClient, createTVClientFromIP } from './lib/tv-client.js';
import { STREAMING_APPS, RemoteKey } from './lib/types.js';
import {
  getDevices,
  saveDevice,
  removeDevice,
  getDefaultDevice,
  setDefaultDevice,
  getConfigPath,
} from './utils/config.js';
import { isValidIP, isValidMAC } from './utils/helpers.js';

const program = new Command();

program
  .name('samsung-tv')
  .description('Samsung Smart TV integration CLI for the Agentics TV5 Hackathon')
  .version('1.0.0');

// Discover command
program
  .command('discover')
  .description('Discover Samsung Smart TVs on the local network')
  .option('-t, --timeout <ms>', 'Discovery timeout in milliseconds', '5000')
  .action(async (options) => {
    const spinner = ora('Searching for Samsung TVs...').start();
    const timeout = parseInt(options.timeout, 10);

    try {
      const devices = await discoverTVs({ timeout });

      if (devices.length === 0) {
        spinner.warn('No Samsung TVs found on the network');
        console.log(chalk.gray('Make sure your TV is powered on and connected to the same network.'));
        return;
      }

      spinner.succeed(`Found ${devices.length} Samsung TV(s)`);

      devices.forEach((device, index) => {
        console.log(chalk.cyan(`\n${index + 1}. ${device.name}`));
        console.log(`   IP: ${device.ip}`);
        console.log(`   Model: ${device.model || 'Unknown'}`);
        console.log(`   ID: ${device.id}`);

        // Save to config
        saveDevice(device);
      });

      // Set first as default if no default exists
      if (!getDefaultDevice() && devices.length > 0) {
        setDefaultDevice(devices[0].id);
        console.log(chalk.green(`\n${devices[0].name} set as default TV`));
      }
    } catch (error) {
      spinner.fail('Discovery failed');
      console.error(chalk.red(error instanceof Error ? error.message : 'Unknown error'));
    }
  });

// List command
program
  .command('list')
  .description('List saved Samsung TVs')
  .action(() => {
    const devices = getDevices();
    const defaultDevice = getDefaultDevice();

    if (devices.length === 0) {
      console.log(chalk.yellow('No TVs saved. Run "samsung-tv discover" to find TVs.'));
      return;
    }

    console.log(chalk.bold('\nSaved Samsung TVs:'));
    devices.forEach((device, index) => {
      const isDefault = device.id === defaultDevice?.id;
      const status = isDefault ? chalk.green(' (default)') : '';
      const tokenStatus = device.token ? chalk.green(' [paired]') : chalk.gray(' [not paired]');

      console.log(`\n${index + 1}. ${chalk.cyan(device.name)}${status}${tokenStatus}`);
      console.log(`   IP: ${device.ip}`);
      console.log(`   Model: ${device.model || 'Unknown'}`);
      console.log(`   ID: ${device.id}`);
    });
  });

// Connect command
program
  .command('connect')
  .description('Connect to a Samsung TV and pair')
  .option('-i, --ip <ip>', 'TV IP address')
  .option('-m, --mac <mac>', 'TV MAC address (for Wake-on-LAN)')
  .option('-d, --device <id>', 'Device ID from saved devices')
  .action(async (options) => {
    let device = null;

    if (options.device) {
      device = getDevices().find(d => d.id === options.device);
      if (!device) {
        console.error(chalk.red(`Device not found: ${options.device}`));
        return;
      }
    } else if (options.ip) {
      if (!isValidIP(options.ip)) {
        console.error(chalk.red('Invalid IP address'));
        return;
      }
      device = {
        id: `samsung-tv-${options.ip.replace(/\./g, '-')}`,
        name: `Samsung TV (${options.ip})`,
        ip: options.ip,
        port: 8002,
        mac: options.mac,
        isOnline: false,
        token: undefined as string | undefined,
      };
    } else {
      device = getDefaultDevice();
      if (!device) {
        console.error(chalk.red('No device specified. Use --ip, --device, or run "samsung-tv discover" first.'));
        return;
      }
    }

    const spinner = ora(`Connecting to ${device.name}...`).start();
    const client = createTVClient(device);

    try {
      console.log(chalk.yellow('\nPlease check your TV for a pairing request...'));
      const result = await client.connect();

      if (result.success) {
        spinner.succeed(`Connected to ${device.name}`);
        if (result.token) {
          device.token = result.token;
          saveDevice(device);
          console.log(chalk.green('Token saved for future connections.'));
        }

        // Set as default if first device
        if (getDevices().length === 1) {
          setDefaultDevice(device.id);
        }
      } else {
        spinner.fail(`Failed to connect: ${result.error}`);
      }
    } catch (error) {
      spinner.fail('Connection failed');
      console.error(chalk.red(error instanceof Error ? error.message : 'Unknown error'));
    }
  });

// Power command
program
  .command('power <action>')
  .description('Control TV power (on/off/toggle)')
  .option('-d, --device <id>', 'Device ID')
  .action(async (action, options) => {
    if (!['on', 'off', 'toggle'].includes(action)) {
      console.error(chalk.red('Action must be: on, off, or toggle'));
      return;
    }

    const device = options.device
      ? getDevices().find(d => d.id === options.device)
      : getDefaultDevice();

    if (!device) {
      console.error(chalk.red('No TV configured. Run "samsung-tv discover" first.'));
      return;
    }

    const client = createTVClient(device);
    const spinner = ora(`Sending power ${action}...`).start();

    try {
      const result = await client.executeCommand({ type: 'power', action: action as 'on' | 'off' | 'toggle' });

      if (result.success) {
        spinner.succeed(`Power ${action} sent`);
      } else {
        spinner.fail(`Failed: ${result.error}`);
      }
    } catch (error) {
      spinner.fail('Command failed');
      console.error(chalk.red(error instanceof Error ? error.message : 'Unknown error'));
    }
  });

// Volume command
program
  .command('volume <action>')
  .description('Control TV volume (up/down/mute/unmute)')
  .option('-s, --steps <n>', 'Number of steps', '1')
  .option('-d, --device <id>', 'Device ID')
  .action(async (action, options) => {
    if (!['up', 'down', 'mute', 'unmute'].includes(action)) {
      console.error(chalk.red('Action must be: up, down, mute, or unmute'));
      return;
    }

    const device = options.device
      ? getDevices().find(d => d.id === options.device)
      : getDefaultDevice();

    if (!device) {
      console.error(chalk.red('No TV configured. Run "samsung-tv discover" first.'));
      return;
    }

    const client = createTVClient(device);
    const steps = parseInt(options.steps, 10);

    try {
      const result = await client.setVolume(action as 'up' | 'down' | 'mute' | 'unmute', steps);

      if (result.success) {
        console.log(chalk.green(`Volume ${action}${steps > 1 ? ` (${steps}x)` : ''}`));
      } else {
        console.error(chalk.red(`Failed: ${result.error}`));
      }
    } catch (error) {
      console.error(chalk.red(error instanceof Error ? error.message : 'Unknown error'));
    }
  });

// Apps command
program
  .command('apps')
  .description('List installed apps on the TV')
  .option('-d, --device <id>', 'Device ID')
  .action(async (options) => {
    const device = options.device
      ? getDevices().find(d => d.id === options.device)
      : getDefaultDevice();

    if (!device) {
      console.error(chalk.red('No TV configured. Run "samsung-tv discover" first.'));
      return;
    }

    const client = createTVClient(device);
    const spinner = ora('Fetching apps...').start();

    try {
      const result = await client.getApps();

      if (result.success && result.apps) {
        spinner.succeed(`Found ${result.apps.length} apps`);
        result.apps.forEach((app, index) => {
          const running = app.isRunning ? chalk.green(' [running]') : '';
          console.log(`  ${index + 1}. ${app.name}${running}`);
          console.log(chalk.gray(`     ID: ${app.appId}`));
        });
      } else {
        spinner.fail(`Failed: ${result.error}`);
      }
    } catch (error) {
      spinner.fail('Failed to get apps');
      console.error(chalk.red(error instanceof Error ? error.message : 'Unknown error'));
    }
  });

// Launch command
program
  .command('launch <app>')
  .description('Launch an app (by ID or name like YOUTUBE, NETFLIX, etc.)')
  .option('-d, --device <id>', 'Device ID')
  .action(async (app, options) => {
    const device = options.device
      ? getDevices().find(d => d.id === options.device)
      : getDefaultDevice();

    if (!device) {
      console.error(chalk.red('No TV configured. Run "samsung-tv discover" first.'));
      return;
    }

    const client = createTVClient(device);
    const spinner = ora(`Launching ${app}...`).start();

    try {
      const result = await client.launchApp(app);

      if (result.success) {
        spinner.succeed(`Launched ${app}`);
      } else {
        spinner.fail(`Failed: ${result.error}`);
      }
    } catch (error) {
      spinner.fail('Failed to launch app');
      console.error(chalk.red(error instanceof Error ? error.message : 'Unknown error'));
    }
  });

// Key command
program
  .command('key <key>')
  .description('Send a remote key (e.g., KEY_HOME, KEY_MENU, KEY_PLAY)')
  .option('-d, --device <id>', 'Device ID')
  .action(async (key, options) => {
    const device = options.device
      ? getDevices().find(d => d.id === options.device)
      : getDefaultDevice();

    if (!device) {
      console.error(chalk.red('No TV configured. Run "samsung-tv discover" first.'));
      return;
    }

    const client = createTVClient(device);

    try {
      const result = await client.sendKey(key.toUpperCase() as RemoteKey);

      if (result.success) {
        console.log(chalk.green(`Sent key: ${key}`));
      } else {
        console.error(chalk.red(`Failed: ${result.error}`));
      }
    } catch (error) {
      console.error(chalk.red(error instanceof Error ? error.message : 'Unknown error'));
    }
  });

// Status command
program
  .command('status')
  .description('Get TV status')
  .option('-d, --device <id>', 'Device ID')
  .action(async (options) => {
    const device = options.device
      ? getDevices().find(d => d.id === options.device)
      : getDefaultDevice();

    if (!device) {
      console.error(chalk.red('No TV configured. Run "samsung-tv discover" first.'));
      return;
    }

    const client = createTVClient(device);
    const spinner = ora('Checking status...').start();

    try {
      const result = await client.getState();

      if (result.success && result.state) {
        spinner.succeed(`TV Status: ${result.state.power}`);
        if (result.state.currentApp) {
          console.log(`  Current app: ${result.state.currentAppName || result.state.currentApp}`);
        }
      } else {
        spinner.fail(`Failed: ${result.error}`);
      }
    } catch (error) {
      spinner.fail('Failed to get status');
      console.error(chalk.red(error instanceof Error ? error.message : 'Unknown error'));
    }
  });

// Set default command
program
  .command('default <deviceId>')
  .description('Set a TV as the default')
  .action((deviceId) => {
    const device = getDevices().find(d => d.id === deviceId);

    if (!device) {
      console.error(chalk.red(`Device not found: ${deviceId}`));
      return;
    }

    setDefaultDevice(deviceId);
    console.log(chalk.green(`${device.name} set as default TV`));
  });

// Remove command
program
  .command('remove <deviceId>')
  .description('Remove a saved TV')
  .action((deviceId) => {
    const device = getDevices().find(d => d.id === deviceId);

    if (!device) {
      console.error(chalk.red(`Device not found: ${deviceId}`));
      return;
    }

    removeDevice(deviceId);
    console.log(chalk.green(`Removed ${device.name}`));
  });

// Config command
program
  .command('config')
  .description('Show configuration location')
  .action(() => {
    console.log(`Configuration file: ${getConfigPath()}`);
  });

// Streaming apps reference
program
  .command('streaming-apps')
  .description('List known streaming app IDs')
  .action(() => {
    console.log(chalk.bold('\nKnown Streaming App IDs:'));
    Object.entries(STREAMING_APPS).forEach(([name, id]) => {
      console.log(`  ${chalk.cyan(name)}: ${id}`);
    });
    console.log(chalk.gray('\nUse these names with "samsung-tv launch <name>"'));
  });

// MCP server command
program
  .command('mcp')
  .description('Start the MCP server')
  .option('-t, --transport <type>', 'Transport type (stdio or sse)', 'stdio')
  .option('-p, --port <port>', 'Port for SSE transport', '3456')
  .action(async (options) => {
    if (options.transport === 'sse') {
      process.env.MCP_PORT = options.port;
      await import('./mcp/sse.js');
    } else {
      await import('./mcp/stdio.js');
    }
  });

program.parse();
