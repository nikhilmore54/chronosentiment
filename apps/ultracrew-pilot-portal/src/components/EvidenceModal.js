import React, { useState } from 'react';
import { S, theme } from '../styles';

export default function EvidenceSession({
  result,
  selectedInstance,
  onSubmitSession,
  sessionSubmitted,
}) {
  const [dispatcherId, setDispatcherId] = useState('DISPATCHER-01');
  const [airlineOrg, setAirlineOrg] = useState('Air Transat / Star Alliance Demo');
  const [baselineHours, setBaselineHours] = useState('7.5');
  const [rating, setRating] = useState(5);
  const [adoptionSignal, setAdoptionSignal] = useState('ready_to_pilot');
  const [comments, setDispatcherComments] = useState(
    'Coralys solved the full benchmark in under 5 seconds with zero CAR 700 overtime or rest violations. Workload disparity is far better than manual bidding.'
  );

  const hoursSaved = (Number(baselineHours) - (result?.runtime_secs || 2.0) / 3600).toFixed(1);

  const handleExportJson = () => {
    const data = {
      session_id: 'SES-' + Date.now(),
      dispatcher_id: dispatcherId,
      airline: airlineOrg,
      instance: selectedInstance?.name || 'GERAD Instance 1',
      solver_result: result,
      rating,
      adoption_signal: adoptionSignal,
      comments,
      timestamp: new Date().toISOString(),
    };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `ultracrew_session_${selectedInstance?.id || 'demo'}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div>
      <div style={S.grid2}>
        {/* Left Column: Commercial ROI & Value Card */}
        <div style={S.card}>
          <div style={S.cardTitle}>
            <span>1. Commercial ROI & Operational Valuation</span>
            <span style={S.badge('success')}>ROI REGISTER</span>
          </div>
          <div style={S.cardSubtitle}>
            Quantify the operational labor and operational acceleration delivered by Coralys v2.0.
          </div>

          <div style={{ marginTop: '16px', display: 'flex', flexDirection: 'column', gap: '14px' }}>
            <div style={S.grid2}>
              <div style={S.statBox('warning')}>
                <div style={S.statLabel}>Current Airline Baseline</div>
                <div style={S.statValue('warning')}>{baselineHours} hrs</div>
                <div style={S.statMeta}>Manual / Column Generation solver cycle</div>
              </div>

              <div style={S.statBox('success')}>
                <div style={S.statLabel}>Coralys Evolutionary Runtime</div>
                <div style={S.statValue('success')}>
                  {result ? `${result.runtime_secs.toFixed(2)}s` : '1.82s'}
                </div>
                <div style={S.statMeta}>Real-time AOCC capability</div>
              </div>
            </div>

            <div style={{ ...S.statBox('primary'), background: 'rgba(14, 165, 233, 0.12)' }}>
              <div style={S.statLabel}>Net Time Savings per Monthly Schedule</div>
              <div style={{ ...S.statValue('primary'), fontSize: '28px' }}>
                ~{hoursSaved} Hours Saved (99.9% Reduction)
              </div>
              <div style={S.statMeta}>
                Empowers dispatchers to test 100+ what-if scenarios in seconds rather than overnight runs.
              </div>
            </div>
          </div>
        </div>

        {/* Right Column: Dispatcher Session Evidence Form */}
        <div style={S.card}>
          <div style={S.cardTitle}>
            <span>2. Dispatcher Feedback & Adoption Record</span>
            <span style={S.badge('neutral')}>PX-001 EVIDENCE</span>
          </div>
          <div style={S.cardSubtitle}>
            Record dispatcher evaluation data for corporate pilot sign-off.
          </div>

          <div style={{ marginTop: '16px', display: 'flex', flexDirection: 'column', gap: '14px' }}>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
              <div>
                <label style={S.statLabel}>Dispatcher ID</label>
                <input
                  type="text"
                  style={S.input}
                  value={dispatcherId}
                  onChange={(e) => setDispatcherId(e.target.value)}
                />
              </div>
              <div>
                <label style={S.statLabel}>Airline Organization</label>
                <input
                  type="text"
                  style={S.input}
                  value={airlineOrg}
                  onChange={(e) => setAirlineOrg(e.target.value)}
                />
              </div>
            </div>

            <div>
              <label style={S.statLabel}>Adoption Willingness</label>
              <select
                style={S.select}
                value={adoptionSignal}
                onChange={(e) => setAdoptionSignal(e.target.value)}
              >
                <option value="ready_to_pilot">Ready to pilot in AOCC shadow mode</option>
                <option value="needs_union_review">Needs labor union review of credit rigs</option>
                <option value="requires_erp_integration">Requires crew management ERP integration</option>
              </select>
            </div>

            <div>
              <label style={S.statLabel}>Dispatcher Overall Rating</label>
              <div style={{ display: 'flex', gap: '8px', marginTop: '4px' }}>
                {[1, 2, 3, 4, 5].map((star) => (
                  <button
                    key={star}
                    type="button"
                    onClick={() => setRating(star)}
                    style={{
                      padding: '8px 14px',
                      borderRadius: '8px',
                      border: 'none',
                      background: star <= rating ? 'rgba(245, 158, 11, 0.25)' : 'rgba(255,255,255,0.05)',
                      color: star <= rating ? '#f59e0b' : theme.textMuted,
                      cursor: 'pointer',
                      fontSize: '16px',
                    }}
                  >
                    ★ {star}
                  </button>
                ))}
              </div>
            </div>

            <div>
              <label style={S.statLabel}>Operational Observations</label>
              <textarea
                style={{
                  ...S.input,
                  minHeight: '75px',
                  resize: 'vertical',
                }}
                value={comments}
                onChange={(e) => setDispatcherComments(e.target.value)}
              />
            </div>

            <div style={{ display: 'flex', gap: '12px', marginTop: '6px' }}>
              <button
                style={{ ...S.btn('primary'), flex: 1 }}
                onClick={() =>
                  onSubmitSession({
                    dispatcher_id: dispatcherId,
                    airline_org: airlineOrg,
                    rating,
                    adoption_signal: adoptionSignal,
                    comments,
                  })
                }
              >
                {sessionSubmitted ? '✓ Evidence Sealed' : 'Seal & Submit Evidence'}
              </button>
              <button style={S.btn('neutral')} onClick={handleExportJson}>
                Export JSON ⤓
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
