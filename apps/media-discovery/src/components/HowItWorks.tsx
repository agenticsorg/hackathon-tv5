'use client';

import { motion } from 'framer-motion';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { ArrowRight, Lock, Zap, Brain, RefreshCw } from 'lucide-react';

const steps = [
  {
    number: '01',
    title: 'You Watch Content on Your TV',
    description: 'The Omega Brain (running on your smart TV) learns from your viewing patterns',
    icon: Brain,
    color: 'from-purple-500 to-pink-500',
    details: [
      'On-device AI inference (<15ms)',
      '12-tier memory system',
      '7 temporal feedback loops',
      'Only ~80MB memory footprint',
    ],
  },
  {
    number: '02',
    title: 'Privacy-Preserving Sync',
    description: 'Encrypted deltas (not raw data) are sent to Constellation Servers every 4 hours',
    icon: Lock,
    color: 'from-green-500 to-emerald-500',
    details: [
      '~1KB upload (compressed)',
      'Differential privacy (ε=0.1)',
      'AES-256-GCM encryption',
      'No viewing history transmitted',
    ],
  },
  {
    number: '03',
    title: 'Pattern Aggregation',
    description: 'Federated learning combines patterns from 40M+ TVs to discover trends',
    icon: RefreshCw,
    color: 'from-blue-500 to-cyan-500',
    details: [
      'Minimum 1,000 users per pattern',
      'Noise cancels out individual data',
      'Population trends emerge',
      'Graph Neural Network refinement',
    ],
  },
  {
    number: '04',
    title: 'Smarter Recommendations',
    description: 'Updated embeddings (~5KB) are pushed back to enhance your recommendations',
    icon: Zap,
    color: 'from-yellow-500 to-orange-500',
    details: [
      'Sub-15ms query latency',
      'SIMD-accelerated search',
      '150M+ vectors available',
      'Constantly improving',
    ],
  },
];

export function HowItWorks() {
  return (
    <div className="space-y-8">
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="text-center space-y-4"
      >
        <h2 className="text-4xl font-bold bg-gradient-to-r from-purple-400 via-pink-400 to-orange-400 bg-clip-text text-transparent">
          How It Works
        </h2>
        <p className="text-gray-400 max-w-3xl mx-auto">
          Your privacy-first journey from viewing to intelligent recommendations
        </p>
      </motion.div>

      {/* Steps */}
      <div className="relative">
        {/* Connection Line */}
        <div className="absolute left-8 top-0 bottom-0 w-0.5 bg-gradient-to-b from-purple-500 via-blue-500 to-orange-500 opacity-30 hidden lg:block" />

        <div className="space-y-6">
          {steps.map((step, index) => {
            const Icon = step.icon;
            return (
              <motion.div
                key={step.number}
                initial={{ opacity: 0, x: index % 2 === 0 ? -50 : 50 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: index * 0.1 }}
              >
                <Card className="border-gray-700 hover:border-gray-600 transition-all duration-300 hover:shadow-xl hover:shadow-purple-500/10">
                  <CardHeader>
                    <div className="flex items-start gap-4">
                      {/* Step Number */}
                      <div className={`flex-shrink-0 w-16 h-16 rounded-2xl bg-gradient-to-br ${step.color} flex items-center justify-center text-2xl font-bold text-white shadow-lg`}>
                        {step.number}
                      </div>

                      <div className="flex-1">
                        <div className="flex items-center gap-2">
                          <Icon className="w-5 h-5 text-gray-400" />
                          <CardTitle>{step.title}</CardTitle>
                        </div>
                        <CardDescription className="mt-2">
                          {step.description}
                        </CardDescription>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-2 gap-3 ml-20">
                      {step.details.map((detail) => (
                        <div
                          key={detail}
                          className="flex items-center gap-2 text-sm text-gray-400 bg-gray-800/30 rounded-lg px-3 py-2"
                        >
                          <ArrowRight className="w-3 h-3 text-purple-400 flex-shrink-0" />
                          <span>{detail}</span>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              </motion.div>
            );
          })}
        </div>
      </div>

      {/* Privacy Guarantee */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true }}
        transition={{ duration: 0.5 }}
      >
        <Card className="border-green-800/50 bg-green-900/10">
          <CardContent className="flex flex-col md:flex-row items-center justify-between gap-4 py-6">
            <div className="flex items-center gap-3">
              <Lock className="w-8 h-8 text-green-400" />
              <div>
                <h3 className="font-semibold text-white">Privacy Guaranteed</h3>
                <p className="text-sm text-gray-400">
                  Your individual viewing habits remain completely private
                </p>
              </div>
            </div>
            <div className="flex gap-2">
              <Badge variant="success">Differential Privacy</Badge>
              <Badge variant="success">AES-256-GCM</Badge>
              <Badge variant="success">k-Anonymity</Badge>
            </div>
          </CardContent>
        </Card>
      </motion.div>
    </div>
  );
}
