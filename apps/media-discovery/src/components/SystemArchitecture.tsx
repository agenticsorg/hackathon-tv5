'use client';

import { motion } from 'framer-motion';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Database, Server, Tv, Zap, Brain, Network, Shield, TrendingUp } from 'lucide-react';

const architectureLayers = [
  {
    title: 'Smart TVs (Edge Layer)',
    icon: Tv,
    color: 'from-purple-500 to-pink-500',
    stats: [
      { label: 'Connected Devices', value: '40M+' },
      { label: 'Recommendation Latency', value: '<15ms' },
      { label: 'Memory Footprint', value: '~80MB' },
      { label: 'Power Consumption', value: '<0.5W' },
    ],
    features: [
      'Omega Brain (7 Rust Crates)',
      'SIMD Vector Search',
      '12-Tier Memory System',
      '7 Temporal Feedback Loops',
      'ONNX Inference Engine',
    ],
  },
  {
    title: 'Constellation Servers (Backend)',
    icon: Server,
    color: 'from-blue-500 to-cyan-500',
    stats: [
      { label: 'Active Servers', value: '100' },
      { label: 'Sync Requests/sec', value: '1.2M+' },
      { label: 'Uptime (SLA)', value: '99.99%' },
      { label: 'Data Transfer/day', value: '1.4TB' },
    ],
    features: [
      'gRPC-based Synchronization',
      'Federated Learning',
      'Differential Privacy (ε=0.1)',
      'Raft Consensus',
      'Pattern Aggregation',
    ],
  },
  {
    title: 'RuVector-Postgres (Data Layer)',
    icon: Database,
    color: 'from-green-500 to-emerald-500',
    stats: [
      { label: 'Total Vectors', value: '150M+' },
      { label: 'Query Latency (P99)', value: '<15ms' },
      { label: 'Storage Reduction', value: '75%' },
      { label: 'Speedup vs pgvector', value: '13-41x' },
    ],
    features: [
      'SIMD-Accelerated HNSW',
      'Graph Neural Networks',
      'Adaptive Compression',
      '384-dim Embeddings',
      'Raft HA Cluster',
    ],
  },
];

export function SystemArchitecture() {
  return (
    <div className="space-y-8">
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="text-center space-y-4"
      >
        <h2 className="text-4xl font-bold bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
          Exogenesis Omega Architecture
        </h2>
        <p className="text-gray-400 max-w-3xl mx-auto">
          A three-tier distributed AI system combining on-device intelligence with cloud-scale pattern aggregation
        </p>
      </motion.div>

      {/* Architecture Layers */}
      <div className="grid gap-6 md:grid-cols-3">
        {architectureLayers.map((layer, index) => {
          const Icon = layer.icon;
          return (
            <motion.div
              key={layer.title}
              initial={{ opacity: 0, y: 50 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: index * 0.2 }}
            >
              <Card className="h-full hover:shadow-2xl transition-shadow duration-300 border-gray-700">
                <CardHeader>
                  <div className="flex items-center gap-3">
                    <div className={`p-3 rounded-lg bg-gradient-to-br ${layer.color}`}>
                      <Icon className="w-6 h-6 text-white" />
                    </div>
                    <div>
                      <CardTitle className="text-lg">{layer.title}</CardTitle>
                      <CardDescription>Layer {index + 1}</CardDescription>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  {/* Stats */}
                  <div className="grid grid-cols-2 gap-3">
                    {layer.stats.map((stat) => (
                      <div key={stat.label} className="bg-gray-800/50 rounded-lg p-3">
                        <div className="text-2xl font-bold bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent">
                          {stat.value}
                        </div>
                        <div className="text-xs text-gray-400 mt-1">{stat.label}</div>
                      </div>
                    ))}
                  </div>

                  {/* Features */}
                  <div className="space-y-2">
                    <div className="text-sm font-semibold text-gray-300">Key Features:</div>
                    <div className="flex flex-wrap gap-2">
                      {layer.features.map((feature) => (
                        <Badge key={feature} variant="secondary" className="text-xs">
                          {feature}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </CardContent>
              </Card>
            </motion.div>
          );
        })}
      </div>

      {/* System Benefits */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.8 }}
      >
        <Card className="border-gray-700">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Brain className="w-5 h-5 text-purple-400" />
              Why This Architecture Matters
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-6 md:grid-cols-4">
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <Zap className="w-5 h-5 text-yellow-400" />
                  <h3 className="font-semibold text-white">Blazing Fast</h3>
                </div>
                <p className="text-sm text-gray-400">
                  Sub-15ms recommendations through on-device SIMD-accelerated inference
                </p>
              </div>

              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <Shield className="w-5 h-5 text-green-400" />
                  <h3 className="font-semibold text-white">Privacy First</h3>
                </div>
                <p className="text-sm text-gray-400">
                  Differential privacy (ε=0.1) ensures individual viewing habits remain private
                </p>
              </div>

              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <Network className="w-5 h-5 text-blue-400" />
                  <h3 className="font-semibold text-white">Massive Scale</h3>
                </div>
                <p className="text-sm text-gray-400">
                  Serving 40M+ devices with 1.2M+ sync requests/sec at $0.0006/device/month
                </p>
              </div>

              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <TrendingUp className="w-5 h-5 text-pink-400" />
                  <h3 className="font-semibold text-white">Self-Learning</h3>
                </div>
                <p className="text-sm text-gray-400">
                  Federated learning aggregates patterns across millions of devices
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </div>
  );
}
