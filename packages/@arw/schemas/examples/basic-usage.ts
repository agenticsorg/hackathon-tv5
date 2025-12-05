/**
 * Basic usage examples for @arw/schemas
 */

import type { ARWManifest, WellKnownManifest } from '../src/types.js';
import { validateManifest, validateWellKnownManifest, validateByType } from '../src/validation.js';

// ============================================================================
// Example 1: Creating a typed ARW manifest
// ============================================================================

const manifest: ARWManifest = {
  version: '1.0',
  profile: 'ARW-2',
  site: {
    name: 'CloudCart',
    homepage: 'https://cloudcart.example.com',
    contact: 'ai@cloudcart.example.com',
    description: 'Cloud-native e-commerce platform',
  },
  content: [
    {
      url: '/products/wireless-keyboard',
      machine_view: '/products/wireless-keyboard.llm.md',
      title: 'Wireless Keyboard - Premium Mechanical',
      description: 'High-quality wireless mechanical keyboard',
      priority: 'high',
      tags: ['products', 'electronics', 'keyboards'],
    },
    {
      url: '/docs/api/authentication',
      machine_view: '/docs/api/authentication.llm.md',
      title: 'API Authentication Guide',
      priority: 'medium',
      tags: ['docs', 'api'],
    },
  ],
  policies: {
    training: {
      allowed: 'conditional',
      description: 'Training allowed with attribution',
    },
    inference: {
      allowed: 'allow',
    },
    attribution: {
      required: true,
      template: 'Source: {site_name} ({url})',
    },
    rate_limits: {
      anonymous_rps: 2,
      oauth_rps: 10,
    },
  },
};

console.log('Created ARW manifest:', manifest);

// ============================================================================
// Example 2: Validating an ARW manifest
// ============================================================================

const manifestResult = validateManifest(manifest);

if (manifestResult.valid) {
  console.log('✓ Manifest is valid');
  console.log('  Site:', manifestResult.data?.site.name);
  console.log('  Content items:', manifestResult.data?.content?.length);
} else {
  console.error('✗ Manifest validation failed:');
  manifestResult.errors?.forEach((err) => {
    console.error(`  ${err.path}: ${err.message}`);
  });
}

// ============================================================================
// Example 3: Creating a .well-known/arw-manifest.json
// ============================================================================

const wellKnownManifest: WellKnownManifest = {
  arw_version: '1.0',
  site: {
    name: 'CloudCart',
    homepage: 'https://cloudcart.example.com',
    contact: 'ai@cloudcart.example.com',
  },
  links: {
    guide: '/llms.txt',
    policies: '/.well-known/arw-policies.json',
    content_index: '/.well-known/arw-content-index.json',
    sitemap: '/sitemap.xml',
  },
};

const wellKnownResult = validateWellKnownManifest(wellKnownManifest);

if (wellKnownResult.valid) {
  console.log('✓ .well-known manifest is valid');
} else {
  console.error('✗ .well-known manifest validation failed');
}

// ============================================================================
// Example 4: Validating by type
// ============================================================================

// Simulate fetching and validating a manifest
const fetchedData = {
  version: '1.0',
  profile: 'ARW-1',
  site: {
    name: 'Example Site',
    homepage: 'https://example.com',
  },
};

const typeValidationResult = validateByType('manifest', fetchedData);

if (typeValidationResult.valid) {
  console.log('✓ Fetched manifest is valid');
} else {
  console.error('✗ Validation failed:', typeValidationResult.errors);
}

// ============================================================================
// Example 5: Handling validation errors
// ============================================================================

const invalidManifest = {
  version: '1.0',
  // Missing required 'profile' field
  site: {
    name: 'Invalid Example',
    // Missing required 'homepage' field
  },
};

const errorResult = validateManifest(invalidManifest);

if (!errorResult.valid) {
  console.log('Expected validation errors:');
  errorResult.errors?.forEach((err) => {
    console.log(`  Field: ${err.path}`);
    console.log(`  Error: ${err.message}`);
    if (err.data) {
      console.log(`  Details:`, err.data);
    }
  });
}

// ============================================================================
// Example 6: ARW-3 manifest with actions
// ============================================================================

const manifestWithActions: ARWManifest = {
  version: '1.0',
  profile: 'ARW-3',
  site: {
    name: 'CloudCart',
    homepage: 'https://cloudcart.example.com',
  },
  actions: [
    {
      id: 'add-to-cart',
      name: 'Add to Cart',
      description: 'Add a product to the shopping cart',
      method: 'POST',
      endpoint: '/api/cart/add',
      requires_oauth: true,
      scopes: ['cart:write'],
      input_schema: {
        type: 'object',
        properties: {
          product_id: { type: 'string' },
          quantity: { type: 'integer', minimum: 1 },
        },
        required: ['product_id', 'quantity'],
      },
      output_schema: {
        type: 'object',
        properties: {
          cart_id: { type: 'string' },
          total_items: { type: 'integer' },
        },
      },
    },
  ],
  oauth: {
    authorization_endpoint: 'https://cloudcart.example.com/oauth/authorize',
    token_endpoint: 'https://cloudcart.example.com/oauth/token',
    grant_types: ['authorization_code'],
    scopes: ['cart:read', 'cart:write', 'orders:read', 'orders:write'],
  },
};

const actionsResult = validateManifest(manifestWithActions);

if (actionsResult.valid) {
  console.log('✓ ARW-3 manifest with actions is valid');
  console.log('  Actions:', actionsResult.data?.actions?.length);
  console.log('  OAuth scopes:', actionsResult.data?.oauth?.scopes?.join(', '));
} else {
  console.error('✗ Actions manifest validation failed');
}
