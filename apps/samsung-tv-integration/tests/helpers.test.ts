import { describe, it, expect } from 'vitest';
import {
  generateDeviceId,
  isValidIP,
  isValidMAC,
  normalizeMAC,
  formatDuration,
  truncate,
  parseDeviceString,
} from '../src/utils/helpers.js';

describe('generateDeviceId', () => {
  it('should generate consistent IDs for same IP', () => {
    const id1 = generateDeviceId('192.168.1.100');
    const id2 = generateDeviceId('192.168.1.100');
    expect(id1).toBe(id2);
  });

  it('should generate different IDs for different IPs', () => {
    const id1 = generateDeviceId('192.168.1.100');
    const id2 = generateDeviceId('192.168.1.101');
    expect(id1).not.toBe(id2);
  });

  it('should start with samsung-tv-', () => {
    const id = generateDeviceId('192.168.1.100');
    expect(id.startsWith('samsung-tv-')).toBe(true);
  });
});

describe('isValidIP', () => {
  it('should validate correct IPv4 addresses', () => {
    expect(isValidIP('192.168.1.100')).toBe(true);
    expect(isValidIP('10.0.0.1')).toBe(true);
    expect(isValidIP('172.16.0.1')).toBe(true);
    expect(isValidIP('255.255.255.255')).toBe(true);
    expect(isValidIP('0.0.0.0')).toBe(true);
  });

  it('should reject invalid IP addresses', () => {
    expect(isValidIP('256.1.1.1')).toBe(false);
    expect(isValidIP('192.168.1')).toBe(false);
    expect(isValidIP('192.168.1.1.1')).toBe(false);
    expect(isValidIP('abc.def.ghi.jkl')).toBe(false);
    expect(isValidIP('')).toBe(false);
    expect(isValidIP('192.168.1.1:8080')).toBe(false);
  });
});

describe('isValidMAC', () => {
  it('should validate correct MAC addresses', () => {
    expect(isValidMAC('00:11:22:33:44:55')).toBe(true);
    expect(isValidMAC('00-11-22-33-44-55')).toBe(true);
    expect(isValidMAC('001122334455')).toBe(true);
    expect(isValidMAC('AA:BB:CC:DD:EE:FF')).toBe(true);
    expect(isValidMAC('aa:bb:cc:dd:ee:ff')).toBe(true);
  });

  it('should reject invalid MAC addresses', () => {
    expect(isValidMAC('00:11:22:33:44')).toBe(false);
    expect(isValidMAC('00:11:22:33:44:55:66')).toBe(false);
    expect(isValidMAC('GG:HH:II:JJ:KK:LL')).toBe(false);
    expect(isValidMAC('')).toBe(false);
  });
});

describe('normalizeMAC', () => {
  it('should normalize MAC addresses to colon format', () => {
    expect(normalizeMAC('001122334455')).toBe('00:11:22:33:44:55');
    expect(normalizeMAC('00-11-22-33-44-55')).toBe('00:11:22:33:44:55');
    expect(normalizeMAC('aa:bb:cc:dd:ee:ff')).toBe('AA:BB:CC:DD:EE:FF');
  });
});

describe('formatDuration', () => {
  it('should format milliseconds correctly', () => {
    expect(formatDuration(500)).toBe('500ms');
    expect(formatDuration(1500)).toBe('1.5s');
    expect(formatDuration(60000)).toBe('1m 0s');
    expect(formatDuration(90000)).toBe('1m 30s');
    expect(formatDuration(3600000)).toBe('1h 0m');
    expect(formatDuration(5400000)).toBe('1h 30m');
  });
});

describe('truncate', () => {
  it('should truncate long strings', () => {
    expect(truncate('Hello, World!', 8)).toBe('Hello...');
    expect(truncate('Short', 10)).toBe('Short');
    expect(truncate('Exactly10!', 10)).toBe('Exactly10!');
  });

  it('should use custom suffix', () => {
    expect(truncate('Hello, World!', 10, '…')).toBe('Hello, Wo…');
  });
});

describe('parseDeviceString', () => {
  it('should parse IP only', () => {
    const result = parseDeviceString('192.168.1.100');
    expect(result.ip).toBe('192.168.1.100');
    expect(result.port).toBe(8002);
  });

  it('should parse IP with port', () => {
    const result = parseDeviceString('192.168.1.100:8001');
    expect(result.ip).toBe('192.168.1.100');
    expect(result.port).toBe(8001);
  });
});
