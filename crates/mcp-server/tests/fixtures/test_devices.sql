-- Test fixture: Sample devices for testing list_devices tool
-- Test user ID: 123e4567-e89b-12d3-a456-426614174000

-- Insert test devices
INSERT INTO user_devices (user_id, device_id, device_type, platform, capabilities, app_version, last_seen, is_online, device_name)
VALUES
    (
        '123e4567-e89b-12d3-a456-426614174000',
        'test-device-tv-001',
        'TV',
        'Tizen',
        '{"max_resolution": "UHD_4K", "hdr_support": ["HDR10", "DolbyVision"], "audio_codecs": ["AAC", "DolbyAtmos"], "remote_controllable": true, "can_cast": false, "screen_size": 65.0}'::jsonb,
        '1.0.0',
        NOW() - INTERVAL '1 minute',
        true,
        'Living Room TV'
    ),
    (
        '123e4567-e89b-12d3-a456-426614174000',
        'test-device-phone-001',
        'Phone',
        'iOS',
        '{"max_resolution": "FHD", "hdr_support": ["HDR10"], "audio_codecs": ["AAC"], "remote_controllable": false, "can_cast": true, "screen_size": 6.1}'::jsonb,
        '1.2.3',
        NOW() - INTERVAL '5 minutes',
        true,
        'iPhone 15'
    ),
    (
        '123e4567-e89b-12d3-a456-426614174000',
        'test-device-tablet-001',
        'Tablet',
        'Android',
        '{"max_resolution": "HD", "hdr_support": [], "audio_codecs": ["AAC"], "remote_controllable": false, "can_cast": true, "screen_size": 10.1}'::jsonb,
        '1.1.5',
        NOW() - INTERVAL '10 minutes',
        false,
        'Samsung Tablet'
    );
