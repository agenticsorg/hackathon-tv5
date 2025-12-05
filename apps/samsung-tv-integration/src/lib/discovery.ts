import { Client as SSDPClient } from 'node-ssdp';
import { SamsungTVDevice } from './types.js';
import { generateDeviceId } from '../utils/helpers.js';

const SAMSUNG_TV_SEARCH_TARGET = 'urn:dial-multiscreen-org:service:dial:1';
const SAMSUNG_SERVER_PATTERN = /Samsung/i;

interface DiscoveryOptions {
  timeout?: number;
  searchTarget?: string;
}

interface SSDPHeaders {
  LOCATION?: string;
  SERVER?: string;
  USN?: string;
  ST?: string;
}

interface RemoteInfo {
  address: string;
  port: number;
}

/**
 * Discover Samsung Smart TVs on the local network using SSDP
 */
export async function discoverTVs(options: DiscoveryOptions = {}): Promise<SamsungTVDevice[]> {
  const { timeout = 5000, searchTarget = SAMSUNG_TV_SEARCH_TARGET } = options;
  const devices = new Map<string, SamsungTVDevice>();

  return new Promise((resolve) => {
    const client = new SSDPClient();

    client.on('response', (headers: SSDPHeaders, _statusCode: number, rinfo: RemoteInfo) => {
      // Filter for Samsung TVs
      if (headers.SERVER && SAMSUNG_SERVER_PATTERN.test(headers.SERVER)) {
        const ip = rinfo.address;
        const existingDevice = Array.from(devices.values()).find(d => d.ip === ip);

        if (!existingDevice) {
          const device: SamsungTVDevice = {
            id: generateDeviceId(ip),
            name: extractDeviceName(headers) || `Samsung TV (${ip})`,
            ip,
            port: extractPort(headers.LOCATION) || 8002,
            model: extractModel(headers.SERVER),
            isOnline: true,
            lastSeen: new Date().toISOString(),
          };
          devices.set(device.id, device);
        }
      }
    });

    // Start discovery
    client.search(searchTarget);

    // Stop after timeout
    setTimeout(() => {
      client.stop();
      resolve(Array.from(devices.values()));
    }, timeout);
  });
}

/**
 * Check if a specific TV is online
 */
export async function checkTVOnline(ip: string, port: number = 8002): Promise<boolean> {
  return new Promise((resolve) => {
    const http = require('http');
    const req = http.get(`http://${ip}:${port}/api/v2/`, { timeout: 3000 }, (res: { statusCode: number }) => {
      resolve(res.statusCode === 200);
    });
    req.on('error', () => resolve(false));
    req.on('timeout', () => {
      req.destroy();
      resolve(false);
    });
  });
}

/**
 * Get detailed TV info from its REST API
 */
export async function getTVInfo(ip: string, port: number = 8002): Promise<Partial<SamsungTVDevice> | null> {
  return new Promise((resolve) => {
    const http = require('http');
    const req = http.get(`http://${ip}:${port}/api/v2/`, { timeout: 5000 }, (res: { statusCode: number; on: (event: string, cb: (data?: Buffer) => void) => void }) => {
      if (res.statusCode !== 200) {
        resolve(null);
        return;
      }

      let data = '';
      res.on('data', (chunk: Buffer | undefined) => { if (chunk) data += chunk; });
      res.on('end', () => {
        try {
          const info = JSON.parse(data);
          resolve({
            name: info.device?.name || info.name,
            model: info.device?.modelName || info.model,
            id: info.device?.id || info.id,
          });
        } catch {
          resolve(null);
        }
      });
    });
    req.on('error', () => resolve(null));
    req.on('timeout', () => {
      req.destroy();
      resolve(null);
    });
  });
}

// Helper functions
function extractDeviceName(headers: SSDPHeaders): string | null {
  // Try to extract from USN or other headers
  if (headers.USN) {
    const match = headers.USN.match(/uuid:([^:]+)/);
    if (match) return match[1];
  }
  return null;
}

function extractPort(location: string | undefined): number | null {
  if (!location) return null;
  try {
    const url = new URL(location);
    return parseInt(url.port) || 8001;
  } catch {
    return null;
  }
}

function extractModel(server: string | undefined): string | undefined {
  if (!server) return undefined;
  // Server format: "SHP, UPnP/1.0, Samsung UPnP SDK/1.0"
  const match = server.match(/Samsung[^,]*/i);
  return match ? match[0].trim() : 'Samsung Smart TV';
}

/**
 * Continuous discovery that emits events when TVs are found
 */
export class TVDiscoveryService {
  private client: SSDPClient;
  private devices: Map<string, SamsungTVDevice> = new Map();
  private isRunning = false;
  private intervalId?: ReturnType<typeof setInterval>;
  private listeners: ((device: SamsungTVDevice) => void)[] = [];

  constructor(private options: DiscoveryOptions = {}) {
    this.client = new SSDPClient();
    this.setupListeners();
  }

  private setupListeners() {
    this.client.on('response', (headers: SSDPHeaders, _statusCode: number, rinfo: RemoteInfo) => {
      if (headers.SERVER && SAMSUNG_SERVER_PATTERN.test(headers.SERVER)) {
        const ip = rinfo.address;
        const existingDevice = Array.from(this.devices.values()).find(d => d.ip === ip);

        if (!existingDevice) {
          const device: SamsungTVDevice = {
            id: generateDeviceId(ip),
            name: extractDeviceName(headers) || `Samsung TV (${ip})`,
            ip,
            port: extractPort(headers.LOCATION) || 8002,
            model: extractModel(headers.SERVER),
            isOnline: true,
            lastSeen: new Date().toISOString(),
          };
          this.devices.set(device.id, device);
          this.notifyListeners(device);
        } else {
          // Update last seen
          existingDevice.lastSeen = new Date().toISOString();
          existingDevice.isOnline = true;
        }
      }
    });
  }

  private notifyListeners(device: SamsungTVDevice) {
    this.listeners.forEach(listener => listener(device));
  }

  onDeviceFound(listener: (device: SamsungTVDevice) => void) {
    this.listeners.push(listener);
  }

  start(intervalMs: number = 30000) {
    if (this.isRunning) return;
    this.isRunning = true;

    // Initial search
    this.client.search(this.options.searchTarget || SAMSUNG_TV_SEARCH_TARGET);

    // Periodic search
    this.intervalId = setInterval(() => {
      this.client.search(this.options.searchTarget || SAMSUNG_TV_SEARCH_TARGET);
    }, intervalMs);
  }

  stop() {
    this.isRunning = false;
    if (this.intervalId) {
      clearInterval(this.intervalId);
    }
    this.client.stop();
  }

  getDevices(): SamsungTVDevice[] {
    return Array.from(this.devices.values());
  }
}
