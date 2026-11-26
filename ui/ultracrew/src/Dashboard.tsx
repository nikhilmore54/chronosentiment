import { useState } from 'react';

import type { SimulationState } from './App';
import type { Recommendation } from './types';
import { RecommendationCard } from './components/RecommendationCard';


export interface Coverage {
  covered: number;
  understaffed: number;
  critical: number;
}

export interface Alert {
  employee: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
}

interface DashboardProps {
  data: any;
  baselineData: any;
  simulationState?: SimulationState | null;
  demoStep?: 'baseline' | 'optimization' | 'recovery';
  onAlertClick?: (employeeId: string, dayIndex: number) => void;
}

export const Dashboard = ({
  data,
  baselineData,
  simulationState,
  demoStep = 'baseline',
  onAlertClick,
}: DashboardProps) => {
  // Local UI state
  const [showModal, setShowModal] = useState<string | null>(null);
  const [modifyRec, setModifyRec] = useState<any | null>(null);
  const [detailsRec, setDetailsRec] = useState<any | null>(null);
  const [actionMessage, setActionMessage] = useState<string>('');
  const [recommendations, setRecommendations] = useState<any[]>([]);

  const [decisions, setDecisions] = useState<any[]>([]);
  const [saveError, setSaveError] = useState<string>('');
  const [retryDecision, setRetryDecision] = useState<{action:string, rec:Recommendation}|null>(null);





  interface Decision {
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

  // Helper to create a decision record with mock persistence
  const createDecision = (action: string, rec: Recommendation) => {
    const newDecision: Decision = {
      caseId: `CASE-${rec.constraint_id}`,
      recommendationId: rec.constraint_id,
      selectedAction: action,
      decisionMaker: 'User',
      timestamp: new Date().toISOString(),
      confidence: rec.confidence,
      expectedImpact: 'N/A',
      scheduleVersion: data?.schedule_version || 'v1',
    };
    // Simulate occasional save failure
    const failSave = Math.random() < 0.1;
    if (failSave) {
      setSaveError('Failed to save decision.');
      setRetryDecision({ action, rec });
    } else {
      setDecisions(prev => [newDecision, ...prev]);
      setSaveError('');
      setRetryDecision(null);
    }
  };


  // Handlers for recommendation actions (now include decision creation)
  const handleAccept = (rec: Recommendation) => {
    // Mark as accepted and update status
    setRecommendations(prev => prev.map(r => r.constraint_id === rec.constraint_id ? { ...r, status: 'ACCEPTED' } : r));
    createDecision('Accept', rec);
    setActionMessage(`Accepted recommendation ${rec.constraint_id}`);
    setTimeout(() => setActionMessage(''), 3000);
    // No refresh to preserve state
  };
  const handleReject = (rec: Recommendation) => {
    // Remove the recommendation from the open list
    setRecommendations(prev => prev.filter(r => r.constraint_id !== rec.constraint_id));
    // Record the reject decision
    createDecision('Reject', rec);
    setActionMessage(`Rejected recommendation ${rec.constraint_id}`);
    setTimeout(() => setActionMessage(''), 3000);
    // No refresh to preserve state
  };
  const handleModify = (rec: Recommendation) => {
    setModifyRec(rec);
    setShowModal('modify');
  };
  const handleDetails = (rec: Recommendation) => {
    setDetailsRec(rec);
    setShowModal('details');
  };

  // Retry saving a failed decision
  const retrySave = () => {
    if (retryDecision) {
      setSaveError('');
      createDecision(retryDecision.action, retryDecision.rec);
    }
  };






  if (!data) {
    return <p>Loading Dashboard...</p>;
  }

  const renderStepper = () => {
    const steps = [
      { id: 'baseline', label: '1. Roster Baseline', desc: 'Optimized baseline schedule' },
      { id: 'optimization', label: '2. MOGA Scheduler', desc: 'Solving recovery swaps' },
      { id: 'recovery', label: '3. Recovery Complete', desc: 'Workload balanced, coverage restored' }
    ];

    let activeIdx = 0;
    if (demoStep === 'optimization') activeIdx = 1;
    else if (demoStep === 'recovery') activeIdx = 2;

    return (
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        backgroundColor: 'var(--bg-color)',
        borderRadius: '12px',
        padding: '1.25rem 2rem',
        marginBottom: '1.5rem',
        border: '1px solid var(--border-color)'
      }}>
        {steps.map((step, idx) => {
          const isActive = idx === activeIdx;
          const isCompleted = idx < activeIdx;
          return (
            <div key={step.id} style={{ display: 'flex', alignItems: 'center', gap: '1rem', flex: 1 }}>
              <div style={{
                width: '32px',
                height: '32px',
                borderRadius: '50%',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontWeight: 700,
                backgroundColor: isCompleted ? 'var(--success-color)' : (isActive ? 'var(--primary-color)' : 'var(--panel-bg)'),
                color: isCompleted || isActive ? 'white' : 'var(--text-muted)',
                border: isActive ? '2px solid var(--accent-color)' : '1px solid var(--border-color)',
                transition: 'all 0.3s'
              }}>
                {isCompleted ? '✓' : idx + 1}
              </div>
              <div style={{ display: 'flex', flexDirection: 'column' }}>
                <span style={{ fontWeight: 600, fontSize: '0.9rem', color: isActive ? 'var(--accent-color)' : 'var(--text-main)' }}>{step.label}</span>
                <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>{step.desc}</span>
              </div>
              {idx < steps.length - 1 && (
                <div style={{
                  height: '2px',
                  backgroundColor: isCompleted ? 'var(--success-color)' : 'var(--border-color)',
                  flex: 1,
                  margin: '0 1.5rem',
                  minWidth: '30px'
                }} />
              )}
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div style={{ padding: '2rem' }}>
      {renderStepper()}

        {/* Primary Action Buttons */}
        <div style={{ display: 'flex', gap: '1rem', marginTop: '1rem' }}>
        </div>

        {/* Summary Cards */}
        <div style={{ display: 'flex', gap: '1rem', marginTop: '1.5rem', flexWrap: 'wrap' }}>
          <div style={{ flex: '1 1 200px', backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
            <h4 style={{ margin: 0, color: 'var(--text-muted)' }}>Decision Cases</h4>
            <p style={{ fontSize: '1.5rem', margin: '0.5rem 0', color: 'var(--text-main)' }}>{decisions.length}</p>
          </div>
          <div style={{ flex: '1 1 200px', backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
            <h4 style={{ margin: 0, color: 'var(--text-muted)' }}>Ready for Review</h4>
            <p style={{ fontSize: '1.5rem', margin: '0.5rem 0', color: 'var(--text-main)' }}>0</p>
          </div>
          <div style={{ flex: '1 1 200px', backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
            <h4 style={{ margin: 0, color: 'var(--text-muted)' }}>Ready to Commit</h4>
            <p style={{ fontSize: '1.5rem', margin: '0.5rem 0', color: 'var(--text-main)' }}>0</p>
          </div>
          <div style={{ flex: '1 1 200px', backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
            <h4 style={{ margin: 0, color: 'var(--text-muted)' }}>Schedule Health</h4>
            <p style={{ fontSize: '1.5rem', margin: '0.5rem 0', color: 'var(--success-color)' }}>Good</p>
          </div>
        </div>


          {recommendations.filter(r => r.status === 'OPEN').length > 0 ? (
            <div style={{ marginTop: '1rem' }}>
              <h3 style={{ marginBottom: '0.5rem' }}>Recommendations</h3>
              {recommendations.filter(r => r.status === 'OPEN').map((rec, idx) => (
                <RecommendationCard
                  key={idx}
                  recommendation={rec}
                  onAccept={handleAccept}
                  onReject={handleReject}
                  onModify={handleModify}
                  onDetails={handleDetails}
                />
              ))}
            </div>
          ) : (
            <div style={{ marginTop: '1rem', color: 'var(--text-muted)' }}>No recommendations available.</div>
          )}
          {actionMessage && (
      <div style={{ marginTop: '1rem', color: 'var(--accent-color)', fontWeight: 600 }}>{actionMessage}</div>
    )}
    {saveError && (
      <div style={{ marginTop: '1rem', color: 'var(--danger-color)' }}>{saveError}</div>
    )}
    {retryDecision && (
      <button onClick={retrySave} style={{ marginTop: '0.5rem' }}>Retry Save</button>
    )}
    
    
          {/* Decision Log Panel */}
          <div style={{ marginTop: '2rem' }}>
            <h3>Decision Log</h3>
            {decisions.length === 0 ? (
              <div style={{ color: 'var(--text-muted)' }}>No decisions recorded yet.</div>
            ) : (
              <ul style={{ listStyle: 'none', padding: 0 }}>
                {decisions.map((d, i) => (
                  <li key={i} style={{ borderBottom: '1px solid var(--border-color)', padding: '0.5rem 0' }}>
                    <strong>{d.selectedAction}</strong> on {d.recommendationId} at {new Date(d.timestamp).toLocaleString()}
                  </li>
                ))}
              </ul>
            )}
          </div>
      {showModal && showModal !== 'modify' && showModal !== 'details' && (
        <div style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, backgroundColor: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}>
          <div style={{ backgroundColor: 'white', padding: '2rem', borderRadius: '8px', maxWidth: '400px', width: '90%' }}>
            <h3>Confirm Action</h3>
            <p>Are you sure you want to {showModal}?</p>
            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
              <button onClick={() => setShowModal(null)}>Cancel</button>
              <button onClick={() => { onAlertClick?.(showModal, 0); setShowModal(null); }}>Confirm</button>
            </div>
          </div>
        </div>
      )}
      {/* Modify Modal */}
      {showModal === 'modify' && modifyRec && (
        <div style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, backgroundColor: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}>
          <div style={{ backgroundColor: 'white', padding: '2rem', borderRadius: '8px', maxWidth: '500px', width: '90%' }}>
            <h3>Modify Recommendation {modifyRec.constraint_id}</h3>
            <textarea id="modify-action" style={{ width: '100%', height: '100px', marginTop: '0.5rem' }} defaultValue={modifyRec.recommended_action} />
            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end', marginTop: '1rem' }}>
              <button onClick={() => { setShowModal(null); setModifyRec(null); }}>Cancel</button>
              <button onClick={() => {
                const newAction = (document.getElementById('modify-action') as HTMLTextAreaElement).value;
                setRecommendations(prev => prev.map(r => r.constraint_id === modifyRec.constraint_id ? { ...r, recommended_action: newAction, status: 'MODIFIED' } : r));
                // Record the modify action as a decision
                createDecision('Modify', { ...modifyRec, recommended_action: newAction });
                setActionMessage(`Modified recommendation ${modifyRec.constraint_id}`);
                setTimeout(() => setActionMessage(''), 3000);
                setShowModal(null);
                setModifyRec(null);
              }}>Save</button>
            </div>
          </div>
        </div>
      )}
      {/* Details Modal */}
      {showModal === 'details' && detailsRec && (
        <div style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, backgroundColor: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}>
          <div style={{ backgroundColor: 'white', padding: '2rem', borderRadius: '8px', maxWidth: '500px', width: '90%' }}>
            <h3>Recommendation Details {detailsRec.constraint_id}</h3>
            <p><strong>Severity:</strong> {detailsRec.severity}</p>
            <p><strong>Explanation:</strong> {detailsRec.explanation}</p>
            <p><strong>Recommended Action:</strong> {detailsRec.recommended_action}</p>
            {detailsRec.confidence !== undefined && <p><strong>Confidence:</strong> {detailsRec.confidence}%</p>}
            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end', marginTop: '1rem' }}>
              <button onClick={() => { setShowModal(null); setDetailsRec(null); }}>Close</button>
            </div>
          </div>
        </div>
      )}
      <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: '2rem', marginBottom: 0 }}>
        <h2>Workforce Health</h2>
        
        {/* Row 1: Coverage Status */}
        <div>
          <h3 style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '1rem' }}>
            Am I Covered?
          </h3>
          <div style={{ display: 'flex', gap: '1rem' }}>
            <div style={{ padding: '1.25rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1, border: '1px solid var(--border-color)' }}>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Fully Covered Shifts</div>
              <div style={{ fontSize: '2rem', fontWeight: 600, color: 'var(--success-color)' }}>{data.coverage.covered}%</div>
            </div>
            <div style={{ padding: '1.25rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1, border: '1px solid var(--border-color)' }}>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Understaffed Shifts</div>
              <div style={{ fontSize: '2rem', fontWeight: 600, color: 'var(--accent-color)' }}>{data.coverage.understaffed}</div>
            </div>
            <div style={{ padding: '1.25rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1, border: '1px solid var(--border-color)' }}>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Critical Gaps</div>
              <div style={{ fontSize: '2rem', fontWeight: 600, color: data.coverage.critical > 0 ? 'var(--danger-color)' : 'var(--text-main)' }}>{data.coverage.critical}</div>
            </div>
          </div>
        </div>

        {/* Row 2: Accordions */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          <details open={false} style={{ border: '1px solid var(--border-color)', borderRadius: '8px', padding: '0.75rem' }}>
            <summary style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)', cursor: 'pointer' }}>Recent Changes</summary>
            <div>{Array.isArray(data.recent_changes) && data.recent_changes.length > 0 ? (
              <ul style={{ margin: '0.5rem 0', paddingLeft: '1.2rem' }}>
                {data.recent_changes.map((c: any, i: number) => (
                  <li key={i} style={{ fontSize: '0.8rem', color: 'var(--text-main)' }}>{c}</li>
                ))}
              </ul>
            ) : (
              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>No recent changes.</span>
            )}</div>
          </details>

          <details open={false} style={{ border: '1px solid var(--border-color)', borderRadius: '8px', padding: '0.75rem' }}>
            <summary style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)', cursor: 'pointer' }}>Pending Decisions</summary>
            <div>{Array.isArray(data.pending_decisions) && data.pending_decisions.length > 0 ? (
              <ul style={{ margin: '0.5rem 0', paddingLeft: '1.2rem' }}>
                {data.pending_decisions.map((d: any, i: number) => (
                  <li key={i} style={{ fontSize: '0.8rem', color: 'var(--text-main)' }}>{d}</li>
                ))}
              </ul>
            ) : (
              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>No pending decisions.</span>
            )}</div>
          </details>

          <details open={false} style={{ border: '1px solid var(--border-color)', borderRadius: '8px', padding: '0.75rem' }}>
            <summary style={{ fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)', cursor: 'pointer' }}>Activity Feed</summary>
            <div>{Array.isArray(data.activity_feed) && data.activity_feed.length > 0 ? (
              <ul style={{ margin: '0.5rem 0', paddingLeft: '1.2rem' }}>
                {data.activity_feed.map((a: any, i: number) => (
                  <li key={i} style={{ fontSize: '0.8rem', color: 'var(--text-main)' }}>{a}</li>
                ))}
              </ul>
            ) : (
              <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>No recent activity.</span>
            )}</div>
          </details>
        </div>


        {baselineData && data && simulationState && (
          <div style={{
            marginTop: '1rem',
            padding: '1.5rem',
            backgroundColor: 'rgba(56, 189, 248, 0.05)',
            border: '1px dashed var(--accent-color)',
            borderRadius: '12px'
          }}>
            <h3 style={{ margin: '0 0 1.5rem 0', color: 'var(--accent-color)', fontSize: '0.9rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              ⚖️ Before vs. After Recovery Comparison
            </h3>
            
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '1rem' }}>
              {/* Legality */}
              <div style={{ backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Roster Legality</div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '0.5rem' }}>
                  <div>
                    <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Before:</span><br/>
                    <span style={{ fontWeight: 600, color: baselineData.validation_report.is_legal ? 'var(--success-color)' : 'var(--danger-color)' }}>
                      {baselineData.validation_report.is_legal ? 'LEGAL' : 'INVALID'}
                    </span>
                  </div>
                  <div style={{ fontSize: '1.2rem', color: 'var(--text-muted)' }}>→</div>
                  <div>
                    <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>After:</span><br/>
                    <span style={{ fontWeight: 600, color: data.validation_report.is_legal ? 'var(--success-color)' : 'var(--danger-color)' }}>
                      {data.validation_report.is_legal ? 'LEGAL' : 'INVALID'}
                    </span>
                  </div>
                </div>
                <div style={{ marginTop: '0.5rem', fontSize: '0.85rem', fontWeight: 600, color: data.validation_report.is_legal && !baselineData.validation_report.is_legal ? 'var(--success-color)' : 'var(--text-muted)' }}>
                  {data.validation_report.is_legal && !baselineData.validation_report.is_legal ? '↑ Improved' : 'No Change'}
                </div>
              </div>

              {/* Coverage */}
              <div style={{ backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Coverage Score</div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '0.5rem' }}>
                  <div>
                    <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Before:</span><br/>
                    <span style={{ fontWeight: 600 }}>{baselineData.roster_health.coverage_score}%</span>
                  </div>
                  <div style={{ fontSize: '1.2rem', color: 'var(--text-muted)' }}>→</div>
                  <div>
                    <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>After:</span><br/>
                    <span style={{ fontWeight: 600 }}>{data.roster_health.coverage_score}%</span>
                  </div>
                </div>
                {(() => {
                  const delta = data.roster_health.coverage_score - baselineData.roster_health.coverage_score;
                  return (
                    <div style={{ marginTop: '0.5rem', fontSize: '0.85rem', fontWeight: 600, color: delta > 0 ? 'var(--success-color)' : (delta < 0 ? 'var(--danger-color)' : 'var(--text-muted)') }}>
                      {delta > 0 ? `↑ +${delta}% Improved` : delta < 0 ? `↓ ${delta}% Reduced` : 'No Change'}
                    </div>
                  );
                })()}
              </div>

              {/* Balance */}
              <div style={{ backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Workload Balance</div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '0.5rem' }}>
                  <div>
                    <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Before:</span><br/>
                    <span style={{ fontWeight: 600 }}>{baselineData.roster_health.balance_score}%</span>
                  </div>
                  <div style={{ fontSize: '1.2rem', color: 'var(--text-muted)' }}>→</div>
                  <div>
                    <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>After:</span><br/>
                    <span style={{ fontWeight: 600 }}>{data.roster_health.balance_score}%</span>
                  </div>
                </div>
                {(() => {
                  const delta = data.roster_health.balance_score - baselineData.roster_health.balance_score;
                  return (
                    <div style={{ marginTop: '0.5rem', fontSize: '0.85rem', fontWeight: 600, color: delta > 0 ? 'var(--success-color)' : (delta < 0 ? 'var(--danger-color)' : 'var(--text-muted)') }}>
                      {delta > 0 ? `↑ +${delta}% Improved` : delta < 0 ? `↓ ${delta}% Reduced` : 'No Change'}
                    </div>
                  );
                })()}
              </div>

              {/* Rest Gaps */}
              <div style={{ backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
                <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Rest Violations</div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '0.5rem' }}>
                  <div>
                    <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Before:</span><br/>
                    <span style={{ fontWeight: 600, color: baselineData.validation_report.forbidden_successions > 0 ? 'var(--danger-color)' : 'inherit' }}>
                      {baselineData.validation_report.forbidden_successions}
                    </span>
                  </div>
                  <div style={{ fontSize: '1.2rem', color: 'var(--text-muted)' }}>→</div>
                  <div>
                    <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>After:</span><br/>
                    <span style={{ fontWeight: 600 }}>{data.validation_report.forbidden_successions}</span>
                  </div>
                </div>
                {(() => {
                  const delta = data.validation_report.forbidden_successions - baselineData.validation_report.forbidden_successions;
                  return (
                    <div style={{ marginTop: '0.5rem', fontSize: '0.85rem', fontWeight: 600, color: delta < 0 ? 'var(--success-color)' : (delta > 0 ? 'var(--danger-color)' : 'var(--text-muted)') }}>
                      {delta < 0 ? `↑ Improved (↓ ${delta})` : delta > 0 ? `↓ Worsened (↑ +${delta})` : 'No Change'}
                    </div>
                  );
                })()}
              </div>
            </div>
          </div>
        )}

        {simulationState && (
          <div style={{
            marginTop: '1rem',
            padding: '1.5rem',
            backgroundColor: 'rgba(34, 197, 94, 0.05)',
            border: '1px solid rgba(34, 197, 94, 0.15)',
            borderRadius: '12px'
          }}>
            <h3 style={{ margin: '0 0 1.25rem 0', color: 'var(--success-color)', fontSize: '0.9rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              🔍 Why UltraCrew Selected {Object.keys(simulationState.creditors)[0] || 'HN_3'}
            </h3>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '1.25rem', fontSize: '0.9rem' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-main)' }}>
                <span style={{ color: 'var(--success-color)', fontWeight: 'bold', fontSize: '1.1rem' }}>✓</span> 
                <span>Required skills available</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-main)' }}>
                <span style={{ color: 'var(--success-color)', fontWeight: 'bold', fontSize: '1.1rem' }}>✓</span> 
                <span>Rest guidelines satisfied</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-main)' }}>
                <span style={{ color: 'var(--success-color)', fontWeight: 'bold', fontSize: '1.1rem' }}>✓</span> 
                <span>Lowest workload increase</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-main)' }}>
                <span style={{ color: 'var(--success-color)', fontWeight: 'bold', fontSize: '1.1rem' }}>✓</span> 
                <span>Weekend balance preserved</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
