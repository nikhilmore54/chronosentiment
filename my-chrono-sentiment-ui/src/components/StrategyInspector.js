import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import StrategyColumn from './StrategyColumn';
import ComparisonPanels from './ComparisonPanels';

function normalizeTraceEvent(raw) {
  if (!raw || typeof raw !== 'object') return raw;
  const p = raw.payload != null && typeof raw.payload === 'object' ? raw.payload : {};
  return { ...raw, ...p, type: raw.type };
}

function normalizeInspectResponse(data) {
  if (!data || typeof data !== 'object') return data;
  return {
    ...data,
    execution_trace: Array.isArray(data.execution_trace) ? data.execution_trace.map(normalizeTraceEvent) : data.execution_trace,
    decision_trace:  Array.isArray(data.decision_trace)  ? data.decision_trace.map(normalizeTraceEvent)  : data.decision_trace,
    event_sequence:  Array.isArray(data.event_sequence)  ? data.event_sequence.map(normalizeTraceEvent)  : data.event_sequence,
  };
}

const getOrderId = (event) => {
  if (event && event.order_id !== undefined && event.order_id !== null) return event.order_id;
  if (event && event.sequence_id !== undefined) return `seq-${event.sequence_id}`;
  return 'N/A';
};

const groupAndNarrateEvents = (trace, maxSeqIdFilter) => {
  if (!trace || trace.length === 0) return [];
  const sortedTrace = [...trace].sort((a, b) => a.sequence_id - b.sequence_id);
  const narrativeBlocks = [];
  let currentOrderId = null;
  let currentQueueProgressionEvents = [];
  let currentQueueParentId = null;

  const flushQueueProgression = () => {
    if (currentQueueProgressionEvents.length > 0) {
      const queueAheads = currentQueueProgressionEvents.map(e => e.queue_ahead !== undefined && e.queue_ahead !== null ? e.queue_ahead : '?');
      narrativeBlocks.push({
        group: 'Queue Progression',
        id: currentQueueProgressionEvents[0].sequence_id,
        parentId: currentQueueProgressionEvents[0].parent_sequence_id,
        timestamp: currentQueueProgressionEvents[0].timestamp,
        narrative: `Order ${getOrderId(currentQueueProgressionEvents[0])} waiting in queue, position improving: ${queueAheads.join(' → ')}`,
        isKeyEvent: queueAheads.includes(0),
        keyEventMarker: queueAheads.includes(0) ? 'Queue cleared' : null,
      });
      currentQueueProgressionEvents = [];
      currentQueueParentId = null;
    }
  };

  const filteredTrace = maxSeqIdFilter !== null ? sortedTrace.filter(e => e.sequence_id <= maxSeqIdFilter) : sortedTrace;
  let firstExecutionEventAdded = false;

  filteredTrace.forEach((event) => {
    const eventOrderId = getOrderId(event);
    if (
      (event.type !== 'QueueProgression' && currentQueueProgressionEvents.length > 0) ||
      (currentOrderId !== null && eventOrderId !== 'N/A' && eventOrderId !== currentOrderId) ||
      (event.type === 'QueueProgression' && currentQueueProgressionEvents.length > 0 && event.parent_sequence_id !== currentQueueProgressionEvents[0].parent_sequence_id)
    ) { flushQueueProgression(); }

    currentOrderId = eventOrderId;
    let isKeyEvent = false, keyEventMarker = null;

    switch (event.type) {
      case 'OrderIntent':
        narrativeBlocks.push({ group: 'Intent', narrative: `Strategy decision: Order ${getOrderId(event)} placed for ${event.quantity} at price ${event.price} (Side: ${event.side}).`, id: event.sequence_id, parentId: event.parent_sequence_id, timestamp: event.timestamp, isKeyEvent, keyEventMarker });
        break;
      case 'OrderEnteredQueue':
        narrativeBlocks.push({ group: 'Queue Entry', narrative: `Order ${getOrderId(event)} entered the queue at position ${event.queue_ahead}.`, id: event.sequence_id, parentId: event.parent_sequence_id, timestamp: event.timestamp, isKeyEvent, keyEventMarker });
        currentQueueParentId = event.parent_sequence_id;
        break;
      case 'QueueProgression':
        if (currentQueueParentId === null || event.parent_sequence_id === currentQueueParentId) {
          currentQueueProgressionEvents.push(event); currentQueueParentId = event.parent_sequence_id;
        } else { flushQueueProgression(); currentQueueProgressionEvents.push(event); currentQueueParentId = event.parent_sequence_id; }
        break;
      case 'PartialFill':
        isKeyEvent = true;
        keyEventMarker = !firstExecutionEventAdded ? 'Execution begins (Partial Fill)' : 'Partial Fill';
        firstExecutionEventAdded = true;
        narrativeBlocks.push({ group: 'Execution', narrative: `Execution started: Order ${getOrderId(event)} partially filled ${event.filled_qty} at price ${event.price}.`, id: event.sequence_id, parentId: event.parent_sequence_id, timestamp: event.timestamp, isKeyEvent, keyEventMarker });
        break;
      case 'OrderFilled':
        if (!firstExecutionEventAdded) { isKeyEvent = true; keyEventMarker = 'Execution begins (Full Fill)'; firstExecutionEventAdded = true; }
        narrativeBlocks.push({ group: 'Execution', narrative: `Order ${getOrderId(event)} fully executed.`, id: event.sequence_id, parentId: event.parent_sequence_id, timestamp: event.timestamp, isKeyEvent, keyEventMarker });
        break;
      case 'MarketEvent': break;
      default:
        narrativeBlocks.push({ group: 'Other', narrative: `Unhandled event type: ${event.type} for order ${eventOrderId}.`, id: event.sequence_id, parentId: event.parent_sequence_id, timestamp: event.timestamp, isKeyEvent, keyEventMarker });
    }
  });

  flushQueueProgression();
  return narrativeBlocks;
};

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
      const response = await fetch('http://localhost:8000/inspect_strategy', {
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

  const allNarrativeBlocks1 = useMemo(() => inspectionResult?.execution_trace ? groupAndNarrateEvents(inspectionResult.execution_trace, null) : [], [inspectionResult?.execution_trace]);
  const allNarrativeBlocks2 = useMemo(() => inspectionResult2?.execution_trace ? groupAndNarrateEvents(inspectionResult2.execution_trace, null) : [], [inspectionResult2?.execution_trace]);

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

  const narratedExecutionTrace1 = useMemo(() => inspectionResult?.execution_trace ? groupAndNarrateEvents(inspectionResult.execution_trace, selectedMaxSeqId) : [], [inspectionResult?.execution_trace, selectedMaxSeqId]);
  const narratedExecutionTrace2 = useMemo(() => inspectionResult2?.execution_trace ? groupAndNarrateEvents(inspectionResult2.execution_trace, selectedMaxSeqId) : [], [inspectionResult2?.execution_trace, selectedMaxSeqId]);

  const minSeqId = useMemo(() => inspectionResult?.execution_trace?.length > 0 ? Math.min(...inspectionResult.execution_trace.map(e => e.sequence_id)) : 0, [inspectionResult?.execution_trace]);
  const maxAvailableSeqId = useMemo(() => inspectionResult?.execution_trace?.length > 0 ? Math.max(...inspectionResult.execution_trace.map(e => e.sequence_id)) : 0, [inspectionResult?.execution_trace]);

  const getGroupColorClass = (group) => {
    switch (group) {
      case 'Intent':           return 'intent';
      case 'Queue Entry':      return 'queue';
      case 'Queue Progression':return 'queue';
      case 'Execution':        return 'execution';
      default:                 return 'other';
    }
  };

  const isDualMode = !!(inspectionResult && inspectionResult2);
  const currentMaxSeqId = selectedMaxSeqId !== null ? selectedMaxSeqId : maxAvailableSeqId;

  const divergenceStatements = useMemo(() => isDualMode ? compareNarrativeBlocks(allNarrativeBlocks1, allNarrativeBlocks2) : [], [isDualMode, allNarrativeBlocks1, allNarrativeBlocks2]);
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

  return (
    <div className="cs-gap-20">
      {/* Header */}
      <div>
        <div className="cs-section-sub">Execution Analysis</div>
        <div className="cs-section-title">
          {isDualMode
            ? `Comparing: ${strategyId || 'N/A'} vs ${strategyId2 || 'N/A'}`
            : 'Strategy Inspector'}
        </div>
      </div>

      {/* Input form */}
      <div className="cs-card">
        <div className="cs-card-title">Strategy Inputs</div>
        <div className="cs-form-grid">
          <div className="cs-field">
            <label className="cs-label" htmlFor="strategy_id">Strategy ID 1</label>
            <input type="text" id="strategy_id" className="cs-input" value={strategyId} onChange={e => setStrategyId(e.target.value)} onBlur={() => handleInspectStrategy(1)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="strategy_id_2">Strategy ID 2 (optional)</label>
            <input type="text" id="strategy_id_2" className="cs-input" value={strategyId2} onChange={e => setStrategyId2(e.target.value)} onBlur={() => handleInspectStrategy(2)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="seed_inspect">Seed 1</label>
            <input type="number" id="seed_inspect" className="cs-input" value={seed} onChange={e => setSeed(Number(e.target.value))} onBlur={() => handleInspectStrategy(1)} />
          </div>
          <div className="cs-field">
            <label className="cs-label" htmlFor="seed_inspect_2">Seed 2 (optional)</label>
            <input type="number" id="seed_inspect_2" className="cs-input" value={seed2} onChange={e => setSeed2(Number(e.target.value))} onBlur={() => handleInspectStrategy(2)} />
          </div>
        </div>
        <div className="cs-row-gap-12">
          <button
            className="cs-btn cs-btn-primary"
            onClick={() => { handleInspectStrategy(1); if (strategyId2) handleInspectStrategy(2); }}
            disabled={loading || loading2}
          >
            {(loading || loading2) ? 'Inspecting…' : isDualMode ? 'Compare Executions' : 'Inspect Strategy'}
          </button>
          <label className="cs-checkbox-label">
            <input type="checkbox" className="cs-checkbox" checked={showRawEvents} onChange={e => setShowRawEvents(e.target.checked)} />
            Show raw events
          </label>
        </div>
      </div>

      {error  && <div className="cs-alert red"><div className="cs-alert-title">Error (Strategy 1)</div><div className="cs-alert-body">{error}</div></div>}
      {error2 && <div className="cs-alert red"><div className="cs-alert-title">Error (Strategy 2)</div><div className="cs-alert-body">{error2}</div></div>}

      {inspectionResult && (
        <div className="cs-gap-16">
          {/* Execution model */}
          <div className="cs-card">
            <div className="cs-card-title">Execution Model</div>
            <p style={{ fontSize: 12, color: 'var(--t2)', fontFamily: 'var(--mono)' }}>
              Decision → Queue → Market Interaction → Execution
            </p>
          </div>

          {/* Timeline banner + slider */}
          <div className="cs-card">
            <div className="cs-card-title">Replay Position</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
              <input
                type="range"
                className="cs-range"
                min={minSeqId}
                max={maxAvailableSeqId}
                value={selectedMaxSeqId !== null ? selectedMaxSeqId : maxAvailableSeqId}
                onChange={e => setSelectedMaxSeqId(Number(e.target.value))}
              />
              <span style={{ fontFamily: 'var(--mono)', fontSize: 12, color: 'var(--blu)', whiteSpace: 'nowrap' }}>
                Seq {currentMaxSeqId} / {maxAvailableSeqId}
              </span>
            </div>
            <p style={{ fontSize: 11, color: 'var(--tm)', fontFamily: 'var(--mono)' }}>
              Showing events Seq {minSeqId} → {currentMaxSeqId}
            </p>
          </div>

          {/* Strategy columns */}
          <div className={isDualMode ? 'cs-dual-grid' : ''}>
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
        </div>
      )}
    </div>
  );
};

export default StrategyInspector;