/**
 * Project template for initializing new hackathon projects
 */

export const PACKAGE_JSON_TEMPLATE = (projectName: string, teamName?: string) => ({
  name: projectName,
  version: '0.1.0',
  description: `${teamName ? `${teamName}'s ` : ''}Agentics TV5 Hackathon Project`,
  type: 'module',
  main: 'dist/index.js',
  scripts: {
    build: 'tsc',
    dev: 'tsc -w',
    start: 'node dist/index.js',
    lint: 'eslint src --ext .ts',
    test: 'echo "No tests configured"'
  },
  keywords: [
    'hackathon',
    'agentics',
    'agentic-ai',
    'tv5'
  ],
  author: teamName || '',
  license: 'MIT',
  dependencies: {},
  devDependencies: {
    typescript: '^5.3.0',
    '@types/node': '^20.10.0'
  }
});

export const TSCONFIG_TEMPLATE = {
  compilerOptions: {
    target: 'ES2022',
    module: 'NodeNext',
    moduleResolution: 'NodeNext',
    lib: ['ES2022'],
    outDir: './dist',
    rootDir: './src',
    strict: true,
    esModuleInterop: true,
    skipLibCheck: true,
    forceConsistentCasingInFileNames: true,
    declaration: true,
    sourceMap: true
  },
  include: ['src/**/*'],
  exclude: ['node_modules', 'dist']
};

export const INDEX_TS_TEMPLATE = `/**
 * Agentics TV5 Hackathon Project
 *
 * This is your project's main entry point.
 * Start building your agentic AI solution here!
 */

async function main() {
  console.log('🚀 Welcome to the Agentics TV5 Hackathon!');
  console.log('📖 Documentation: https://agentics.org/hackathon');
  console.log('💬 Discord: https://discord.agentics.org');
  console.log('');
  console.log('Start building your agentic AI solution...');

  // Your code here!
}

main().catch(console.error);
`;

export const GITIGNORE_TEMPLATE = `# Dependencies
node_modules/

# Build output
dist/
build/
*.tsbuildinfo

# Environment
.env
.env.local
.env.*.local

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
npm-debug.log*

# Testing
coverage/

# Temporary
tmp/
temp/
`;

export const PROJECT_README_TEMPLATE = (projectName: string, teamName?: string, track?: string) => `# ${projectName}

${teamName ? `**Team:** ${teamName}` : ''}
${track ? `**Track:** ${track}` : ''}

## About

This project was created for the [Agentics Foundation TV5 Hackathon](https://agentics.org/hackathon).

## Getting Started

\`\`\`bash
# Install dependencies
npm install

# Build the project
npm run build

# Run the project
npm start
\`\`\`

## Development

\`\`\`bash
# Watch mode
npm run dev
\`\`\`

## Resources

- [Hackathon Website](https://agentics.org/hackathon)
- [Discord Community](https://discord.agentics.org)
- [Google ADK Docs](https://google.github.io/adk-docs/)
- [Claude Docs](https://docs.anthropic.com)

## License

MIT
`;
