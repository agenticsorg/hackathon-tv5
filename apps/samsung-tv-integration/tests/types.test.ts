import { describe, it, expect } from 'vitest';
import {
  SamsungTVDeviceSchema,
  TVStateSchema,
  TVCommandSchema,
  RemoteKeySchema,
  STREAMING_APPS,
} from '../src/lib/types.js';

describe('SamsungTVDeviceSchema', () => {
  it('should validate a valid device', () => {
    const device = {
      id: 'samsung-tv-abc123',
      name: 'Living Room TV',
      ip: '192.168.1.100',
      port: 8002,
    };

    const result = SamsungTVDeviceSchema.safeParse(device);
    expect(result.success).toBe(true);
  });

  it('should validate device with optional fields', () => {
    const device = {
      id: 'samsung-tv-abc123',
      name: 'Living Room TV',
      ip: '192.168.1.100',
      mac: '00:11:22:33:44:55',
      model: 'Samsung Q80T',
      port: 8002,
      token: 'auth-token-123',
      isOnline: true,
      lastSeen: '2024-01-01T00:00:00.000Z',
    };

    const result = SamsungTVDeviceSchema.safeParse(device);
    expect(result.success).toBe(true);
  });

  it('should reject invalid device (missing required fields)', () => {
    const device = {
      id: 'samsung-tv-abc123',
      // missing name and ip
    };

    const result = SamsungTVDeviceSchema.safeParse(device);
    expect(result.success).toBe(false);
  });
});

describe('TVStateSchema', () => {
  it('should validate power states', () => {
    expect(TVStateSchema.safeParse({ power: 'on' }).success).toBe(true);
    expect(TVStateSchema.safeParse({ power: 'off' }).success).toBe(true);
    expect(TVStateSchema.safeParse({ power: 'unknown' }).success).toBe(true);
  });

  it('should reject invalid power state', () => {
    const result = TVStateSchema.safeParse({ power: 'maybe' });
    expect(result.success).toBe(false);
  });

  it('should validate volume range', () => {
    expect(TVStateSchema.safeParse({ power: 'on', volume: 50 }).success).toBe(true);
    expect(TVStateSchema.safeParse({ power: 'on', volume: 0 }).success).toBe(true);
    expect(TVStateSchema.safeParse({ power: 'on', volume: 100 }).success).toBe(true);
  });

  it('should reject volume out of range', () => {
    expect(TVStateSchema.safeParse({ power: 'on', volume: -1 }).success).toBe(false);
    expect(TVStateSchema.safeParse({ power: 'on', volume: 101 }).success).toBe(false);
  });
});

describe('TVCommandSchema', () => {
  it('should validate power commands', () => {
    expect(TVCommandSchema.safeParse({ type: 'power', action: 'on' }).success).toBe(true);
    expect(TVCommandSchema.safeParse({ type: 'power', action: 'off' }).success).toBe(true);
    expect(TVCommandSchema.safeParse({ type: 'power', action: 'toggle' }).success).toBe(true);
  });

  it('should validate volume commands', () => {
    expect(TVCommandSchema.safeParse({ type: 'volume', action: 'up' }).success).toBe(true);
    expect(TVCommandSchema.safeParse({ type: 'volume', action: 'down' }).success).toBe(true);
    expect(TVCommandSchema.safeParse({ type: 'volume', action: 'mute' }).success).toBe(true);
    expect(TVCommandSchema.safeParse({ type: 'volume', action: 'set', value: 50 }).success).toBe(true);
  });

  it('should validate key commands', () => {
    expect(TVCommandSchema.safeParse({ type: 'key', key: 'KEY_HOME' }).success).toBe(true);
    expect(TVCommandSchema.safeParse({ type: 'key', key: 'KEY_VOLUP' }).success).toBe(true);
  });

  it('should validate app commands', () => {
    expect(TVCommandSchema.safeParse({ type: 'app', action: 'list' }).success).toBe(true);
    expect(TVCommandSchema.safeParse({ type: 'app', action: 'launch', appId: '111299001912' }).success).toBe(true);
  });
});

describe('RemoteKeySchema', () => {
  it('should validate common keys', () => {
    const commonKeys = [
      'KEY_POWER', 'KEY_UP', 'KEY_DOWN', 'KEY_LEFT', 'KEY_RIGHT',
      'KEY_ENTER', 'KEY_HOME', 'KEY_MENU', 'KEY_VOLUP', 'KEY_VOLDOWN',
      'KEY_MUTE', 'KEY_PLAY', 'KEY_PAUSE', 'KEY_STOP',
    ];

    commonKeys.forEach(key => {
      expect(RemoteKeySchema.safeParse(key).success).toBe(true);
    });
  });

  it('should reject invalid keys', () => {
    expect(RemoteKeySchema.safeParse('INVALID_KEY').success).toBe(false);
    expect(RemoteKeySchema.safeParse('key_home').success).toBe(false);
  });
});

describe('STREAMING_APPS', () => {
  it('should contain common streaming apps', () => {
    expect(STREAMING_APPS.YOUTUBE).toBe('111299001912');
    expect(STREAMING_APPS.NETFLIX).toBe('11101200001');
    expect(STREAMING_APPS.PRIME_VIDEO).toBe('3201512006785');
    expect(STREAMING_APPS.DISNEY_PLUS).toBe('3201601007250');
    expect(STREAMING_APPS.SPOTIFY).toBe('3201606009684');
  });
});
