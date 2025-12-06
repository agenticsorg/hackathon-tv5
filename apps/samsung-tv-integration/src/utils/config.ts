import Conf from 'conf';
import { TVConfig, TVConfigSchema, SamsungTVDevice } from '../lib/types.js';

const CONFIG_NAME = 'samsung-tv-integration';

interface ConfigStore {
  devices: SamsungTVDevice[];
  defaultDeviceId?: string;
  discoveryTimeout: number;
  connectionTimeout: number;
}

const store = new Conf<ConfigStore>({
  projectName: CONFIG_NAME,
  defaults: {
    devices: [],
    discoveryTimeout: 5000,
    connectionTimeout: 10000,
  },
});

/**
 * Get the full configuration
 */
export function getConfig(): TVConfig {
  const raw = {
    devices: store.get('devices'),
    defaultDeviceId: store.get('defaultDeviceId'),
    discoveryTimeout: store.get('discoveryTimeout'),
    connectionTimeout: store.get('connectionTimeout'),
  };
  return TVConfigSchema.parse(raw);
}

/**
 * Save a device to configuration
 */
export function saveDevice(device: SamsungTVDevice): void {
  const devices = store.get('devices');
  const existingIndex = devices.findIndex(d => d.id === device.id);

  if (existingIndex >= 0) {
    devices[existingIndex] = device;
  } else {
    devices.push(device);
  }

  store.set('devices', devices);
}

/**
 * Remove a device from configuration
 */
export function removeDevice(deviceId: string): boolean {
  const devices = store.get('devices');
  const filtered = devices.filter(d => d.id !== deviceId);

  if (filtered.length === devices.length) {
    return false;
  }

  store.set('devices', filtered);

  // Clear default if it was this device
  if (store.get('defaultDeviceId') === deviceId) {
    store.delete('defaultDeviceId');
  }

  return true;
}

/**
 * Get all saved devices
 */
export function getDevices(): SamsungTVDevice[] {
  return store.get('devices');
}

/**
 * Get a specific device by ID
 */
export function getDevice(deviceId: string): SamsungTVDevice | undefined {
  const devices = store.get('devices');
  return devices.find(d => d.id === deviceId);
}

/**
 * Get device by IP address
 */
export function getDeviceByIP(ip: string): SamsungTVDevice | undefined {
  const devices = store.get('devices');
  return devices.find(d => d.ip === ip);
}

/**
 * Set the default device
 */
export function setDefaultDevice(deviceId: string): void {
  store.set('defaultDeviceId', deviceId);
}

/**
 * Get the default device
 */
export function getDefaultDevice(): SamsungTVDevice | undefined {
  const defaultId = store.get('defaultDeviceId');
  if (!defaultId) {
    const devices = store.get('devices');
    return devices[0];
  }
  return getDevice(defaultId);
}

/**
 * Update device token
 */
export function updateDeviceToken(deviceId: string, token: string): void {
  const devices = store.get('devices');
  const device = devices.find(d => d.id === deviceId);

  if (device) {
    device.token = token;
    store.set('devices', devices);
  }
}

/**
 * Clear all configuration
 */
export function clearConfig(): void {
  store.clear();
}

/**
 * Get configuration file path
 */
export function getConfigPath(): string {
  return store.path;
}
