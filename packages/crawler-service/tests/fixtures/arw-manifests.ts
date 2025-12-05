/**
 * ARW manifest test fixtures
 * Sample llms.txt and arw-manifest.json content for testing
 */

export const VALID_ARW_MANIFEST = {
  version: '0.1',
  profile: 'ARW-1',
  site: {
    name: 'Test Site',
    description: 'A test website for ARW crawler',
    homepage: 'https://test.example.com',
    contact: 'ai@test.example.com'
  },
  content: [
    {
      url: '/docs/getting-started',
      machine_view: '/docs/getting-started.llm.md',
      purpose: 'documentation',
      priority: 'high'
    },
    {
      url: '/products/widget',
      machine_view: '/products/widget.llm.md',
      purpose: 'product',
      priority: 'medium'
    }
  ],
  policies: {
    training: {
      allowed: false,
      note: 'Content not licensed for model training'
    },
    inference: {
      allowed: true,
      restrictions: ['attribution_required']
    },
    attribution: {
      required: true,
      format: 'link'
    }
  }
};

export const MINIMAL_ARW_MANIFEST = {
  version: '0.1',
  site: {
    name: 'Minimal Site',
    homepage: 'https://minimal.example.com'
  }
};

export const ARW_MANIFEST_WITH_ACTIONS = {
  ...VALID_ARW_MANIFEST,
  actions: [
    {
      id: 'add_to_cart',
      name: 'Add to Cart',
      endpoint: '/api/actions/add-to-cart',
      method: 'POST',
      auth: 'oauth2',
      scopes: ['cart:write'],
      schema: {
        type: 'object',
        properties: {
          productId: { type: 'string' },
          quantity: { type: 'integer' }
        }
      }
    }
  ]
};

export const LLMS_TXT_YAML = `
version: 0.1
profile: ARW-1

site:
  name: 'Test Site'
  description: 'A test website for ARW crawler'
  homepage: 'https://test.example.com'
  contact: 'ai@test.example.com'

content:
  - url: /docs/getting-started
    machine_view: /docs/getting-started.llm.md
    purpose: documentation
    priority: high

policies:
  training:
    allowed: false
    note: 'Content not licensed for model training'
  inference:
    allowed: true
    restrictions: ['attribution_required']
`;

export const MACHINE_VIEW_MARKDOWN = `
# Getting Started

<!-- chunk: overview -->

## Overview

Clear, semantic Markdown optimized for LLM parsing.
No navigation, ads, or clutter—just pure content.

<!-- chunk: installation -->

## Installation

\`\`\`bash
npm install @agent-ready-web/cli
\`\`\`

<!-- chunk: usage -->

## Usage

Initialize ARW in your project:

\`\`\`bash
arw init
\`\`\`
`;
