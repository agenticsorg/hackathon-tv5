export interface EmotionalState {
  valence: number;    // [-1.0, 1.0] negative to positive
  arousal: number;    // [-1.0, 1.0] calm to excited
  stress: number;     // [0.0, 1.0] relaxed to stressed
  confidence: number; // [0.0, 1.0] prediction confidence
}

export interface DesiredState {
  valence: number;    // Target valence [-1.0, 1.0]
  arousal: number;    // Target arousal [-1.0, 1.0]
  confidence: number; // Prediction confidence [0.0, 1.0]
}

export interface QTableEntry {
  stateHash: string;
  contentId: string;
  qValue: number;
  visitCount: number;
  lastUpdated: number;
}

export interface ActionSelection {
  contentId: string;
  qValue: number;
  isExploration: boolean;
  explorationBonus: number;
  confidence: number;
  stateHash: string;
}

export interface EmotionalExperience {
  stateBefore: EmotionalState;
  stateAfter: EmotionalState;
  contentId: string;
  desiredState: DesiredState;
  reward: number;
}

export interface PolicyUpdate {
  stateHash: string;
  contentId: string;
  oldQValue: number;
  newQValue: number;
  tdError: number;
  reward: number;
  visitCount: number;
}
