-- Test content fixtures for integration tests

INSERT INTO content (id, title, description, content_type, metadata, quality_score, created_at, updated_at)
VALUES
    ('550e8400-e29b-41d4-a716-446655440001', 'The Matrix', 'A computer hacker learns about the true nature of reality', 'movie',
     '{"genre": ["action", "sci-fi"], "year": 1999, "platforms": ["netflix", "hbo"]}'::jsonb,
     0.95, NOW(), NOW()),

    ('550e8400-e29b-41d4-a716-446655440002', 'Inception', 'A thief who steals corporate secrets through dream-sharing', 'movie',
     '{"genre": ["action", "sci-fi", "thriller"], "year": 2010, "platforms": ["netflix"]}'::jsonb,
     0.92, NOW(), NOW()),

    ('550e8400-e29b-41d4-a716-446655440003', 'Breaking Bad', 'A high school chemistry teacher turned meth producer', 'series',
     '{"genre": ["drama", "crime"], "year": 2008, "platforms": ["netflix"]}'::jsonb,
     0.98, NOW(), NOW()),

    ('550e8400-e29b-41d4-a716-446655440004', 'The Dark Knight', 'Batman faces the Joker in Gotham City', 'movie',
     '{"genre": ["action", "crime", "drama"], "year": 2008, "platforms": ["hbo", "amazon"]}'::jsonb,
     0.96, NOW(), NOW()),

    ('550e8400-e29b-41d4-a716-446655440005', 'Interstellar', 'Explorers travel through a wormhole in space', 'movie',
     '{"genre": ["sci-fi", "drama", "adventure"], "year": 2014, "platforms": ["netflix", "paramount"]}'::jsonb,
     0.90, NOW(), NOW());
