import { SwarmConfig } from 'agentic-flow';

export const productionSwarmConfig: SwarmConfig = {
    topology: 'adaptive',

    coordinator: {
        agent: 'adaptive-coordinator',
        maxConcurrentTasks: 13, // Updated for 13 agents
        conflictResolution: 'odd-prime-consensus', // Leverage odd-prime advantage
        autoCommit: {
            enabled: true,
            interval: 1800000, // 30 minutes
            requireTestsPass: true,
            generateCommitMessage: true
        }
    },

    teams: [
        {
            name: 'backend',
            agents: [
                'backend-dev',
                'database-architect',
                'api-docs',
                'platform-integrator' // NEW: 4th backend agent
            ],
            tasks: [
                'Build Metadata API with Express.js',
                'Design Firestore hypergraph schema for 400M users',
                'Generate OpenAPI specs and ARW manifest',
                'Implement Netflix, Amazon, FAST platform connectors' // NEW
            ],
            autoCommit: true
        },
        {
            name: 'qa-testing',
            agents: [
                'tdd-london-swarm',
                'sparc-agent',
                'tester',
                'production-validator'
            ],
            tasks: [
                'Write TDD tests (London School) - test FIRST',
                'SPARC validation and architecture review',
                'E2E and load testing for 400M users',
                'Production readiness validation'
            ],
            requireTestsPass: true,
            blockCommitsOnFailure: true
        },
        {
            name: 'devops',
            agents: [
                'cicd-engineer',
                'release-manager',
                'system-architect'
            ],
            tasks: [
                'Setup CI/CD pipeline with GitHub Actions',
                'Configure Cloud Run auto-scaling (100-10000 instances)',
                'Design multi-region architecture for 400M users'
            ],
            autoCommit: true
        },
        {
            name: 'data-ml',
            agents: [
                'data-scientist'
            ],
            tasks: [
                'Generate 1M+ test records with Agentic-Synth',
                'Integrate Vertex AI Matching Engine'
            ],
            useAgenticSynth: true
        }
    ],

    deployment: {
        platform: 'gcp',
        service: 'cloud-run',
        region: 'us-central1',
        multiRegion: true,
        useNativeUrls: true, // NEW: Use Cloud Run auto-generated URLs
        autoScaling: {
            minInstances: 100,
            maxInstances: 10000,
            targetConcurrency: 1000
        },
        resources: {
            cpu: 4,
            memory: '8Gi'
        }
    },

    swarmOptimization: {
        agentCount: 13, // Odd prime for optimal consensus
        consensusAlgorithm: 'odd-prime-voting',
        deadlockPrevention: true
    }
};
