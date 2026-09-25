export interface Recommendation {
  constraint_id: string;
  severity: string;
  explanation: string;
  recommended_action: string;
  confidence?: number;
  status: 'OPEN' | 'ACCEPTED' | 'REJECTED' | 'MODIFIED' | 'COMMITTED';
}

export interface Decision {
  caseId: string;
  recommendationId: string;
  selectedAction: string;
  decisionReason?: string;
  decisionMaker: string;
  timestamp: string;
  confidence?: number;
  expectedImpact?: string;
  scheduleVersion?: string;
}
