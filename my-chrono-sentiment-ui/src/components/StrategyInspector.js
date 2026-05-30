import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import StrategyColumn from './StrategyColumn';
import ComparisonPanels from './ComparisonPanels';
import { apiUrl } from '../services/api';

function normalizeTraceEvent(raw) {
  if (!raw || typeof raw !== 'object') return raw;
  const p = raw.payload != null && typeof raw.payload === 'object' ? raw.payload : {};
  return { ...raw, ...p, type: raw.type };
}

// ARTIFACT-009: normalizeNarrativeBlock — field-name bridge between backend snake_case
// and the camelCase shape expected by NarrativeBlock.js / StrategyColumn.js.
// Handles both canonical decision_trace.schema.json fields (block_id, sequence_id,
// parent_block_id, block_type, divergence_score) and legacy inspect_strategy response fields.
// Sunset condition: backend emits camelCase narrative_blocks[] natively, or a
// canonical JS SDK handles the mapping. Registered: 2026-05-25.
function normalizeNarrativeBlock(block) {
  if (!block || typeof block !== 'object') return block;
  // timestamp_ns is the canonical field name per event.schema.json (chrono:schema:event:v1).
  // Fall back to legacy `timestamp` field for backward compatibility (ARTIFACT-009 bridge).
  const rawTs = block.timestamp_ns ?? block.timestamp ?? null;
  return {
    // Canonical: decision_trace.schema.json uses `sequence_id`; legacy uses `id`
    id:             block.sequence_id ?? block.id ?? null,
    // Canonical: decision_trace.schema.json uses `parent_block_id` (UUID) or
    // event.schema.json uses `parent_sequence_id` (integer); legacy uses `parentId`
    parentId:       block.parent_sequence_id ?? block.parentId ?? null,
    group:          block.group,
    narrative:      block.narrative,
    // Canonical: decision_trace.schema.json uses `block_type`
    blockType:      block.block_type ?? block.blockType ?? null,
    // Canonical: decision_trace.schema.json uses `divergence_score`
    divergenceScore: block.divergence_score ?? block.divergenceScore ?? null,
    timestamp_ns:   rawTs,
    isKeyEvent:     block.is_key_event    ?? block.isKeyEvent    ?? false,
    keyEventMarker: block.key_event_marker ?? block.keyEventMarker ?? null,
  };
}

function normalizeInspectResponse(data) {
  if (!data || typeof data !== 'object') return data;
  return {
    ...data,
    execution_trace:  Array.isArray(data.execution_trace)  ? data.execution_trace.map(normalizeTraceEvent)   : data.execution_trace,
    decision_trace:   Array.isArray(data.decision_trace)   ? data.decision_trace.map(normalizeTraceEvent)    : data.decision_trace,
    event_sequence:   Array.isArray(data.event_sequence)   ? data.event_sequence.map(normalizeTraceEvent)    : data.event_sequence,
    // Consume backend-certified narrative_blocks[] — Law One: UI must not synthesize narrative.
    narrative_blocks: Array.isArray(data.narrative_blocks) ? data.narrative_blocks.map(normalizeNarrativeBlock) : [],
  };
}

// ARTIFACT-002 (groupAndNarrateEvents) eliminated 2026-05-25.
// getOrderId() was its sole dependent — removed with it.
// Backend emits certified narrative_blocks[] via POST /inspect_strategy.

// ARTIFACT-010: compareNarrativeBlocks — frontend divergence analysis between two
// backend-certified narrative_blocks[] arrays. Sunset condition: backend emits
// divergence_analysis[] in CanonicalInspectResponse. Registered: 2026-05-25.
const compareNarrativeBlocks = (b1, b2) => {
  const divergenceStatements = [];
  const maxLength = Math.max(b1.length, b2.length);
  for (let i = 0; i < maxLength; i++) {
    const block1 = b1[i], block2 = b2[i];
    if (!block1 && block2) { divergenceStatements.push({ type: 'missing_s1', message: `Strategy 1 ended earlier. Strategy 2 has block '${block2.group}' (Seq ${block2.id}) at step ${i + 1}.`, block: block2 }); }
    else if (block1 && !block2) { divergenceStatements.push({ type: 'missing_s2', message: `Strategy 2 ended earlier. Strategy 1 has block '${block1.group}' (Seq ${block1.id}) at step ${i + 1}.`, block: block1 }); }
    else if (block1 && block2) {
      if (block1.group !== block2.group) divergenceStatements.push({ type: 'group_type_divergence', message: `Event group divergence at step ${i + 1}: Strategy 1 has '${block1.group}' (Seq ${block1.id}) vs Strategy 2 has '${block2.group}' (Seq ${block2.id}).`, block1, block2 });
      if (block1.narrative !== block2.narrative) divergenceStatements.push({ type: 'narrative_content_divergence', message: `Narrative content divergence at step ${i + 1} for group '${block1.group}': S1: '${block1.narrative}' vs S2: '${block2.narrative}'.`, block1, block2 });
      if (block1.id !== block2.id) divergenceStatements.push({ type: 'sequence_id_timing_divergence', message: `Sequence ID timing divergence at step ${i + 1}: Strategy 1 has Seq ${block1.id} vs Strategy 2 has Seq ${block2.id}.`, block1, block2 });
      if (block1.parentId !== block2.parentId) divergenceStatements.push({ type: 'causal_parent_divergence', message: `Causal parent divergence at step ${i + 1} for group '${block1.group}': S1 parent Seq ${block1.parentId || 'None'} vs S2 parent Seq ${block2.parentId || 'None'}.`, block1, block2 });
    }
  }
  return divergenceStatements;
};

// ARTIFACT-011: getExecutionSummary — frontend execution summary derived from
// backend narrative_blocks[]. Sunset condition: backend emits execution_summary
// object in CanonicalInspectResponse. Registered: 2026-05-25.
const getExecutionSummary = (narrativeBlocks) => ({
  totalSteps: narrativeBlocks.length,
  partialFills: narrativeBlocks.filter(b => b.group === 'Execution' && b.narrative.includes('partially filled')).length || 0,
  queueProgressions: narrativeBlocks.filter(b => b.group === 'Queue Progression').length || 0,
  hasQueueProgression: narrativeBlocks.some(b => b.group === 'Queue Progression'),
  totalFills: narrativeBlocks.filter(b => b.group === 'Execution' && b.narrative.includes('fully executed')).length || 0,
});

const CONFIDENCE_LEVELS = { HIGH: 'High Confidence', MEDIUM: 'Medium Confidence', LOW: 'Low Confidence' };

const StrategyInspector = ({ strategyId: propStrategyId, seed: propSeed, strategyId2: propStrategyId2, seed2: propSeed2, onReset }) => {
  const [strategyId,  setStrategyId]  = useState(propStrategyId  || '');
  const [seed,        setSeed]        = useState(propSeed        || 42);
  const [strategyId2, setStrategyId2] = useState(propStrategyId2 || '');
  const [seed2,       setSeed2]       = useState(propSeed2       || 42);

  const [inspectionResult,  setInspectionResult]  = useState(null);
  const [inspectionResult2, setInspectionResult2] = useState(null);
  const [loading,  setLoading]  = useState(false);
  const [loading2, setLoading2] = useState(false);
  const [error,    setError]    = useState(null);
  const [error2,   setError2]   = useState(null);

  const [selectedSeqId,    setSelectedSeqId]    = useState(null);
  const [selectedMaxSeqId, setSelectedMaxSeqId] = useState(null);
  const [showRawEvents,    setShowRawEvents]    = useState(false);

  const rawEventRefs = useRef({});
const handleInspectStrategy = useCallback(async (strategyNum) => {
    const isFirst = strategyNum === 1;
    const currentStrategyId = isFirst ? strategyId : strategyId2;
    const currentSeed = isFirst ? seed : seed2;
    if (!currentStrategyId || currentSeed === null) return;
    isFirst ? setLoading(true) : setLoading2(true);
    isFirst ? setError(null) : setError2(null);
    setSelectedSeqId(null);
    try {
      const response = await fetch(apiUrl('/inspect_strategy'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ strategy_id: currentStrategyId, seed: Number(currentSeed) }),
      });
      if (!response.ok) {
        const errText = await response.text();
        let msg = 'Failed to inspect strategy';
        try { const d = JSON.parse(errText); msg = d.message || d.error || msg; } catch { if (errText) msg = errText; }
        throw new Error(msg);
      }
      const rawJson = await response.json();
      const data = normalizeInspectResponse(rawJson);
      if (isFirst) {
        setInspectionResult(data);
        if (data.execution_trace && data.execution_trace.length > 0) {
          setSelectedMaxSeqId(Math.max(...data.execution_trace.map(e => e.sequence_id)));
        }
      } else {
        setInspectionResult2(data);
      }
    } catch (err) {
      isFirst ? setError(err.message) : setError2(err.message);
    } finally {
      isFirst ? setLoading(false) : setLoading2(false);
    }
  }, [strategyId, strategyId2, seed, seed2]);

  useEffect(() => {
    setStrategyId(propStrategyId || '');
    setSeed(propSeed || 42);
    if (propStrategyId && propStrategyId !== strategyId) { setStrategyId2(''); setSeed2(42); }
  }, [propStrategyId, propSeed]);

  useEffect(() => { setStrategyId2(propStrategyId2 || ''); setSeed2(propSeed2 || 42); }, [propStrategyId2, propSeed2]);

  useEffect(() => {
    if (strategyId && seed !== null) handleInspectStrategy(1);
    else { setInspectionResult(null); setError(null); }
  }, [strategyId, seed, handleInspectStrategy]);

  useEffect(() => {
    if (strategyId2 && seed2 !== null) handleInspectStrategy(2);
    else { setInspectionResult2(null); setError2(null); }
  }, [strategyId2, seed2, handleInspectStrategy]);

  useEffect(() => {
    if (selectedSeqId && showRawEvents && rawEventRefs.current[selectedSeqId]) {
      rawEventRefs.current[selectedSeqId].scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
  }, [selectedSeqId, showRawEvents]);

  // Law One: consume backend-certified narrative_blocks[] directly.
  // normalizeInspectResponse() maps snake_case → camelCase (ARTIFACT-009).
  const allNarrativeBlocks1 = useMemo(() => inspectionResult?.narrative_blocks  ?? [], [inspectionResult?.narrative_blocks]);
  const allNarrativeBlocks2 = useMemo(() => inspectionResult2?.narrative_blocks ?? [], [inspectionResult2?.narrative_blocks]);

  const eventMap = useMemo(() => {
    const map = {};
    [...allNarrativeBlocks1, ...allNarrativeBlocks2].forEach(block => { map[block.id] = block; });
    return map;
  }, [allNarrativeBlocks1, allNarrativeBlocks2]);

  const getCausalChain = (seqId) => {
    const chain = new Set();
    let current = seqId;
    while (current !== null && eventMap[current]) { chain.add(current); current = eventMap[current].parentId; }
    return chain;
  };

  const activeChain = selectedSeqId ? getCausalChain(selectedSeqId) : new Set();

  // Slider filtering: pure observer operation — filters backend blocks by sequence position.
  // No synthesis; block content is unchanged from backend emission.
  const narratedExecutionTrace1 = useMemo(() => {
    const blocks = inspectionResult?.narrative_blocks ?? [];
    return selectedMaxSeqId !== null ? blocks.filter(b => (b.id ?? 0) <= selectedMaxSeqId) : blocks;
  }, [inspectionResult?.narrative_blocks, selectedMaxSeqId]);
  const narratedExecutionTrace2 = useMemo(() => {
    const blocks = inspectionResult2?.narrative_blocks ?? [];
    return selectedMaxSeqId !== null ? blocks.filter(b => (b.id ?? 0) <= selectedMaxSeqId) : blocks;
  }, [inspectionResult2?.narrative_blocks, selectedMaxSeqId]);

  const minSeqId = useMemo(() => inspectionResult?.execution_trace?.length > 0 ? Math.min(...inspectionResult.execution_trace.map(e => e.sequence_id)) : 0, [inspectionResult?.execution_trace]);
  const maxAvailableSeqId = useMemo(() => inspectionResult?.execution_trace?.length > 0 ? Math.max(...inspectionResult.execution_trace.map(e => e.sequence_id)) : 0, [inspectionResult?.execution_trace]);

  // getGroupColorClass — maps canonical group enum values (decision_trace.schema.json)
  // to CSS modifier classes. Handles both canonical uppercase (INTENT, QUEUE, EXECUTION,
  // SETTLEMENT, GOVERNANCE) and legacy mixed-case values for backward compatibility.
  const getGroupColorClass = (group) => {
    if (!group) return 'other';
    switch (group.toUpperCase()) {
      case 'INTENT':      return 'intent';
      case 'QUEUE':       return 'queue';
      case 'QUEUE ENTRY': return 'queue';
      case 'QUEUE PROGRESSION': return 'queue';
      case 'EXECUTION':   return 'execution';
      case 'SETTLEMENT':  return 'execution';
      case 'GOVERNANCE':  return 'other';
      default:            return 'other';
    }
  };

  const isDualMode = !!(inspectionResult && inspectionResult2);
  const currentMaxSeqId = selectedMaxSeqId !== null ? selectedMaxSeqId : maxAvailableSeqId;

  const divergenceStatements = useMemo(() => isDualMode ? compareNarrativeBlocks(allNarrativeBlocks1, allNarrativeBlocks2) : [], [isDualMode, allNarrativeBlocks1, allNarrativeBlocks2]);

  // Divergence accumulation at current replay position:
  // filter to divergences where at least one involved block is within the visible window.
  // Pure derivation from backend-certified blocks — no synthesis.
  const visibleDivergences = useMemo(() => {
    if (!isDualMode) return [];
    return divergenceStatements.filter(d => {
      const id1 = d.block1?.id ?? d.block?.id ?? null;
      const id2 = d.block2?.id ?? null;
      const withinWindow = (id) => id !== null && id <= currentMaxSeqId;
      return withinWindow(id1) || withinWindow(id2);
    });
  }, [divergenceStatements, isDualMode, currentMaxSeqId]);

  // Type distribution of visible divergences
  const divergenceTypeCounts = useMemo(() => {
    const counts = {};
    visibleDivergences.forEach(d => { counts[d.type] = (counts[d.type] ?? 0) + 1; });
    return counts;
  }, [visibleDivergences]);

  const DIVERGENCE_SHORT = {
    group_type_divergence:         'Group',
    narrative_content_divergence:  'Narrative',
    sequence_id_timing_divergence: 'Timing',
    causal_parent_divergence:      'Causal',
    missing_s1:                    'Missing S1',
    missing_s2:                    'Missing S2',
  };
  const executionSummary1 = useMemo(() => getExecutionSummary(allNarrativeBlocks1), [allNarrativeBlocks1]);
  const executionSummary2 = useMemo(() => getExecutionSummary(allNarrativeBlocks2), [allNarrativeBlocks2]);

  let finalVerdict = 'No clear execution advantage — trade-offs observed.';
  let confidenceLevel = CONFIDENCE_LEVELS.LOW;
  let confidenceColorClass = 'red';
  let confidenceReason = '';

  const s1Completed = executionSummary1.totalFills > 0;
  const s2Completed = executionSummary2.totalFills > 0;
  const s1StrictlyBetter = executionSummary1.totalSteps <= executionSummary2.totalSteps && executionSummary1.partialFills <= executionSummary2.partialFills && (executionSummary1.totalSteps < executionSummary2.totalSteps || executionSummary1.partialFills < executionSummary2.partialFills);
  const s2StrictlyBetter = executionSummary2.totalSteps <= executionSummary1.totalSteps && executionSummary2.partialFills <= executionSummary1.partialFills && (executionSummary2.totalSteps < executionSummary1.totalSteps || executionSummary2.partialFills < executionSummary1.partialFills);

  if (s1Completed && !s2Completed) { finalVerdict = 'Strategy 1 provides more complete execution overall.'; confidenceLevel = CONFIDENCE_LEVELS.HIGH; confidenceColorClass = 'grn'; confidenceReason = 'Strategy 1 completed execution while Strategy 2 did not.'; }
  else if (!s1Completed && s2Completed) { finalVerdict = 'Strategy 2 provides more complete execution overall.'; confidenceLevel = CONFIDENCE_LEVELS.HIGH; confidenceColorClass = 'grn'; confidenceReason = 'Strategy 2 completed execution while Strategy 1 did not.'; }
  else if (s1Completed && s2Completed) {
    if (s1StrictlyBetter) { finalVerdict = 'Strategy 1 provides more efficient execution overall.'; confidenceLevel = CONFIDENCE_LEVELS.HIGH; confidenceColorClass = 'grn'; confidenceReason = 'Strategy 1 strictly dominates Strategy 2 in execution speed and fragmentation.'; }
    else if (s2StrictlyBetter) { finalVerdict = 'Strategy 2 provides more efficient execution overall.'; confidenceLevel = CONFIDENCE_LEVELS.HIGH; confidenceColorClass = 'grn'; confidenceReason = 'Strategy 2 strictly dominates Strategy 1 in execution speed and fragmentation.'; }
  }

  if (confidenceLevel === CONFIDENCE_LEVELS.LOW && s1Completed === s2Completed) {
    if (executionSummary1.totalSteps < executionSummary2.totalSteps && executionSummary1.partialFills > executionSummary2.partialFills) confidenceReason = 'Strategy 1 is faster but more fragmented; Strategy 2 is slower but cleaner.';
    else if (executionSummary2.totalSteps < executionSummary1.totalSteps && executionSummary2.partialFills > executionSummary1.partialFills) confidenceReason = 'Strategy 2 is faster but more fragmented; Strategy 1 is slower but cleaner.';
    else if (executionSummary1.hasQueueProgression !== executionSummary2.hasQueueProgression) confidenceReason = 'Execution differs due to queue interaction.';
    else if (executionSummary1.totalSteps === executionSummary2.totalSteps && executionSummary1.partialFills !== executionSummary2.partialFills) confidenceReason = 'Both strategies executed in similar time but differ in fragmentation.';
    else confidenceReason = 'Multiple trade-offs observed across execution speed, queue behavior, and fragmentation.';
    finalVerdict = 'Trade-offs observed across execution characteristics.';
    confidenceLevel = CONFIDENCE_LEVELS.MEDIUM;
    confidenceColorClass = 'amb';
  }
  if (confidenceLevel === CONFIDENCE_LEVELS.LOW) confidenceReason = 'No clear dominance or significant trade-offs found.';

  // Derive certification state from backend response (Law One: never compute client-side)
  const certState = inspectionResult?.certification_state ?? null;
  const certReason = inspectionResult?.certification_reason ?? null;
  const certBadgeColor = certState === 'CERTIFIED' ? 'var(--grn)' : certState === 'DEGRADED' ? 'var(--amb)' : certState === 'PARTIAL' ? 'var(--amb)' : certState === 'INVALID' ? 'var(--red)' : 'var(--tm)';
  const certBgColor   = certState === 'CERTIFIED' ? 'var(--gdim)' : certState === 'DEGRADED' ? 'var(--adim)' : certState === 'PARTIAL' ? 'var(--adim)' : certState === 'INVALID' ? 'var(--rdim)' : 'var(--card2)';
  const certBorderColor = certState === 'CERTIFIED' ? 'var(--bgrn)' : certState === 'DEGRADED' ? 'var(--bamb)' : certState === 'PARTIAL' ? 'var(--bamb)' : certState === 'INVALID' ? 'var(--bred)' : 'var(--b)';

  // Visible event count at current replay position
  const visibleEventCount1 = narratedExecutionTrace1.length;
  const visibleEventCount2 = narratedExecutionTrace2.length;
  const totalEventCount = Math.max(allNarrativeBlocks1.length, allNarrativeBlocks2.length);

  return (
    <div style={{ display: 'flex', gap: '60px', alignItems: 'flex-start', paddingTop: '20px' }}>
      
      {/* ─── LEFT: Editorial Sidebar (Timeline Anchor & Inputs) ─── */}
      <aside style={{ width: '280px', flexShrink: 0, position: 'sticky', top: '80px' }}>
        <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', marginBottom: '24px' }}>Causal reconstruction</h2>
        
        {/* Timeline anchor */}
        {inspectionResult && (
          <div style={{ marginBottom: '40px', paddingBottom: '30px', borderBottom: '1px solid var(--b)' }}>
            {/* Certification badge */}
            {certState && (
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '16px', padding: '8px 10px', background: certBgColor, border: `1px solid ${certBorderColor}`, borderRadius: 'var(--r8)' }}>
                <span style={{ fontSize: '10px', fontWeight: 700, color: certBadgeColor, fontFamily: 'var(--mono)', letterSpacing: '0.05em' }}>{certState}</span>
                {certReason && (
                  <span style={{ fontSize: '10px', color: 'var(--tm)', flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={certReason}>— {certReason}</span>
                )}
              </div>
            )}
            <div style={{ fontSize: '12px', fontWeight: 500, color: 'var(--t2)', marginBottom: '16px' }}>Replay position</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '12px' }}>
              <input
                type="range"
                className="cs-range"
                min={minSeqId}
                max={maxAvailableSeqId}
                value={selectedMaxSeqId !== null ? selectedMaxSeqId : maxAvailableSeqId}
                onChange={e => setSelectedMaxSeqId(Number(e.target.value))}
              />
            </div>
            {/* Replay position context strip */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
              <span style={{ fontSize: '11px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>Seq {minSeqId}</span>
              <span style={{ fontSize: '13px', color: 'var(--t1)', fontWeight: 600, fontFamily: 'var(--mono)' }}>Seq {currentMaxSeqId}</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ fontSize: '10px', color: 'var(--tm)' }}>
                {isDualMode
                  ? `S1: ${visibleEventCount1} / S2: ${visibleEventCount2} events`
                  : `${visibleEventCount1} of ${totalEventCount} events`}
              </span>
              {selectedMaxSeqId !== null && selectedMaxSeqId < maxAvailableSeqId && (
                <button
                  style={{ fontSize: '10px', color: 'var(--blu)', background: 'none', border: 'none', cursor: 'pointer', padding: 0, fontFamily: 'var(--sans)' }}
                  onClick={() => setSelectedMaxSeqId(maxAvailableSeqId)}
                >
                  Jump to end
                </button>
              )}
            </div>
          </div>
        )}

        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', marginBottom: '24px' }}>
          {/* Strategy 1 */}
          <div style={{ fontSize: '10px', fontWeight: 700, color: 'var(--tm)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: '4px' }}>Strategy 1</div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="strategy_id">Strategy ID</label>
            <input type="text" id="strategy_id" className="cs-input" style={{ background: 'var(--card)', border: '1px solid var(--b)' }} value={strategyId} onChange={e => setStrategyId(e.target.value)} onBlur={() => handleInspectStrategy(1)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="seed_inspect">Seed</label>
            <input type="number" id="seed_inspect" className="cs-input" style={{ background: 'var(--card)', border: '1px solid var(--b)' }} value={seed} onChange={e => setSeed(Number(e.target.value))} onBlur={() => handleInspectStrategy(1)} />
          </div>
        </div>

        {/* Strategy 2 — dual-mode */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', marginBottom: '24px', paddingTop: '16px', borderTop: '1px solid var(--b)' }}>
          <div style={{ fontSize: '10px', fontWeight: 700, color: 'var(--tm)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: '4px' }}>Strategy 2 <span style={{ fontWeight: 400, textTransform: 'none', letterSpacing: 0 }}>(optional)</span></div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="strategy_id2">Strategy ID</label>
            <input type="text" id="strategy_id2" className="cs-input" style={{ background: 'var(--card)', border: '1px solid var(--b)' }} value={strategyId2} onChange={e => setStrategyId2(e.target.value)} onBlur={() => { if (strategyId2) handleInspectStrategy(2); }} placeholder="Leave blank for single mode" />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="seed_inspect2">Seed</label>
            <input type="number" id="seed_inspect2" className="cs-input" style={{ background: 'var(--card)', border: '1px solid var(--b)' }} value={seed2} onChange={e => setSeed2(Number(e.target.value))} onBlur={() => { if (strategyId2) handleInspectStrategy(2); }} />
          </div>
        </div>

        <button className="cs-btn cs-btn-primary" style={{ width: '100%', padding: '10px 0', marginBottom: '12px' }} onClick={() => { handleInspectStrategy(1); if (strategyId2) handleInspectStrategy(2); }} disabled={loading || loading2}>
          {(loading || loading2) ? 'Inspecting...' : 'Reconstruct trace'}
        </button>

        {/* Show raw events toggle */}
        <label className="cs-checkbox-label" style={{ marginBottom: '20px' }}>
          <input
            type="checkbox"
            className="cs-checkbox"
            checked={showRawEvents}
            onChange={e => setShowRawEvents(e.target.checked)}
          />
          Show raw events
        </label>

        {error && (
          <div style={{ padding: '12px', background: 'var(--rdim)', border: '1px solid var(--bred)', borderRadius: 'var(--r8)', color: 'var(--red)', fontSize: '12px', fontFamily: 'var(--mono)' }}>
            {error}
          </div>
        )}
        {error2 && (
          <div style={{ padding: '12px', background: 'var(--rdim)', border: '1px solid var(--bred)', borderRadius: 'var(--r8)', color: 'var(--red)', fontSize: '12px', fontFamily: 'var(--mono)', marginTop: '8px' }}>
            S2: {error2}
          </div>
        )}
      </aside>

      {/* ─── RIGHT: The Forensics Stream ─── */}
      <main style={{ flex: 1, maxWidth: '900px', minHeight: '600px' }}>

        {/* State: Loading skeleton */}
        {(loading || loading2) && (
          <div style={{ padding: '32px', background: 'var(--card)', border: '1px solid var(--b)' }}>
            <div className="cs-skeleton" style={{ height: '14px', width: '100px', marginBottom: '24px' }}></div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
              {[...Array(5)].map((_, i) => (
                <div key={i} style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
                  <div className="cs-skeleton" style={{ height: '12px', width: '60px', flexShrink: 0 }}></div>
                  <div className="cs-skeleton" style={{ height: '12px', flex: 1 }}></div>
                </div>
              ))}
            </div>
            <div style={{ marginTop: '24px', fontSize: '12px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>
              Reconstructing causal trace...
            </div>
          </div>
        )}

        {/* State: Error */}
        {(error || error2) && !loading && !loading2 && (
          <div style={{ padding: '16px', background: 'var(--rdim)', border: '1px solid var(--bred)', borderRadius: 'var(--r8)', color: 'var(--red)', fontSize: '12px', fontFamily: 'var(--mono)', marginBottom: '16px' }}>
            {error || error2}
          </div>
        )}

        {/* State: Pre-Execution idle block */}
        {!inspectionResult && !loading && !loading2 && !error && (
          <div className="cs-empty" style={{ padding: '48px 32px', background: 'var(--card)', border: '1px solid var(--b)', borderRadius: 'var(--r10)', textAlign: 'center' }}>
            <div className="cs-empty-icon">⟳</div>
            <div className="cs-empty-title">No trace loaded</div>
            <div style={{ fontSize: '12px', color: 'var(--tm)', marginTop: '8px', lineHeight: 1.6 }}>
              Enter a strategy ID in the sidebar and press <strong style={{ color: 'var(--t2)' }}>Reconstruct trace</strong> to begin causal replay.
            </div>
          </div>
        )}

      {inspectionResult && !loading && !loading2 && (
        <div style={{ display: 'flex', flexDirection: 'column' }}>
          {/* Execution Model Context Strip */}
          <div style={{ marginBottom: '40px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
              <h2 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--t1)', margin: 0 }}>Causal trace</h2>
              {certState && (
                <span style={{ fontSize: '10px', fontWeight: 700, color: certBadgeColor, fontFamily: 'var(--mono)', letterSpacing: '0.05em', padding: '2px 8px', background: certBgColor, border: `1px solid ${certBorderColor}`, borderRadius: 'var(--r4)' }}>
                  {certState}
                </span>
              )}
            </div>
            <div style={{ display: 'flex', gap: '24px', borderBottom: '1px solid var(--b)', paddingBottom: '20px', flexWrap: 'wrap' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                <span style={{ fontSize: '12px', color: 'var(--t2)' }}>Decision</span>
                <span style={{ color: 'var(--tm)' }}>→</span>
                <span style={{ fontSize: '12px', color: 'var(--t2)' }}>Queue</span>
                <span style={{ color: 'var(--tm)' }}>→</span>
                <span style={{ fontSize: '12px', color: 'var(--t2)' }}>Market interaction</span>
                <span style={{ color: 'var(--tm)' }}>→</span>
                <span style={{ fontSize: '12px', color: 'var(--t2)' }}>Execution</span>
              </div>
              <div style={{ marginLeft: 'auto', fontSize: '11px', color: 'var(--tm)', fontFamily: 'var(--mono)' }}>
                {totalEventCount} event{totalEventCount !== 1 ? 's' : ''} · Seq {minSeqId}–{maxAvailableSeqId}
              </div>
            </div>

            {/* Divergence accumulation summary — only in dual mode */}
            {isDualMode && (
              <div style={{ marginTop: '12px', display: 'flex', alignItems: 'center', gap: '12px', flexWrap: 'wrap' }}>
                <span style={{ fontSize: '10px', color: 'var(--tm)', fontWeight: 500 }}>
                  Divergences at Seq {currentMaxSeqId}:
                </span>
                {visibleDivergences.length === 0 ? (
                  <span style={{ fontSize: '10px', color: 'var(--grn)', fontFamily: 'var(--mono)', fontWeight: 600 }}>none detected</span>
                ) : (
                  <>
                    <span style={{ fontSize: '10px', color: 'var(--red)', fontFamily: 'var(--mono)', fontWeight: 700 }}>
                      {visibleDivergences.length} total
                    </span>
                    {Object.entries(divergenceTypeCounts).map(([type, count]) => (
                      <span key={type} style={{
                        fontSize: '9px', fontFamily: 'var(--mono)', fontWeight: 600,
                        color: (type === 'group_type_divergence' || type === 'causal_parent_divergence' || type === 'missing_s1' || type === 'missing_s2') ? 'var(--red)' : 'var(--amb)',
                        padding: '1px 5px', borderRadius: 'var(--r4)',
                        background: 'rgba(0,0,0,0.08)',
                        border: '1px solid currentColor',
                      }}>
                        {DIVERGENCE_SHORT[type] ?? type} ×{count}
                      </span>
                    ))}
                  </>
                )}
              </div>
            )}
          </div>

          <div className={isDualMode ? 'cs-dual-grid' : ''} style={{ gap: '40px' }}>
            {inspectionResult && (
              <StrategyColumn
                strategyNum={1}
                strategyId={strategyId}
                seed={seed}
                inspectionResult={inspectionResult}
                narratedExecutionTrace={narratedExecutionTrace1}
                rawEventRefs={rawEventRefs}
                activeChain={activeChain}
                eventMap={eventMap}
                showRawEvents={showRawEvents}
                getGroupColorClass={getGroupColorClass}
                divergenceStatements={divergenceStatements}
                setSelectedSeqId={setSelectedSeqId}
                selectedSeqId={selectedSeqId}
              />
            )}
            {isDualMode && inspectionResult2 && (
              <StrategyColumn
                strategyNum={2}
                strategyId={strategyId2}
                seed={seed2}
                inspectionResult={inspectionResult2}
                narratedExecutionTrace={narratedExecutionTrace2}
                rawEventRefs={rawEventRefs}
                activeChain={activeChain}
                eventMap={eventMap}
                showRawEvents={showRawEvents}
                getGroupColorClass={getGroupColorClass}
                divergenceStatements={divergenceStatements}
                setSelectedSeqId={setSelectedSeqId}
                selectedSeqId={selectedSeqId}
              />
            )}
          </div>

          {/* ComparisonPanels — dual-mode execution comparison surface */}
          {isDualMode && (
            <ComparisonPanels
              isDualMode={isDualMode}
              allNarrativeBlocks1={allNarrativeBlocks1}
              allNarrativeBlocks2={allNarrativeBlocks2}
              strategyId={strategyId}
              strategyId2={strategyId2}
              executionSummary1={executionSummary1}
              executionSummary2={executionSummary2}
              finalVerdict={finalVerdict}
              confidenceLevel={confidenceLevel}
              confidenceColorClass={confidenceColorClass}
              confidenceReason={confidenceReason}
              divergenceStatements={divergenceStatements}
            />
          )}
        </div>
      )}
      </main>
    </div>
  );
};

export default StrategyInspector;