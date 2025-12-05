#!/usr/bin/env node
// Validate an ARW manifest (YAML) against a JSON Schema using Ajv
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import Ajv from 'ajv';
import addFormats from 'ajv-formats';
import yaml from 'yaml';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const [, , manifestPath, schemaPath] = process.argv;
if (!manifestPath || !schemaPath) {
  console.error('usage: node tools/validators/validate-arw.mjs <manifest.yaml> <schema.json>');
  process.exit(2);
}

const manifestStr = fs.readFileSync(manifestPath, 'utf8');
const data = yaml.parse(manifestStr);

const schemaStr = fs.readFileSync(schemaPath, 'utf8');
const schema = JSON.parse(schemaStr);

const ajv = new Ajv({ allErrors: true, strict: false });
addFormats(ajv);
const validate = ajv.compile(schema);
const valid = validate(data);

if (!valid) {
  console.error('❌ Manifest failed validation:');
  for (const err of validate.errors) {
    const loc = err.instancePath || '(root)';
    console.error(`  - ${loc} ${err.message}`);
  }
  process.exit(1);
} else {
  console.log('✅ Manifest is valid.');
}
