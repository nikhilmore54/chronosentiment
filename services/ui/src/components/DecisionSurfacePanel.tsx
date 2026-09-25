import React, { useState } from 'react';
import rp600Data from '../data/rp600_time_machine_decision_tapes.json';
import rp601Data from '../data/rp601_live_crypto_decision_tapes.json';

interface Resolution {
  label: string;
  score: number;
}

interface DecisionCard {
  card_id: string;
  instrument: string;
  timestamp: number;
  datetime: string;
  current_price: number;
  domain_status?: string;
  resolutions: {
    '15m': Resolution;
    '30m': Resolution;
    '1d': Resolution;
  };
  cross_resolution_relationship: string;
  risk_state: string;
  decision_state: string;
  evidence: string[];
}

interface Tape {
  tape_id: string;
  symbol?: string;
  universe?: string;
  domain_status?: string;
  engine_version: string;
  interpreter_config: string;
  total_events?: number;
  total_cards?: number;
  events?: DecisionCard[];
  cards?: DecisionCard[];
}

export const DecisionSurfacePanel: React.FC = () => {
  const [activeMode, setActiveMode] = useState<'OBSERVE' | 'REPLAY' | 'COMPARE'>('OBSERVE');
  const [universe, setUniverse] = useState<'CRYPTO' | 'EQUITY'>('CRYPTO');
  
  // Selected instrument for detailed view
  const [selectedCryptoIndex, setSelectedCryptoIndex] = useState<number>(0);
  const [selectedEquityTapeIndex, setSelectedEquityTapeIndex] = useState<number>(0);
  const [replayStepIndex, setReplayStepIndex] = useState<number>(0);

  // Data sources
  const cryptoCards: DecisionCard[] = (rp601Data as unknown as Tape).cards || [];
  const equityTapes: Tape[] = rp600Data as unknown as Tape[];

  const currentEquityTape = equityTapes[selectedEquityTapeIndex] || equityTapes[0];
  const equityEvents = currentEquityTape?.events || [];

  const activeCard: DecisionCard = universe === 'CRYPTO' 
    ? (cryptoCards[selectedCryptoIndex] || cryptoCards[0])
    : (equityEvents[replayStepIndex] || equityEvents[0]);

  // Helper color badges
  const getDecisionStateColor = (state: string) => {
    switch (state) {
      case 'EXIT': return { bg: '#991b1b', text: '#fca5a5', border: '#ef4444' };
      case 'WATCH': return { bg: '#854d0e', text: '#fef08a', border: '#eab308' };
      case 'HOLD': return { bg: '#065f46', text: '#6ee7b7', border: '#10b981' };
      default: return { bg: '#1e293b', text: '#cbd5e1', border: '#64748b' };
    }
  };

  const getLabelBadgeColor = (label: string) => {
    switch (label) {
      case 'REVERSAL': return '#f43f5e';
      case 'PULLBACK': return '#f59e0b';
      case 'CONTINUATION': return '#10b981';
      case 'INSUFFICIENT': return '#64748b';
      default: return '#94a3b8';
    }
  };

  return (
    <div style={{
      fontFamily: 'Inter, system-ui, -apple-system, sans-serif',
      color: '#f8fafc',
      backgroundColor: '#090d16',
      borderRadius: '12px',
      padding: '24px',
      border: '1px solid rgba(255, 255, 255, 0.08)',
      boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.5)'
    }}>
      {/* Top Header & Mode Navigation */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '16px',
        marginBottom: '24px',
        borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
        paddingBottom: '16px'
      }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <h2 style={{ margin: 0, fontSize: '20px', fontWeight: 700, letterSpacing: '-0.02em', color: '#38bdf8' }}>
              RP-600 / RP-601 Decision Surface
            </h2>
            <span style={{
              fontSize: '11px',
              fontWeight: 600,
              padding: '3px 8px',
              borderRadius: '9999px',
              backgroundColor: 'rgba(56, 189, 248, 0.15)',
              color: '#38bdf8',
              border: '1px solid rgba(56, 189, 248, 0.3)'
            }}>
              FROZEN SUBSTRATE v0.1.0
            </span>
          </div>
          <p style={{ margin: '4px 0 0 0', fontSize: '13px', color: '#94a3b8' }}>
            Read-Only Presentation Layer over Canonical DecisionState & DecisionTape Substrate
          </p>
        </div>

        {/* Mode Selector Tabs */}
        <div style={{
          display: 'flex',
          backgroundColor: '#0f172a',
          padding: '4px',
          borderRadius: '8px',
          border: '1px solid rgba(255, 255, 255, 0.06)'
        }}>
          {(['OBSERVE', 'REPLAY', 'COMPARE'] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => {
                setActiveMode(mode);
                if (mode === 'OBSERVE') setUniverse('CRYPTO');
                if (mode === 'REPLAY') setUniverse('EQUITY');
              }}
              style={{
                padding: '8px 16px',
                borderRadius: '6px',
                border: 'none',
                backgroundColor: activeMode === mode ? '#0ea5e9' : 'transparent',
                color: activeMode === mode ? '#ffffff' : '#94a3b8',
                fontWeight: 600,
                fontSize: '13px',
                cursor: 'pointer',
                transition: 'all 0.2s ease',
                boxShadow: activeMode === mode ? '0 4px 12px rgba(14, 165, 233, 0.3)' : 'none'
              }}
            >
              {mode === 'OBSERVE' && '🟢 LIVE OBSERVE (24/7)'}
              {mode === 'REPLAY' && '⏱️ TIME MACHINE REPLAY'}
              {mode === 'COMPARE' && '📊 MULTI-HORIZON COMPARE'}
            </button>
          ))}
        </div>
      </div>

      {/* Universe & Asset Selection Toolbar */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '12px',
        marginBottom: '20px',
        backgroundColor: 'rgba(15, 23, 42, 0.6)',
        padding: '12px 16px',
        borderRadius: '8px',
        border: '1px solid rgba(255, 255, 255, 0.05)'
      }}>
        {/* Universe Selector */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ fontSize: '12px', fontWeight: 600, color: '#64748b', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            Domain Target:
          </span>
          <button
            onClick={() => setUniverse('CRYPTO')}
            style={{
              padding: '6px 12px',
              borderRadius: '6px',
              border: universe === 'CRYPTO' ? '1px solid #38bdf8' : '1px solid rgba(255, 255, 255, 0.1)',
              backgroundColor: universe === 'CRYPTO' ? 'rgba(56, 189, 248, 0.15)' : 'transparent',
              color: universe === 'CRYPTO' ? '#38bdf8' : '#94a3b8',
              fontSize: '12px',
              fontWeight: 600,
              cursor: 'pointer'
            }}
          >
            ⚡ Liquid Crypto 24/7 (RP-601)
          </button>
          <button
            onClick={() => setUniverse('EQUITY')}
            style={{
              padding: '6px 12px',
              borderRadius: '6px',
              border: universe === 'EQUITY' ? '1px solid #10b981' : '1px solid rgba(255, 255, 255, 0.1)',
              backgroundColor: universe === 'EQUITY' ? 'rgba(16, 185, 129, 0.15)' : 'transparent',
              color: universe === 'EQUITY' ? '#10b981' : '#94a3b8',
              fontSize: '12px',
              fontWeight: 600,
              cursor: 'pointer'
            }}
          >
            📈 Equity Time Machine (RP-600)
          </button>
        </div>

        {/* Instrument Selector Dropdown */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
          <span style={{ fontSize: '12px', color: '#94a3b8' }}>Select Instrument:</span>
          {universe === 'CRYPTO' ? (
            <select
              value={selectedCryptoIndex}
              onChange={(e) => setSelectedCryptoIndex(Number(e.target.value))}
              style={{
                backgroundColor: '#1e293b',
                color: '#f8fafc',
                border: '1px solid rgba(255, 255, 255, 0.15)',
                padding: '6px 12px',
                borderRadius: '6px',
                fontSize: '13px',
                fontWeight: 600,
                outline: 'none',
                cursor: 'pointer'
              }}
            >
              {cryptoCards.map((card, idx) => (
                <option key={card.instrument} value={idx}>
                  {card.instrument} — ${card.current_price?.toLocaleString()} ({card.decision_state})
                </option>
              ))}
            </select>
          ) : (
            <select
              value={selectedEquityTapeIndex}
              onChange={(e) => {
                setSelectedEquityTapeIndex(Number(e.target.value));
                setReplayStepIndex(0);
              }}
              style={{
                backgroundColor: '#1e293b',
                color: '#f8fafc',
                border: '1px solid rgba(255, 255, 255, 0.15)',
                padding: '6px 12px',
                borderRadius: '6px',
                fontSize: '13px',
                fontWeight: 600,
                outline: 'none',
                cursor: 'pointer'
              }}
            >
              {equityTapes.map((tape, idx) => (
                <option key={tape.tape_id} value={idx}>
                  {tape.symbol} ({tape.total_events} Simulation Steps)
                </option>
              ))}
            </select>
          )}
        </div>
      </div>

      {/* Main Content View based on Mode */}
      {activeMode === 'OBSERVE' && activeCard && (
        <div>
          {/* Domain Mandate Warning Badge */}
          {activeCard.domain_status && (
            <div style={{
              backgroundColor: 'rgba(245, 158, 11, 0.1)',
              border: '1px solid rgba(245, 158, 11, 0.3)',
              color: '#fbbf24',
              padding: '10px 14px',
              borderRadius: '8px',
              fontSize: '12px',
              fontWeight: 600,
              marginBottom: '20px',
              display: 'flex',
              alignItems: 'center',
              gap: '8px'
            }}>
              <span>⚠️</span>
              <span><strong>MANDATE:</strong> {activeCard.domain_status}</span>
            </div>
          )}

          {/* Primary Decision Card */}
          <div style={{
            backgroundColor: 'rgba(15, 23, 42, 0.8)',
            border: '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: '12px',
            padding: '24px',
            position: 'relative',
            overflow: 'hidden'
          }}>
            {/* Top Bar: Symbol, Price & Decision State */}
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'flex-start',
              marginBottom: '20px',
              borderBottom: '1px solid rgba(255, 255, 255, 0.06)',
              paddingBottom: '16px'
            }}>
              <div>
                <div style={{ fontSize: '24px', fontWeight: 800, color: '#ffffff', letterSpacing: '-0.02em' }}>
                  {activeCard.instrument}
                </div>
                <div style={{ fontSize: '13px', color: '#94a3b8', marginTop: '2px' }}>
                  Recorded UTC: <code style={{ color: '#38bdf8' }}>{activeCard.datetime}</code> (Epoch: {activeCard.timestamp})
                </div>
              </div>

              <div style={{ textAlign: 'right' }}>
                <div style={{ fontSize: '22px', fontWeight: 700, color: '#f8fafc' }}>
                  ${activeCard.current_price?.toLocaleString()}
                </div>
                {/* Decision Badge */}
                {(() => {
                  const colors = getDecisionStateColor(activeCard.decision_state);
                  return (
                    <div style={{
                      display: 'inline-block',
                      marginTop: '6px',
                      padding: '4px 12px',
                      borderRadius: '6px',
                      backgroundColor: colors.bg,
                      color: colors.text,
                      border: `1px solid ${colors.border}`,
                      fontSize: '13px',
                      fontWeight: 700,
                      letterSpacing: '0.05em'
                    }}>
                      {activeCard.decision_state}
                    </div>
                  );
                })()}
              </div>
            </div>

            {/* Resolutions Multi-Horizon Grid */}
            <div style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
              gap: '12px',
              marginBottom: '20px'
            }}>
              {(['15m', '30m', '1d'] as const).map((horizon) => {
                const res = activeCard.resolutions[horizon];
                const badgeColor = getLabelBadgeColor(res.label);
                return (
                  <div key={horizon} style={{
                    backgroundColor: 'rgba(30, 41, 59, 0.5)',
                    border: '1px solid rgba(255, 255, 255, 0.08)',
                    borderRadius: '8px',
                    padding: '14px',
                    textAlign: 'center'
                  }}>
                    <div style={{ fontSize: '11px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase' }}>
                      Horizon {horizon}
                    </div>
                    <div style={{
                      fontSize: '16px',
                      fontWeight: 800,
                      color: badgeColor,
                      margin: '6px 0'
                    }}>
                      {res.label}
                    </div>
                    <div style={{ fontSize: '12px', color: '#94a3b8' }}>
                      Score: <span style={{ fontWeight: 600, color: res.score >= 0 ? '#10b981' : '#f43f5e' }}>{res.score >= 0 ? `+${res.score}` : res.score}</span>
                    </div>
                  </div>
                );
              })}
            </div>

            {/* Alignment & Risk Relationship Banner */}
            <div style={{
              display: 'grid',
              gridTemplateColumns: '1fr 1fr',
              gap: '12px',
              marginBottom: '24px'
            }}>
              <div style={{
                backgroundColor: 'rgba(30, 41, 59, 0.4)',
                border: '1px solid rgba(255, 255, 255, 0.06)',
                borderRadius: '8px',
                padding: '12px 16px'
              }}>
                <div style={{ fontSize: '11px', color: '#64748b', fontWeight: 600 }}>CROSS-RESOLUTION RELATIONSHIP</div>
                <div style={{ fontSize: '14px', fontWeight: 700, color: '#38bdf8', marginTop: '2px' }}>
                  {activeCard.cross_resolution_relationship}
                </div>
              </div>

              <div style={{
                backgroundColor: 'rgba(30, 41, 59, 0.4)',
                border: '1px solid rgba(255, 255, 255, 0.06)',
                borderRadius: '8px',
                padding: '12px 16px'
              }}>
                <div style={{ fontSize: '11px', color: '#64748b', fontWeight: 600 }}>CANONICAL RISK STATE</div>
                <div style={{ fontSize: '14px', fontWeight: 700, color: '#f59e0b', marginTop: '2px' }}>
                  {activeCard.risk_state}
                </div>
              </div>
            </div>

            {/* Canonical Evidence Bullet List */}
            <div>
              <div style={{ fontSize: '12px', fontWeight: 700, color: '#94a3b8', marginBottom: '10px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                Canonical Evidence Justifications
              </div>
              <div style={{
                backgroundColor: '#0f172a',
                border: '1px solid rgba(255, 255, 255, 0.06)',
                borderRadius: '8px',
                padding: '16px'
              }}>
                <ul style={{ margin: 0, paddingLeft: '20px', fontSize: '13px', color: '#cbd5e1', lineHeight: '1.8' }}>
                  {activeCard.evidence.map((bullet, idx) => (
                    <li key={idx} style={{ marginBottom: '4px' }}>
                      {bullet.startsWith('• ') ? bullet.substring(2) : bullet}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* REPLAY Mode */}
      {activeMode === 'REPLAY' && (
        <div>
          <div style={{
            backgroundColor: '#0f172a',
            border: '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: '12px',
            padding: '20px',
            marginBottom: '20px'
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
              <div>
                <h3 style={{ margin: 0, fontSize: '16px', color: '#38bdf8' }}>
                  Time Machine Prospective Simulation: {currentEquityTape?.symbol}
                </h3>
                <span style={{ fontSize: '12px', color: '#64748b' }}>
                  Cutoff Firewall Policy: {currentEquityTape?.information_cutoff_policy}
                </span>
              </div>
              <div style={{ fontSize: '14px', fontWeight: 700, color: '#f8fafc', backgroundColor: '#1e293b', padding: '6px 14px', borderRadius: '6px' }}>
                Step {replayStepIndex + 1} of {equityEvents.length}
              </div>
            </div>

            {/* Stepper Controls */}
            <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
              <button
                disabled={replayStepIndex === 0}
                onClick={() => setReplayStepIndex(prev => Math.max(0, prev - 1))}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid rgba(255, 255, 255, 0.15)',
                  backgroundColor: replayStepIndex === 0 ? '#1e293b' : '#0284c7',
                  color: '#ffffff',
                  fontWeight: 600,
                  cursor: replayStepIndex === 0 ? 'not-allowed' : 'pointer'
                }}
              >
                ◀ Previous Step
              </button>
              <button
                disabled={replayStepIndex >= equityEvents.length - 1}
                onClick={() => setReplayStepIndex(prev => Math.min(equityEvents.length - 1, prev + 1))}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid rgba(255, 255, 255, 0.15)',
                  backgroundColor: replayStepIndex >= equityEvents.length - 1 ? '#1e293b' : '#0284c7',
                  color: '#ffffff',
                  fontWeight: 600,
                  cursor: replayStepIndex >= equityEvents.length - 1 ? 'not-allowed' : 'pointer'
                }}
              >
                Next Step ▶
              </button>
              <button
                onClick={() => setReplayStepIndex(0)}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  backgroundColor: '#334155',
                  color: '#cbd5e1',
                  cursor: 'pointer'
                }}
              >
                🔄 Reset to T₀
              </button>
            </div>
          </div>

          {/* Render Active Point-in-Time Card */}
          {activeCard && (
            <div style={{
              backgroundColor: 'rgba(15, 23, 42, 0.8)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              borderRadius: '12px',
              padding: '24px'
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '16px' }}>
                <div>
                  <h4 style={{ margin: 0, fontSize: '18px', color: '#f8fafc' }}>{activeCard.instrument}</h4>
                  <span style={{ fontSize: '12px', color: '#94a3b8' }}>Point-in-Time UTC: {activeCard.datetime}</span>
                </div>
                <div style={{ fontSize: '18px', fontWeight: 700, color: '#f8fafc' }}>
                  ₹{activeCard.current_price?.toFixed(2)}
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '12px', marginBottom: '16px' }}>
                {(['15m', '30m', '1d'] as const).map(h => (
                  <div key={h} style={{ backgroundColor: '#1e293b', padding: '12px', borderRadius: '6px', textAlign: 'center' }}>
                    <div style={{ fontSize: '11px', color: '#64748b' }}>Horizon {h}</div>
                    <div style={{ fontSize: '14px', fontWeight: 700, color: getLabelBadgeColor(activeCard.resolutions[h].label) }}>
                      {activeCard.resolutions[h].label}
                    </div>
                  </div>
                ))}
              </div>

              <div style={{ backgroundColor: '#0f172a', padding: '14px', borderRadius: '6px' }}>
                <div style={{ fontSize: '12px', color: '#94a3b8', fontWeight: 600 }}>CANONICAL EVIDENCE:</div>
                <ul style={{ margin: '8px 0 0 0', paddingLeft: '20px', fontSize: '13px', color: '#cbd5e1' }}>
                  {activeCard.evidence.map((b, idx) => (
                    <li key={idx}>{b}</li>
                  ))}
                </ul>
              </div>
            </div>
          )}
        </div>
      )}

      {/* COMPARE Mode */}
      {activeMode === 'COMPARE' && (
        <div>
          <h3 style={{ margin: '0 0 16px 0', fontSize: '16px', color: '#38bdf8' }}>
            Multi-Horizon Cross-Resolution Alignment Matrix
          </h3>

          <div style={{ overflowX: 'auto' }}>
            <table style={{
              width: '100%',
              borderCollapse: 'collapse',
              textAlign: 'left',
              fontSize: '13px',
              backgroundColor: '#0f172a',
              borderRadius: '8px',
              overflow: 'hidden'
            }}>
              <thead>
                <tr style={{ backgroundColor: '#1e293b', color: '#94a3b8', borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                  <th style={{ padding: '12px 16px' }}>Asset Ticker</th>
                  <th style={{ padding: '12px 16px' }}>15m Intraday</th>
                  <th style={{ padding: '12px 16px' }}>30m Horizon</th>
                  <th style={{ padding: '12px 16px' }}>1d Macro</th>
                  <th style={{ padding: '12px 16px' }}>Cross-Resolution State</th>
                  <th style={{ padding: '12px 16px' }}>Decision State</th>
                </tr>
              </thead>
              <tbody>
                {(universe === 'CRYPTO' ? cryptoCards : equityEvents).map((card, idx) => (
                  <tr key={card.card_id || idx} style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                    <td style={{ padding: '12px 16px', fontWeight: 700, color: '#f8fafc' }}>{card.instrument}</td>
                    <td style={{ padding: '12px 16px', color: getLabelBadgeColor(card.resolutions['15m'].label), fontWeight: 600 }}>
                      {card.resolutions['15m'].label} ({card.resolutions['15m'].score})
                    </td>
                    <td style={{ padding: '12px 16px', color: getLabelBadgeColor(card.resolutions['30m'].label), fontWeight: 600 }}>
                      {card.resolutions['30m'].label} ({card.resolutions['30m'].score})
                    </td>
                    <td style={{ padding: '12px 16px', color: getLabelBadgeColor(card.resolutions['1d'].label), fontWeight: 600 }}>
                      {card.resolutions['1d'].label} ({card.resolutions['1d'].score})
                    </td>
                    <td style={{ padding: '12px 16px', color: '#38bdf8', fontWeight: 600 }}>
                      {card.cross_resolution_relationship}
                    </td>
                    <td style={{ padding: '12px 16px' }}>
                      <span style={{
                        padding: '3px 8px',
                        borderRadius: '4px',
                        fontSize: '11px',
                        fontWeight: 700,
                        backgroundColor: getDecisionStateColor(card.decision_state).bg,
                        color: getDecisionStateColor(card.decision_state).text
                      }}>
                        {card.decision_state}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
};

export default DecisionSurfacePanel;
