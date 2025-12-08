import { parseSearchQuery } from './src/lib/natural-language-search.ts';

console.log('Testing query parsing...\n');

const testQuery = 'show me something cool';
console.log(`Query: "${testQuery}"\n`);

try {
  const result = await parseSearchQuery(testQuery);
  console.log('Parsed result:');
  console.log(JSON.stringify(result, null, 2));
} catch (error) {
  console.error('Error:', error.message);
}
