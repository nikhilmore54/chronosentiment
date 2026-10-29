import React, { useState, useEffect } from 'react';
import rp600Data from '../data/rp600_time_machine_decision_tapes.json';
import staticCryptoData from '../data/rp601_live_crypto_decision_tapes.json';

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
  position_side?: string;
  entry_signal?: string;
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
  information_cutoff_policy?: string;
  last_updated_utc?: string;
  engine_version: string;
  interpreter_config: string;
  total_events?: number;
  total_cards?: number;
  events?: DecisionCard[];
  cards?: DecisionCard[];
  tapes?: Tape[];
}

export const DecisionSurfacePanel: React.FC = () => {
  const [activeMode, setActiveMode] = useState<'OBSERVE' | 'REPLAY' | 'COMPARE'>('OBSERVE');
  const [universe, setUniverse] = useState<'CRYPTO' | 'EQUITY'>('CRYPTO');
  
  // Selected instrument for detailed view
  const [selectedCryptoIndex, setSelectedCryptoIndex] = useState<number>(0);
  const [selectedCryptoTapeIndex, setSelectedCryptoTapeIndex] = useState<number>(0);
  const [selectedEquityTapeIndex, setSelectedEquityTapeIndex] = useState<number>(0);
  const [replayStepIndex, setReplayStepIndex] = useState<number>(0);

  // Dynamic live crypto tape state
  const [cryptoTape, setCryptoTape] = useState<Tape>(staticCryptoData as unknown as Tape);
  const [lastPollTime, setLastPollTime] = useState<string>('');

  // Continuous background polling loop (fetches fresh cards every 10 seconds)
  useEffect(() => {
    const pollLiveTape = async () => {
      try {
        const res = await fetch('/rp601_live_crypto_decision_tapes.json?t=' + Date.now());
        if (res.ok) {
          const freshTape: Tape = await res.json();
          setCryptoTape(freshTape);
          setLastPollTime(new Date().toLocaleTimeString());
        }
      } catch (err) {
        // Fallback to imported static data if network fails
      }
    };

    pollLiveTape();
    const interval = setInterval(pollLiveTape, 10000);
    return () => clearInterval(interval);
  }, []);

  // Data sources
  const cryptoCards: DecisionCard[] = cryptoTape.cards || (staticCryptoData as unknown as Tape).cards || [];
  const cryptoTapes: Tape[] = cryptoTape.tapes || (staticCryptoData as unknown as Tape).tapes || [];
  const equityTapes: Tape[] = rp600Data as unknown as Tape[];

  // Current replay tape resolution
  const currentReplayTape: Tape = universe === 'CRYPTO'
    ? (cryptoTapes[selectedCryptoTapeIndex] || cryptoTapes[0])
    : (equityTapes[selectedEquityTapeIndex] || equityTapes[0]);

  const replayEvents: DecisionCard[] = currentReplayTape?.events || [];

  const activeCard: DecisionCard = activeMode === 'REPLAY'
    ? (replayEvents[replayStepIndex] || replayEvents[0])
    : universe === 'CRYPTO'
      ? (cryptoCards[selectedCryptoIndex] || cryptoCards[0])
      : (equityTapes[selectedEquityTapeIndex]?.events?.[0] || (rp600Data[0] as unknown as Tape).events?.[0] || cryptoCards[0]);

  // Universe-level recommendation summary statistics
  const currentCards = universe === 'CRYPTO' ? cryptoCards : equityEvents;
  const exitCount = currentCards.filter(c => c.decision_state === 'EXIT').length;
  const watchCount = currentCards.filter(c => c.decision_state === 'WATCH').length;
  const holdCount = currentCards.filter(c => c.decision_state === 'HOLD').length;

  // Helper color badges and action directives
  const getRecommendationDetails = (state: string) => {
    switch (state) {
      case 'EXIT':
        return {
          title: 'RECOMMENDATION: EXIT / CLOSE POSITION',
          directive: 'Close active position or tighten protective stop-loss immediately. Structural reversal boundaries confirmed across intraday and macro horizons.',
          color: '#ef4444',
          bgGradient: 'linear-gradient(135deg, rgba(153, 27, 27, 0.4) 0%, rgba(15, 23, 42, 0.9) 100%)',
          border: '1px solid rgba(239, 68, 68, 0.4)',
          icon: '🛑'
        };
      case 'WATCH':
        return {
          title: 'RECOMMENDATION: WATCH / STANDBY',
          directive: 'Maintain standby observation. Multi-horizon trajectories exhibit evidence conflict (intraday pullback vs macro trend). Await cross-resolution alignment.',
          color: '#f59e0b',
          bgGradient: 'linear-gradient(135deg, rgba(133, 77, 14, 0.4) 0%, rgba(15, 23, 42, 0.9) 100%)',
          border: '1px solid rgba(245, 158, 11, 0.4)',
          icon: '⚖️'
        };
      case 'HOLD':
        return {
          title: 'RECOMMENDATION: HOLD / MAINTAIN POSITION',
          directive: 'Maintain active position. Multi-resolution alignment confirms positive momentum stability across intraday and macro horizons.',
          color: '#10b981',
          bgGradient: 'linear-gradient(135deg, rgba(6, 95, 70, 0.4) 0%, rgba(15, 23, 42, 0.9) 100%)',
          border: '1px solid rgba(16, 185, 129, 0.4)',
          icon: '🟢'
        };
      default:
        return {
          title: 'RECOMMENDATION: NO ACTION',
          directive: 'Insufficient evidence to form a definitive recommendation.',
          color: '#94a3b8',
          bgGradient: 'linear-gradient(135deg, rgba(30, 41, 59, 0.4) 0%, rgba(15, 23, 42, 0.9) 100%)',
          border: '1px solid rgba(148, 163, 184, 0.3)',
          icon: 'ℹ️'
        };
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
            {cryptoTape.information_cutoff_policy === 'LIVE_CONTINUOUS_POLLING_DAEMON' && (
              <span style={{
                fontSize: '11px',
                fontWeight: 700,
                padding: '3px 8px',
                borderRadius: '9999px',
                backgroundColor: 'rgba(16, 185, 129, 0.2)',
                color: '#10b981',
                border: '1px solid rgba(16, 185, 129, 0.4)'
              }}>
                🟢 LIVE CONTINUOUS DAEMON ACTIVE
              </span>
            )}
          </div>
          <p style={{ margin: '4px 0 0 0', fontSize: '13px', color: '#94a3b8' }}>
            Read-Only Presentation Layer over Canonical DecisionState & DecisionTape Substrate
            {cryptoTape.last_updated_utc && ` (Last Engine Pass: ${cryptoTape.last_updated_utc})`}
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
                setReplayStepIndex(0);
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

      {/* Universe Recommendation Summary Bar */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '12px',
        marginBottom: '20px'
      }}>
        <div style={{
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          border: '1px solid rgba(239, 68, 68, 0.25)',
          borderRadius: '8px',
          padding: '12px 16px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between'
        }}>
          <div>
            <div style={{ fontSize: '11px', fontWeight: 700, color: '#fca5a5', textTransform: 'uppercase' }}>🔴 ACTION REQUIRED (EXIT)</div>
            <div style={{ fontSize: '18px', fontWeight: 800, color: '#ef4444', marginTop: '2px' }}>{exitCount} Assets</div>
          </div>
          <span style={{ fontSize: '20px' }}>🛑</span>
        </div>

        <div style={{
          backgroundColor: 'rgba(245, 158, 11, 0.1)',
          border: '1px solid rgba(245, 158, 11, 0.25)',
          borderRadius: '8px',
          padding: '12px 16px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between'
        }}>
          <div>
            <div style={{ fontSize: '11px', fontWeight: 700, color: '#fef08a', textTransform: 'uppercase' }}>🟡 UNDER MONITORING (WATCH)</div>
            <div style={{ fontSize: '18px', fontWeight: 800, color: '#f59e0b', marginTop: '2px' }}>{watchCount} Assets</div>
          </div>
          <span style={{ fontSize: '20px' }}>⚖️</span>
        </div>

        <div style={{
          backgroundColor: 'rgba(16, 185, 129, 0.1)',
          border: '1px solid rgba(16, 185, 129, 0.25)',
          borderRadius: '8px',
          padding: '12px 16px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between'
        }}>
          <div>
            <div style={{ fontSize: '11px', fontWeight: 700, color: '#6ee7b7', textTransform: 'uppercase' }}>🟢 STABLE TREND (HOLD)</div>
            <div style={{ fontSize: '18px', fontWeight: 800, color: '#10b981', marginTop: '2px' }}>{holdCount} Assets</div>
          </div>
          <span style={{ fontSize: '20px' }}>✅</span>
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

          {/* HERO RECOMMENDATION BANNER */}
          {(() => {
            const rec = getRecommendationDetails(activeCard.decision_state);
            return (
              <div style={{
                background: rec.bgGradient,
                border: rec.border,
                borderRadius: '12px',
                padding: '20px 24px',
                marginBottom: '20px',
                boxShadow: '0 10px 20px -5px rgba(0, 0, 0, 0.3)',
                display: 'flex',
                alignItems: 'flex-start',
                gap: '16px'
              }}>
                <div style={{ fontSize: '36px', lineHeight: 1 }}>{rec.icon}</div>
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: '18px', fontWeight: 800, color: rec.color, letterSpacing: '0.02em' }}>
                    {rec.title}
                  </div>
                  <div style={{ fontSize: '14px', fontWeight: 500, color: '#f8fafc', marginTop: '6px', lineHeight: '1.5' }}>
                    {rec.directive}
                  </div>
                </div>
              </div>
            );
          })()}

          {/* EXPLICIT POSITION SIDE & ENTRY SIGNAL BAR */}
          <div style={{
            display: 'grid',
            gridTemplateColumns: '1fr 2fr',
            gap: '12px',
            marginBottom: '20px'
          }}>
            <div style={{
              backgroundColor: 'rgba(2, 132, 199, 0.15)',
              border: '1px solid rgba(56, 189, 248, 0.4)',
              borderRadius: '10px',
              padding: '14px 18px',
              display: 'flex',
              alignItems: 'center',
              gap: '12px'
            }}>
              <span style={{ fontSize: '24px' }}>🔵</span>
              <div>
                <div style={{ fontSize: '11px', fontWeight: 700, color: '#93c5fd', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                  POSITION SIDE
                </div>
                <div style={{ fontSize: '18px', fontWeight: 800, color: '#38bdf8', marginTop: '2px' }}>
                  {activeCard.position_side || 'LONG'}
                </div>
              </div>
            </div>

            <div style={{
              backgroundColor: 'rgba(30, 41, 59, 0.6)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              borderRadius: '10px',
              padding: '14px 18px',
              display: 'flex',
              alignItems: 'center',
              gap: '12px'
            }}>
              <span style={{ fontSize: '24px' }}>⚡</span>
              <div>
                <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                  ENTRY SIGNAL DIRECTIVE
                </div>
                <div style={{ fontSize: '15px', fontWeight: 700, color: '#f8fafc', marginTop: '2px' }}>
                  {activeCard.entry_signal || (activeCard.decision_state === 'HOLD' ? 'LONG POSITION MONITORING (ACTIVE)' : activeCard.decision_state === 'EXIT' ? 'CLOSE LONG POSITION (EXIT)' : 'STANDBY / AWAIT ALIGNMENT')}
                </div>
              </div>
            </div>
          </div>

          {/* Primary Decision Card */}
          <div style={{
            backgroundColor: 'rgba(15, 23, 42, 0.8)',
            border: '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: '12px',
            padding: '24px',
            position: 'relative',
            overflow: 'hidden'
          }}>
            {/* Top Bar: Symbol & Price */}
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
                <div style={{ fontSize: '24px', fontWeight: 800, color: '#f8fafc' }}>
                  ${activeCard.current_price?.toLocaleString()}
                </div>
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
          {/* Controls Panel */}
          <div style={{
            backgroundColor: '#0f172a',
            border: '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: '12px',
            padding: '24px',
            marginBottom: '24px',
            boxShadow: '0 10px 30px rgba(0,0,0,0.5)'
          }}>
            {/* Header Title & Info */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '12px', marginBottom: '20px' }}>
              <div>
                <h3 style={{ margin: 0, fontSize: '18px', fontWeight: 800, color: '#38bdf8', display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <span>⏱️</span> Time Machine Replay: {currentReplayTape?.symbol || currentReplayTape?.tape_id}
                </h3>
                <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '4px' }}>
                  Cutoff Firewall Policy: <strong style={{ color: '#38bdf8' }}>{currentReplayTape?.information_cutoff_policy || 'STRICT_T0_FIREWALL_VERIFIED'}</strong> | Engine: <strong style={{ color: '#10b981' }}>{currentReplayTape?.engine_version || 'v0.1.0'}</strong> ({currentReplayTape?.interpreter_config || 'Config A'})
                </div>
              </div>
              <div style={{
                fontSize: '13px',
                fontWeight: 800,
                color: '#38bdf8',
                backgroundColor: 'rgba(56, 189, 248, 0.15)',
                padding: '8px 16px',
                borderRadius: '8px',
                border: '1px solid rgba(56, 189, 248, 0.3)'
              }}>
                Step {replayEvents.length > 0 ? replayStepIndex + 1 : 0} of {replayEvents.length} Point-in-Time Events
              </div>
            </div>

            {/* Universe & Asset Selection Sub-Bar */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '14px', marginBottom: '24px', backgroundColor: '#1e293b', padding: '16px', borderRadius: '10px', border: '1px solid rgba(255,255,255,0.06)' }}>
              {/* Universe Selector Toggle */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flexWrap: 'wrap' }}>
                <span style={{ fontSize: '12px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                  REPLAY UNIVERSE:
                </span>
                <div style={{ display: 'flex', gap: '8px' }}>
                  <button
                    onClick={() => {
                      setUniverse('CRYPTO');
                      setSelectedCryptoTapeIndex(0);
                      setReplayStepIndex(0);
                    }}
                    style={{
                      padding: '6px 14px',
                      borderRadius: '6px',
                      border: '1px solid ' + (universe === 'CRYPTO' ? '#38bdf8' : 'rgba(255,255,255,0.1)'),
                      backgroundColor: universe === 'CRYPTO' ? '#0ea5e9' : 'transparent',
                      color: universe === 'CRYPTO' ? '#ffffff' : '#94a3b8',
                      fontWeight: 700,
                      fontSize: '12px',
                      cursor: 'pointer'
                    }}
                  >
                    🪙 LIQUID CRYPTO TAPES (24/7)
                  </button>
                  <button
                    onClick={() => {
                      setUniverse('EQUITY');
                      setSelectedEquityTapeIndex(0);
                      setReplayStepIndex(0);
                    }}
                    style={{
                      padding: '6px 14px',
                      borderRadius: '6px',
                      border: '1px solid ' + (universe === 'EQUITY' ? '#38bdf8' : 'rgba(255,255,255,0.1)'),
                      backgroundColor: universe === 'EQUITY' ? '#0ea5e9' : 'transparent',
                      color: universe === 'EQUITY' ? '#ffffff' : '#94a3b8',
                      fontWeight: 700,
                      fontSize: '12px',
                      cursor: 'pointer'
                    }}
                  >
                    📈 SPOT EQUITY TAPES (RP-600)
                  </button>
                </div>
              </div>

              {/* Ticker Selector Ribbon */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                <span style={{ fontSize: '12px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                  SELECT ASSET TAPE:
                </span>
                {universe === 'CRYPTO' ? (
                  cryptoTapes.map((t, idx) => (
                    <button
                      key={t.tape_id || idx}
                      onClick={() => {
                        setSelectedCryptoTapeIndex(idx);
                        setReplayStepIndex(0);
                      }}
                      style={{
                        padding: '6px 12px',
                        borderRadius: '6px',
                        border: '1px solid ' + (selectedCryptoTapeIndex === idx ? '#38bdf8' : 'rgba(255,255,255,0.08)'),
                        backgroundColor: selectedCryptoTapeIndex === idx ? 'rgba(56, 189, 248, 0.2)' : '#0f172a',
                        color: selectedCryptoTapeIndex === idx ? '#38bdf8' : '#cbd5e1',
                        fontWeight: selectedCryptoTapeIndex === idx ? 800 : 500,
                        fontSize: '12px',
                        cursor: 'pointer'
                      }}
                    >
                      {t.symbol}
                    </button>
                  ))
                ) : (
                  equityTapes.map((t, idx) => (
                    <button
                      key={t.tape_id || idx}
                      onClick={() => {
                        setSelectedEquityTapeIndex(idx);
                        setReplayStepIndex(0);
                      }}
                      style={{
                        padding: '6px 12px',
                        borderRadius: '6px',
                        border: '1px solid ' + (selectedEquityTapeIndex === idx ? '#38bdf8' : 'rgba(255,255,255,0.08)'),
                        backgroundColor: selectedEquityTapeIndex === idx ? 'rgba(56, 189, 248, 0.2)' : '#0f172a',
                        color: selectedEquityTapeIndex === idx ? '#38bdf8' : '#cbd5e1',
                        fontWeight: selectedEquityTapeIndex === idx ? 800 : 500,
                        fontSize: '12px',
                        cursor: 'pointer'
                      }}
                    >
                      {t.symbol}
                    </button>
                  ))
                )}
              </div>
            </div>

            {/* Interactive Timeline Range Slider */}
            <div style={{ marginBottom: '20px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', color: '#94a3b8', marginBottom: '8px', fontWeight: 600 }}>
                <span>T₀ (Start): <strong style={{ color: '#f8fafc' }}>{replayEvents[0]?.datetime || 'N/A'}</strong></span>
                <span style={{ color: '#38bdf8', fontWeight: 800 }}>Active Step UTC: {activeCard?.datetime}</span>
                <span>T_final (End): <strong style={{ color: '#f8fafc' }}>{replayEvents[replayEvents.length - 1]?.datetime || 'N/A'}</strong></span>
              </div>
              <input
                type="range"
                min={0}
                max={Math.max(0, replayEvents.length - 1)}
                value={replayStepIndex}
                onChange={(e) => setReplayStepIndex(Number(e.target.value))}
                style={{
                  width: '100%',
                  height: '8px',
                  borderRadius: '4px',
                  accentColor: '#38bdf8',
                  cursor: 'pointer'
                }}
              />
            </div>

            {/* Step Ribbon Buttons */}
            {replayEvents.length > 0 && (
              <div style={{ display: 'flex', gap: '6px', overflowX: 'auto', paddingBottom: '12px', marginBottom: '20px' }}>
                {replayEvents.map((evt, idx) => {
                  const rec = getRecommendationDetails(evt.decision_state);
                  const isSelected = idx === replayStepIndex;
                  return (
                    <button
                      key={evt.card_id || idx}
                      onClick={() => setReplayStepIndex(idx)}
                      style={{
                        padding: '6px 10px',
                        borderRadius: '6px',
                        border: isSelected ? `2px solid ${rec.color}` : '1px solid rgba(255,255,255,0.1)',
                        backgroundColor: isSelected ? rec.color + '30' : '#1e293b',
                        color: isSelected ? rec.color : '#94a3b8',
                        fontSize: '11px',
                        fontWeight: 700,
                        cursor: 'pointer',
                        whiteSpace: 'nowrap',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '4px'
                      }}
                    >
                      <span>{rec.icon}</span>
                      <span>Step {idx + 1}</span>
                      <span style={{ opacity: 0.7 }}>({evt.datetime?.slice(11, 16)})</span>
                    </button>
                  );
                })}
              </div>
            )}

            {/* Stepper Control Buttons */}
            <div style={{ display: 'flex', gap: '12px', alignItems: 'center', flexWrap: 'wrap' }}>
              <button
                disabled={replayStepIndex === 0}
                onClick={() => setReplayStepIndex(prev => Math.max(0, prev - 1))}
                style={{
                  padding: '8px 18px',
                  borderRadius: '6px',
                  border: '1px solid rgba(255, 255, 255, 0.15)',
                  backgroundColor: replayStepIndex === 0 ? '#1e293b' : '#0284c7',
                  color: replayStepIndex === 0 ? '#64748b' : '#ffffff',
                  fontWeight: 700,
                  fontSize: '13px',
                  cursor: replayStepIndex === 0 ? 'not-allowed' : 'pointer'
                }}
              >
                ◀ Previous Step
              </button>
              <button
                disabled={replayStepIndex >= replayEvents.length - 1}
                onClick={() => setReplayStepIndex(prev => Math.min(replayEvents.length - 1, prev + 1))}
                style={{
                  padding: '8px 18px',
                  borderRadius: '6px',
                  border: '1px solid rgba(255, 255, 255, 0.15)',
                  backgroundColor: replayStepIndex >= replayEvents.length - 1 ? '#1e293b' : '#0284c7',
                  color: replayStepIndex >= replayEvents.length - 1 ? '#64748b' : '#ffffff',
                  fontWeight: 700,
                  fontSize: '13px',
                  cursor: replayStepIndex >= replayEvents.length - 1 ? 'not-allowed' : 'pointer'
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
                  fontWeight: 600,
                  fontSize: '13px',
                  cursor: 'pointer'
                }}
              >
                🔄 Reset to T₀
              </button>
              <button
                onClick={() => setReplayStepIndex(Math.max(0, replayEvents.length - 1))}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  backgroundColor: '#334155',
                  color: '#cbd5e1',
                  fontWeight: 600,
                  fontSize: '13px',
                  cursor: 'pointer'
                }}
              >
                ⏭️ Jump to Latest (T_final)
              </button>
            </div>
          </div>

          {/* Render Active Point-in-Time Card with Hero Recommendation & Side */}
          {activeCard && (
            <div>
              {/* HERO RECOMMENDATION BANNER IN REPLAY */}
              {(() => {
                const rec = getRecommendationDetails(activeCard.decision_state);
                return (
                  <div style={{
                    background: rec.bgGradient,
                    border: rec.border,
                    borderRadius: '12px',
                    padding: '20px 24px',
                    marginBottom: '20px',
                    display: 'flex',
                    alignItems: 'flex-start',
                    gap: '16px'
                  }}>
                    <div style={{ fontSize: '36px', lineHeight: 1 }}>{rec.icon}</div>
                    <div style={{ flex: 1 }}>
                      <div style={{ fontSize: '18px', fontWeight: 800, color: rec.color, letterSpacing: '0.02em' }}>
                        {rec.title}
                      </div>
                      <div style={{ fontSize: '14px', fontWeight: 500, color: '#f8fafc', marginTop: '6px', lineHeight: '1.5' }}>
                        {rec.directive}
                      </div>
                    </div>
                  </div>
                );
              })()}

              {/* Position Side & Entry Signal Bar */}
              <div style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
                gap: '16px',
                marginBottom: '20px'
              }}>
                <div style={{
                  backgroundColor: 'rgba(2, 132, 199, 0.15)',
                  border: '1px solid rgba(56, 189, 248, 0.4)',
                  borderRadius: '10px',
                  padding: '14px 18px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px'
                }}>
                  <span style={{ fontSize: '24px' }}>🔵</span>
                  <div>
                    <div style={{ fontSize: '11px', fontWeight: 700, color: '#93c5fd', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                      POSITION SIDE
                    </div>
                    <div style={{ fontSize: '18px', fontWeight: 800, color: '#38bdf8', marginTop: '2px' }}>
                      {activeCard.position_side || 'LONG'}
                    </div>
                  </div>
                </div>

                <div style={{
                  backgroundColor: 'rgba(30, 41, 59, 0.6)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '10px',
                  padding: '14px 18px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px'
                }}>
                  <span style={{ fontSize: '24px' }}>⚡</span>
                  <div>
                    <div style={{ fontSize: '11px', fontWeight: 700, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                      ENTRY SIGNAL DIRECTIVE
                    </div>
                    <div style={{ fontSize: '15px', fontWeight: 700, color: '#f8fafc', marginTop: '2px' }}>
                      {activeCard.entry_signal || (activeCard.decision_state === 'HOLD' ? 'LONG POSITION MONITORING (ACTIVE)' : activeCard.decision_state === 'EXIT' ? 'CLOSE LONG POSITION (EXIT)' : 'STANDBY / AWAIT ALIGNMENT')}
                    </div>
                  </div>
                </div>
              </div>

              {/* Point-in-Time Decision Card Body */}
              <div style={{
                backgroundColor: 'rgba(15, 23, 42, 0.8)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '12px',
                padding: '24px'
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                  <div>
                    <h4 style={{ margin: 0, fontSize: '20px', fontWeight: 800, color: '#f8fafc' }}>{activeCard.instrument}</h4>
                    <span style={{ fontSize: '13px', color: '#94a3b8', fontFamily: 'monospace' }}>Point-in-Time UTC: {activeCard.datetime}</span>
                  </div>
                  <div style={{ fontSize: '22px', fontWeight: 800, color: '#38bdf8' }}>
                    {activeCard.instrument.includes('-USD') ? `$${activeCard.current_price?.toFixed(2)}` : `₹${activeCard.current_price?.toFixed(2)}`}
                  </div>
                </div>

                {/* Resolution Scores Grid */}
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px', marginBottom: '20px' }}>
                  {(['15m', '30m', '1d'] as const).map(h => (
                    <div key={h} style={{ backgroundColor: '#1e293b', padding: '14px', borderRadius: '8px', textAlign: 'center', border: '1px solid rgba(255,255,255,0.05)' }}>
                      <div style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700, textTransform: 'uppercase' }}>Horizon {h}</div>
                      <div style={{ fontSize: '16px', fontWeight: 800, color: getLabelBadgeColor(activeCard.resolutions[h].label), marginTop: '4px' }}>
                        {activeCard.resolutions[h].label}
                      </div>
                      <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', fontFamily: 'monospace' }}>
                        Score: {activeCard.resolutions[h].score}
                      </div>
                    </div>
                  ))}
                </div>

                {/* Cross Resolution & Risk State */}
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', marginBottom: '20px' }}>
                  <div style={{ backgroundColor: '#0f172a', padding: '12px 16px', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
                    <span style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>CROSS-RESOLUTION ALIGNMENT</span>
                    <div style={{ fontSize: '14px', fontWeight: 700, color: '#38bdf8', marginTop: '2px' }}>{activeCard.cross_resolution_relationship}</div>
                  </div>
                  <div style={{ backgroundColor: '#0f172a', padding: '12px 16px', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
                    <span style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 700 }}>CANONICAL RISK STATE</span>
                    <div style={{ fontSize: '14px', fontWeight: 700, color: '#f59e0b', marginTop: '2px' }}>{activeCard.risk_state}</div>
                  </div>
                </div>

                {/* Canonical Evidence Bullets */}
                <div style={{ backgroundColor: '#0f172a', padding: '16px', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: '#94a3b8', fontWeight: 700, letterSpacing: '0.05em' }}>CANONICAL EVIDENCE JUSTIFICATION:</div>
                  <ul style={{ margin: '10px 0 0 0', paddingLeft: '20px', fontSize: '13px', color: '#cbd5e1', lineHeight: '1.6' }}>
                    {activeCard.evidence.map((bullet, idx) => (
                      <li key={idx} style={{ marginBottom: '4px' }}>
                        {bullet.startsWith('• ') ? bullet.substring(2) : bullet}
                      </li>
                    ))}
                  </ul>
                </div>
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
                  <th style={{ padding: '12px 16px' }}>Side</th>
                  <th style={{ padding: '12px 16px' }}>Decision Time (UTC)</th>
                  <th style={{ padding: '12px 16px' }}>15m Intraday</th>
                  <th style={{ padding: '12px 16px' }}>30m Horizon</th>
                  <th style={{ padding: '12px 16px' }}>1d Macro</th>
                  <th style={{ padding: '12px 16px' }}>Cross-Resolution State</th>
                  <th style={{ padding: '12px 16px' }}>Signal & Directive</th>
                </tr>
              </thead>
              <tbody>
                {(universe === 'CRYPTO' ? cryptoCards : equityEvents).map((card, idx) => {
                  const rec = getRecommendationDetails(card.decision_state);
                  return (
                    <tr key={card.card_id || idx} style={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                      <td style={{ padding: '12px 16px', fontWeight: 700, color: '#f8fafc' }}>{card.instrument}</td>
                      <td style={{ padding: '12px 16px' }}>
                        <span style={{ padding: '3px 8px', borderRadius: '4px', fontSize: '11px', fontWeight: 800, backgroundColor: 'rgba(56, 189, 248, 0.15)', color: '#38bdf8', border: '1px solid rgba(56, 189, 248, 0.3)' }}>
                          {card.position_side || 'LONG'}
                        </span>
                      </td>
                      <td style={{ padding: '12px 16px', color: '#38bdf8', fontFamily: 'monospace', fontSize: '12px' }}>
                        {card.datetime || (card.timestamp ? new Date(card.timestamp * 1000).toISOString() : 'N/A')}
                      </td>
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
                          padding: '4px 10px',
                          borderRadius: '6px',
                          fontSize: '12px',
                          fontWeight: 800,
                          backgroundColor: rec.color + '20',
                          color: rec.color,
                          border: `1px solid ${rec.color}50`
                        }}>
                          {rec.icon} {card.entry_signal || card.decision_state}
                        </span>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
};

export default DecisionSurfacePanel;
