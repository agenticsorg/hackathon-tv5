import { Samsung, KEYS } from 'samsung-tv-control';
// @ts-ignore - no types available for wake_on_lan
import wol from 'wake_on_lan';
import {
  SamsungTVDevice,
  TVState,
  TVApp,
  TVCommand,
  RemoteKey,
  STREAMING_APPS,
  TVEvent,
  TVEventType,
} from './types.js';
import { checkTVOnline, getTVInfo } from './discovery.js';

// Re-export KEYS from samsung-tv-control for convenience
export { KEYS } from 'samsung-tv-control';

// Map our RemoteKey strings to KEYS enum
const KEY_MAP: Record<RemoteKey, KEYS> = {
  KEY_POWER: KEYS.KEY_POWER,
  KEY_POWEROFF: KEYS.KEY_POWEROFF,
  KEY_UP: KEYS.KEY_UP,
  KEY_DOWN: KEYS.KEY_DOWN,
  KEY_LEFT: KEYS.KEY_LEFT,
  KEY_RIGHT: KEYS.KEY_RIGHT,
  KEY_ENTER: KEYS.KEY_ENTER,
  KEY_RETURN: KEYS.KEY_RETURN,
  KEY_EXIT: KEYS.KEY_EXIT,
  KEY_HOME: KEYS.KEY_HOME,
  KEY_MENU: KEYS.KEY_MENU,
  KEY_SOURCE: KEYS.KEY_SOURCE,
  KEY_GUIDE: KEYS.KEY_GUIDE,
  KEY_INFO: KEYS.KEY_INFO,
  KEY_VOLUP: KEYS.KEY_VOLUP,
  KEY_VOLDOWN: KEYS.KEY_VOLDOWN,
  KEY_MUTE: KEYS.KEY_MUTE,
  KEY_CHUP: KEYS.KEY_CHUP,
  KEY_CHDOWN: KEYS.KEY_CHDOWN,
  KEY_PRECH: KEYS.KEY_PRECH,
  KEY_PLAY: KEYS.KEY_PLAY,
  KEY_PAUSE: KEYS.KEY_PAUSE,
  KEY_STOP: KEYS.KEY_STOP,
  KEY_REWIND: KEYS.KEY_REWIND,
  KEY_FF: KEYS.KEY_FF,
  KEY_REC: KEYS.KEY_REC,
  KEY_0: KEYS.KEY_0,
  KEY_1: KEYS.KEY_1,
  KEY_2: KEYS.KEY_2,
  KEY_3: KEYS.KEY_3,
  KEY_4: KEYS.KEY_4,
  KEY_5: KEYS.KEY_5,
  KEY_6: KEYS.KEY_6,
  KEY_7: KEYS.KEY_7,
  KEY_8: KEYS.KEY_8,
  KEY_9: KEYS.KEY_9,
  KEY_RED: KEYS.KEY_RED,
  KEY_GREEN: KEYS.KEY_GREEN,
  KEY_YELLOW: KEYS.KEY_YELLOW,
  KEY_BLUE: KEYS.KEY_CYAN, // Blue key maps to cyan in the library
  KEY_CONTENTS: KEYS.KEY_CONTENTS,
  KEY_SEARCH: KEYS.KEY_CONTENTS, // Search maps to contents
  KEY_AMBIENT: KEYS.KEY_AMBIENT,
};

type EventListener = (event: TVEvent) => void;

/**
 * Samsung TV Client - Provides high-level control over Samsung Smart TVs
 */
export class SamsungTVClient {
  private device: SamsungTVDevice;
  private control: Samsung | null = null;
  private eventListeners: EventListener[] = [];
  private connectionState: 'disconnected' | 'connecting' | 'connected' = 'disconnected';

  constructor(device: SamsungTVDevice) {
    this.device = device;
  }

  /**
   * Connect to the TV and get authentication token if needed
   */
  async connect(): Promise<{ success: boolean; token?: string; error?: string }> {
    if (this.connectionState === 'connected') {
      return { success: true, token: this.device.token };
    }

    this.connectionState = 'connecting';

    try {
      // Check if TV is online first
      const isOnline = await checkTVOnline(this.device.ip, this.device.port);
      if (!isOnline) {
        this.connectionState = 'disconnected';
        return { success: false, error: 'TV is not reachable. Make sure it is powered on.' };
      }

      // Samsung TV Control requires a MAC address
      const mac = this.device.mac || '00:00:00:00:00:00';

      this.control = new Samsung({
        ip: this.device.ip,
        mac,
        port: this.device.port,
        nameApp: 'HackathonTV5',
        debug: false,
        token: this.device.token,
      });

      // If we don't have a token, we need to get one (TV will show pairing dialog)
      if (!this.device.token) {
        return new Promise((resolve) => {
          this.control!.getToken((token: string | null) => {
            if (token) {
              this.device.token = token;
              this.connectionState = 'connected';
              this.emitEvent('device_connected', { token });
              resolve({ success: true, token });
            } else {
              this.connectionState = 'disconnected';
              resolve({ success: false, error: 'Failed to get token. User may have denied access.' });
            }
          });
        });
      }

      this.connectionState = 'connected';
      this.emitEvent('device_connected', {});
      return { success: true, token: this.device.token };
    } catch (error) {
      this.connectionState = 'disconnected';
      const message = error instanceof Error ? error.message : 'Unknown error';
      return { success: false, error: message };
    }
  }

  /**
   * Disconnect from the TV
   */
  disconnect() {
    if (this.control) {
      this.control.closeConnection();
    }
    this.control = null;
    this.connectionState = 'disconnected';
    this.emitEvent('device_disconnected', {});
  }

  /**
   * Get the current connection state
   */
  getConnectionState(): string {
    return this.connectionState;
  }

  /**
   * Get the device info
   */
  getDevice(): SamsungTVDevice {
    return { ...this.device };
  }

  /**
   * Update device token
   */
  setToken(token: string) {
    this.device.token = token;
  }

  /**
   * Power on the TV using Wake-on-LAN
   */
  async powerOn(): Promise<{ success: boolean; error?: string }> {
    if (!this.device.mac) {
      return { success: false, error: 'MAC address required for Wake-on-LAN' };
    }

    return new Promise((resolve) => {
      wol.wake(this.device.mac!, (error: Error | null) => {
        if (error) {
          resolve({ success: false, error: error.message });
        } else {
          this.emitEvent('state_changed', { power: 'on' });
          resolve({ success: true });
        }
      });
    });
  }

  /**
   * Power off the TV
   */
  async powerOff(): Promise<{ success: boolean; error?: string }> {
    return this.sendKey('KEY_POWER');
  }

  /**
   * Send a remote key press
   */
  async sendKey(key: RemoteKey): Promise<{ success: boolean; error?: string }> {
    if (!this.control || this.connectionState !== 'connected') {
      const connectResult = await this.connect();
      if (!connectResult.success) {
        return { success: false, error: connectResult.error };
      }
    }

    const mappedKey = KEY_MAP[key];
    if (!mappedKey) {
      return { success: false, error: `Unknown key: ${key}` };
    }

    try {
      await this.control!.sendKeyPromise(mappedKey);
      return { success: true };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to send key';
      return { success: false, error: message };
    }
  }

  /**
   * Send multiple keys with delay between them
   */
  async sendKeys(keys: RemoteKey[], delayMs: number = 300): Promise<{ success: boolean; error?: string }> {
    for (const key of keys) {
      const result = await this.sendKey(key);
      if (!result.success) {
        return result;
      }
      await this.delay(delayMs);
    }
    return { success: true };
  }

  /**
   * Adjust volume
   */
  async setVolume(action: 'up' | 'down' | 'mute' | 'unmute', steps: number = 1): Promise<{ success: boolean; error?: string }> {
    const keyMap: Record<string, RemoteKey> = {
      up: 'KEY_VOLUP',
      down: 'KEY_VOLDOWN',
      mute: 'KEY_MUTE',
      unmute: 'KEY_MUTE',
    };

    const key = keyMap[action];
    if (!key) {
      return { success: false, error: `Invalid volume action: ${action}` };
    }

    if (action === 'up' || action === 'down') {
      const keys = Array(steps).fill(key) as RemoteKey[];
      return this.sendKeys(keys, 100);
    }

    return this.sendKey(key);
  }

  /**
   * Get list of installed apps
   */
  async getApps(): Promise<{ success: boolean; apps?: TVApp[]; error?: string }> {
    if (!this.control || this.connectionState !== 'connected') {
      const connectResult = await this.connect();
      if (!connectResult.success) {
        return { success: false, error: connectResult.error };
      }
    }

    try {
      const apps = await this.control!.getAppsFromTVPromise();
      if (!apps || !apps.data?.data) {
        return { success: true, apps: [] };
      }
      const tvApps: TVApp[] = apps.data.data.map((app) => ({
        appId: app.appId,
        name: app.name,
        icon: app.icon,
        isRunning: false,
      }));
      return { success: true, apps: tvApps };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to get apps';
      return { success: false, error: message };
    }
  }

  /**
   * Launch an app by ID or name
   */
  async launchApp(appIdOrName: string): Promise<{ success: boolean; error?: string }> {
    if (!this.control || this.connectionState !== 'connected') {
      const connectResult = await this.connect();
      if (!connectResult.success) {
        return { success: false, error: connectResult.error };
      }
    }

    // Check if it's a known streaming app name
    const appId = this.resolveAppId(appIdOrName);

    try {
      await this.control!.openAppPromise(appId);
      this.emitEvent('app_launched', { appId });
      return { success: true };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to launch app';
      return { success: false, error: message };
    }
  }

  /**
   * Launch a streaming app by name (YouTube, Netflix, etc.)
   */
  async launchStreamingApp(appName: keyof typeof STREAMING_APPS): Promise<{ success: boolean; error?: string }> {
    const appId = STREAMING_APPS[appName];
    if (!appId) {
      return { success: false, error: `Unknown streaming app: ${appName}` };
    }
    return this.launchApp(appId);
  }

  /**
   * Navigate using arrow keys
   */
  async navigate(direction: 'up' | 'down' | 'left' | 'right' | 'enter' | 'back'): Promise<{ success: boolean; error?: string }> {
    const keyMap: Record<string, RemoteKey> = {
      up: 'KEY_UP',
      down: 'KEY_DOWN',
      left: 'KEY_LEFT',
      right: 'KEY_RIGHT',
      enter: 'KEY_ENTER',
      back: 'KEY_RETURN',
    };

    const key = keyMap[direction];
    if (!key) {
      return { success: false, error: `Invalid direction: ${direction}` };
    }

    return this.sendKey(key);
  }

  /**
   * Open the home screen
   */
  async goHome(): Promise<{ success: boolean; error?: string }> {
    return this.sendKey('KEY_HOME');
  }

  /**
   * Execute a TV command
   */
  async executeCommand(command: TVCommand): Promise<{ success: boolean; data?: unknown; error?: string }> {
    switch (command.type) {
      case 'power':
        if (command.action === 'on') {
          return this.powerOn();
        } else if (command.action === 'off') {
          return this.powerOff();
        } else {
          // Toggle - check current state and flip
          const isOnline = await checkTVOnline(this.device.ip, this.device.port);
          return isOnline ? this.powerOff() : this.powerOn();
        }

      case 'volume':
        if (command.action === 'set') {
          // Volume set requires multiple steps - we can't get current volume easily
          return { success: false, error: 'Direct volume set not supported. Use up/down.' };
        }
        return this.setVolume(command.action as 'up' | 'down' | 'mute' | 'unmute');

      case 'channel':
        if (command.action === 'up') {
          return this.sendKey('KEY_CHUP');
        } else if (command.action === 'down') {
          return this.sendKey('KEY_CHDOWN');
        } else if (command.action === 'set' && command.value) {
          // Enter channel number
          const keys = command.value.split('').map(digit => `KEY_${digit}` as RemoteKey);
          return this.sendKeys(keys, 200);
        }
        return { success: false, error: 'Invalid channel command' };

      case 'key':
        return this.sendKey(command.key);

      case 'app':
        if (command.action === 'launch' && command.appId) {
          return this.launchApp(command.appId);
        } else if (command.action === 'list') {
          const result = await this.getApps();
          return { success: result.success, data: result.apps, error: result.error };
        } else if (command.action === 'close') {
          return this.goHome();
        }
        return { success: false, error: 'Invalid app command' };

      case 'text':
        // Text input requires special handling - not all TVs support this
        return { success: false, error: 'Text input not yet implemented' };

      default:
        return { success: false, error: 'Unknown command type' };
    }
  }

  /**
   * Get current TV state (limited - TV doesn't expose much state)
   */
  async getState(): Promise<{ success: boolean; state?: TVState; error?: string }> {
    const isOnline = await checkTVOnline(this.device.ip, this.device.port);

    const state: TVState = {
      power: isOnline ? 'on' : 'off',
    };

    // Try to get additional info
    if (isOnline) {
      const info = await getTVInfo(this.device.ip, this.device.port);
      if (info) {
        // Additional state info could be added here if available
      }
    }

    return { success: true, state };
  }

  /**
   * Register event listener
   */
  onEvent(listener: EventListener) {
    this.eventListeners.push(listener);
  }

  /**
   * Remove event listener
   */
  offEvent(listener: EventListener) {
    this.eventListeners = this.eventListeners.filter(l => l !== listener);
  }

  // Private helpers
  private emitEvent(type: TVEventType, data: unknown) {
    const event: TVEvent = {
      type,
      deviceId: this.device.id,
      timestamp: new Date().toISOString(),
      data,
    };
    this.eventListeners.forEach(listener => listener(event));
  }

  private resolveAppId(appIdOrName: string): string {
    // Check if it's already an app ID (numeric string)
    if (/^\d+$/.test(appIdOrName)) {
      return appIdOrName;
    }

    // Try to match against known streaming apps
    const normalizedName = appIdOrName.toUpperCase().replace(/[^A-Z]/g, '_');
    const knownApp = STREAMING_APPS[normalizedName as keyof typeof STREAMING_APPS];
    if (knownApp) {
      return knownApp;
    }

    // Return as-is, might be a valid app ID
    return appIdOrName;
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Create a TV client from device info
 */
export function createTVClient(device: SamsungTVDevice): SamsungTVClient {
  return new SamsungTVClient(device);
}

/**
 * Create a TV client from IP address
 */
export function createTVClientFromIP(ip: string, options?: { port?: number; mac?: string; token?: string }): SamsungTVClient {
  const device: SamsungTVDevice = {
    id: `samsung-tv-${ip.replace(/\./g, '-')}`,
    name: `Samsung TV (${ip})`,
    ip,
    port: options?.port || 8002,
    mac: options?.mac,
    token: options?.token,
    isOnline: false,
  };
  return new SamsungTVClient(device);
}
