import React, { useState, useEffect, useRef, useMemo, useCallback } from 'react';
import StrategyColumn from './StrategyColumn';
import ComparisonPanels from './ComparisonPanels';

/** Flatten API EventWrapper `{ type, payload: { ... } }` into shape expected by narrative helpers. */
function normalizeTraceEvent(raw) {
  if (!raw || typeof raw !== 'object') return raw;
  const p = raw.payload != null && typeof raw.payload === 'object' ? raw.payload : {};
  return { ...raw, ...p, type: raw.type };
}

function normalizeInspectResponse(data) {
  if (!data || typeof data !== 'object') return data;
  return {
    ...data,
    execution_trace: Array.isArray(data.execution_trace)
      ? data.execution_trace.map(normalizeTraceEvent)
      : data.execution_trace,
    decision_trace: Array.isArray(data.decision_trace)
      ? data.decision_trace.map(normalizeTraceEvent)
      : data.decision_trace,
    event_sequence: Array.isArray(data.event_sequence)
      ? data.event_sequence.map(normalizeTraceEvent)
      : data.event_sequence,
  };
}

// Helper to get order_id or a fallback
const getOrderId = (event) => {
  if (event && (event.order_id !== undefined && event.order_id !== null)) {
    return event.order_id;
  }
  if (event && event.sequence_id !== undefined) {
    return `seq-${event.sequence_id}`;
  }
  return 'N/A';
};

// Helper to group and narrate events into causal blocks
const groupAndNarrateEvents = (trace, maxSeqIdFilter) => {
  if (!trace || trace.length === 0) return [];

  const sortedTrace = [...trace].sort(
    (a, b) => a.sequence_id - b.sequence_id
  );

  const narrativeBlocks = [];
  let currentOrderId = null;
  let currentQueueProgressionEvents = [];
  let currentQueueParentId = null;

  const flushQueueProgression = () => {
    if (currentQueueProgressionEvents.length > 0) {
      const queueAheads = currentQueueProgressionEvents.map(e => {
        if (e.queue_ahead !== undefined && e.queue_ahead !== null) {
          return e.queue_ahead;
        }
        return '?';
      });

      narrativeBlocks.push({
        group: 'Queue Progression',
        id: currentQueueProgressionEvents[0].sequence_id,
        parentId: currentQueueProgressionEvents[0].parent_sequence_id,
        timestamp: currentQueueProgressionEvents[0].timestamp,
        narrative: `Order ${getOrderId(currentQueueProgressionEvents[0])} waiting in queue, position improving due to market activity: ${queueAheads.join(' → ')}`,
        isKeyEvent: queueAheads.includes(0), // KEY EVENT: Queue cleared
        keyEventMarker: queueAheads.includes(0) ? '⚠ Queue cleared' : null,
      });
      currentQueueProgressionEvents = [];
      currentQueueParentId = null;
    }
  };

  const filteredTrace = maxSeqIdFilter !== null
    ? sortedTrace.filter(event => event.sequence_id <= maxSeqIdFilter)
    : sortedTrace;

  let firstExecutionEventAdded = false; // Flag to detect first execution event

  filteredTrace.forEach((event) => {
    const eventOrderId = getOrderId(event);

    if (
        (event.type !== 'QueueProgression' && currentQueueProgressionEvents.length > 0) ||
        (currentOrderId !== null && eventOrderId !== 'N/A' && eventOrderId !== currentOrderId) ||
        (event.type === 'QueueProgression' && currentQueueProgressionEvents.length > 0 &&
         event.parent_sequence_id !== currentQueueProgressionEvents[0].parent_sequence_id)
    ) {
      flushQueueProgression();
    }

    currentOrderId = eventOrderId;

    let isKeyEvent = false;
    let keyEventMarker = null;

    switch (event.type) {
      case 'OrderIntent':
        narrativeBlocks.push({
          group: 'Intent',
          narrative: `Strategy decision: Order ${getOrderId(event)} placed for ${event.quantity} at price ${event.price} (Side: ${event.side}).`,
          id: event.sequence_id,
          parentId: event.parent_sequence_id,
          timestamp: event.timestamp,
          isKeyEvent,
          keyEventMarker,
        });
        break;
      case 'OrderEnteredQueue':
        narrativeBlocks.push({
          group: 'Queue Entry',
          narrative: `Order ${getOrderId(event)} entered the queue at position ${event.queue_ahead}.`,
          id: event.sequence_id,
          parentId: event.parent_sequence_id,
          timestamp: event.timestamp,
          isKeyEvent,
          keyEventMarker,
        });
        currentQueueParentId = event.parent_sequence_id;
        break;
      case 'QueueProgression':
        if (currentQueueParentId === null || event.parent_sequence_id === currentQueueParentId) {
            currentQueueProgressionEvents.push(event);
            currentQueueParentId = event.parent_sequence_id;
        } else {
            flushQueueProgression();
            currentQueueProgressionEvents.push(event);
            currentQueueParentId = event.parent_sequence_id;
        }
        break;
      case 'PartialFill':
        isKeyEvent = true;
        keyEventMarker = '⭐ Partial Fill';
        if (!firstExecutionEventAdded) {
            keyEventMarker = '⭐ Execution begins (Partial Fill)';
            firstExecutionEventAdded = true;
        }
        narrativeBlocks.push({
          group: 'Execution',
          narrative: `Execution started: Order ${getOrderId(event)} partially filled ${event.filled_qty} at price ${event.price}.`,
          id: event.sequence_id,
          parentId: event.parent_sequence_id,
          timestamp: event.timestamp,
          isKeyEvent,
          keyEventMarker,
        });
        break;
      case 'OrderFilled':
        if (!firstExecutionEventAdded) {
            isKeyEvent = true;
            keyEventMarker = '⭐ Execution begins (Full Fill)';
            firstExecutionEventAdded = true;
        }
        narrativeBlocks.push({
          group: 'Execution',
          narrative: `Order ${getOrderId(event)} fully executed.`,
          id: event.sequence_id,
          parentId: event.parent_sequence_id,
          timestamp: event.timestamp,
          isKeyEvent,
          keyEventMarker,
        });
        break;
      case 'MarketEvent':
        break;
      default:
        narrativeBlocks.push({
          group: 'Other',
          narrative: `Unhandled event type: ${event.type} for order ${eventOrderId}.`,
          id: event.sequence_id,
          parentId: event.parent_sequence_id,
          timestamp: event.timestamp,
          isKeyEvent,
          keyEventMarker,
        });
        break;
    }
  });

  flushQueueProgression();

  return narrativeBlocks;
};

// Helper to compare narrative block arrays
const compareNarrativeBlocks = (narrativeBlocks1, narrativeBlocks2) => {
  const divergenceStatements = [];

  const maxLength = Math.max(narrativeBlocks1.length, narrativeBlocks2.length);

  for (let i = 0; i < maxLength; i++) {
    const block1 = narrativeBlocks1[i];
    const block2 = narrativeBlocks2[i];

    if (!block1 && block2) {
      divergenceStatements.push({
        type: 'missing_s1',
        message: `Strategy 1 ended earlier. Strategy 2 has block '${block2.group}' (Seq ${block2.id}) at step ${i + 1}.`,
        block: block2
      });
    } else if (block1 && !block2) {
      divergenceStatements.push({
        type: 'missing_s2',
        message: `Strategy 2 ended earlier. Strategy 1 has block '${block1.group}' (Seq ${block1.id}) at step ${i + 1}.`,
        block: block1
      });
    } else if (block1 && block2) {
      // Compare content for divergence
      if (block1.group !== block2.group) {
        divergenceStatements.push({
          type: 'group_type_divergence',
          message: `Event group divergence at step ${i + 1}: Strategy 1 has '${block1.group}' (Seq ${block1.id}) vs Strategy 2 has '${block2.group}' (Seq ${block2.id}).`,
          block1, block2
        });
      }
      if (block1.narrative !== block2.narrative) {
        divergenceStatements.push({
          type: 'narrative_content_divergence',
          message: `Narrative content divergence at step ${i + 1} for group '${block1.group}': Strategy 1: '${block1.narrative}' vs Strategy 2: '${block2.narrative}'.`,
          block1, block2
        });
      }
      if (block1.id !== block2.id) { // This implies timing divergence in sequence_id
        divergenceStatements.push({
          type: 'sequence_id_timing_divergence',
          message: `Sequence ID timing divergence at step ${i + 1}: Strategy 1 has Seq ${block1.id} vs Strategy 2 has Seq ${block2.id}.`,
          block1, block2
        });
      }
      if (block1.parentId !== block2.parentId) {
        divergenceStatements.push({
          type: 'causal_parent_divergence',
          message: `Causal parent divergence at step ${i + 1} for group '${block1.group}': Strategy 1 parent Seq ${block1.parentId || 'None'} vs Strategy 2 parent Seq ${block2.parentId || 'None'}.`,
          block1, block2
        });
      }
    }
  }

  return divergenceStatements;
};

// Helper to get execution summary
const getExecutionSummary = (narrativeBlocks) => {
  const totalSteps = narrativeBlocks.length;
  const partialFills = narrativeBlocks.filter(block => block.group === 'Execution' && block.narrative.includes('partially filled'))?.length || 0;
  const queueProgressions = narrativeBlocks.filter(block => block.group === 'Queue Progression')?.length || 0;
  const hasQueueProgression = queueProgressions > 0;
  const totalFills = narrativeBlocks.filter(block => block.group === 'Execution' && block.narrative.includes('fully executed'))?.length || 0;

  return { totalSteps, partialFills, queueProgressions, hasQueueProgression, totalFills };
};

// Define confidence levels
const CONFIDENCE_LEVELS = {
  HIGH: 'High Confidence',
  MEDIUM: 'Medium Confidence',
  LOW: 'Low Confidence',
};

const StrategyInspector = ({ strategyId: propStrategyId, seed: propSeed, strategyId2: propStrategyId2, seed2: propSeed2, onReset }) => {
  const [strategyId, setStrategyId] = useState(propStrategyId || '');
  const [seed, setSeed] = useState(propSeed || 42);
  const [inspectionResult, setInspectionResult] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const [strategyId2, setStrategyId2] = useState(propStrategyId2 || '');
  const [seed2, setSeed2] = useState(propSeed2 || 42);
  const [inspectionResult2, setInspectionResult2] = useState(null);
  const [loading2, setLoading2] = useState(false);
  const [error2, setError2] = useState(null);

  const [selectedSeqId, setSelectedSeqId] = useState(null);
  const [selectedMaxSeqId, setSelectedMaxSeqId] = useState(null);
  const [showRawEvents, setShowRawEvents] = useState(false);

  const rawEventRefs = useRef({});

  const handleInspectStrategy = useCallback(async (strategyNum) => {
    const isFirstStrategy = strategyNum === 1;
    const currentStrategyId = isFirstStrategy ? strategyId : strategyId2;
    const currentSeed = isFirstStrategy ? seed : seed2;

    if (!currentStrategyId || currentSeed === null) return;

    isFirstStrategy ? setLoading(true) : setLoading2(true);
    isFirstStrategy ? setError(null) : setError2(null);
    setSelectedSeqId(null);

    try {
      const response = await fetch('http://localhost:8000/inspect_strategy', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          strategy_id: currentStrategyId,
          seed: Number(currentSeed),
        }),
      });

      if (!response.ok) {
        const errText = await response.text();
        let msg = 'Failed to inspect strategy';
        try {
          const errorData = JSON.parse(errText);
          msg = errorData.message || errorData.error || msg;
        } catch {
          if (errText) msg = errText;
        }
        throw new Error(msg);
      }

      const rawJson = await response.json();
      const data = normalizeInspectResponse(rawJson);
      if (isFirstStrategy) {
        setInspectionResult(data);
        if (data.execution_trace && data.execution_trace.length > 0) {
          const maxId = Math.max(...data.execution_trace.map(e => e.sequence_id));
          setSelectedMaxSeqId(maxId);
        }
      } else {
        setInspectionResult2(data);
      }
    } catch (err) {
      isFirstStrategy ? setError(err.message) : setError2(err.message);
    } finally {
      isFirstStrategy ? setLoading(false) : setLoading2(false);
    }
  }, [strategyId, strategyId2, seed, seed2]);

  // Update local state when props change (for primary strategy)
  useEffect(() => {
    setStrategyId(propStrategyId || '');
    setSeed(propSeed || 42);
    if (propStrategyId && propStrategyId !== strategyId) {
        setStrategyId2('');
        setSeed2(42);
    }
  }, [propStrategyId, propSeed]);

  // Update local state when props change (for second strategy)
  useEffect(() => {
    setStrategyId2(propStrategyId2 || '');
    setSeed2(propSeed2 || 42);
  }, [propStrategyId2, propSeed2]);

  // Fetch data for primary strategy when its inputs change
  useEffect(() => {
    if (strategyId && seed !== null) {
      handleInspectStrategy(1);
    } else {
      setInspectionResult(null);
      setError(null);
    }
  }, [strategyId, seed, handleInspectStrategy]);

  // Fetch data for second strategy when its inputs change
  useEffect(() => {
    if (strategyId2 && seed2 !== null) {
      handleInspectStrategy(2);
    } else {
      setInspectionResult2(null);
      setError2(null);
    }
  }, [strategyId2, seed2, handleInspectStrategy]);

  // Sync raw event scroll on selection change
  useEffect(() => {
    if (selectedSeqId && showRawEvents && rawEventRefs.current[selectedSeqId]) {
      rawEventRefs.current[selectedSeqId].scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
      });
    }
  }, [selectedSeqId, showRawEvents]);

  const allNarrativeBlocks1 = useMemo(() => inspectionResult?.execution_trace ? groupAndNarrateEvents(inspectionResult.execution_trace, null) : [], [inspectionResult?.execution_trace]);
  const allNarrativeBlocks2 = useMemo(() => inspectionResult2?.execution_trace ? groupAndNarrateEvents(inspectionResult2.execution_trace, null) : [], [inspectionResult2?.execution_trace]);

  const eventMap = useMemo(() => {
    const map = {};
    [...allNarrativeBlocks1, ...allNarrativeBlocks2].forEach(block => {
      map[block.id] = block;
    });
    return map;
  }, [allNarrativeBlocks1, allNarrativeBlocks2]);

  const getCausalChain = (seqId) => {
    const chain = new Set();
    let current = seqId;

    while (current !== null && eventMap[current]) {
      chain.add(current);
      current = eventMap[current].parentId;
    }

    return chain;
  };

  const activeChain = selectedSeqId
    ? getCausalChain(selectedSeqId)
    : new Set();

  const narratedExecutionTrace1 = useMemo(() =>
    inspectionResult?.execution_trace ? groupAndNarrateEvents(inspectionResult.execution_trace, selectedMaxSeqId) : []
  , [inspectionResult?.execution_trace, selectedMaxSeqId]);

  const narratedExecutionTrace2 = useMemo(() =>
    inspectionResult2?.execution_trace ? groupAndNarrateEvents(inspectionResult2.execution_trace, selectedMaxSeqId) : []
  , [inspectionResult2?.execution_trace, selectedMaxSeqId]);

  const minSeqId = useMemo(() => inspectionResult?.execution_trace?.length > 0
    ? Math.min(...inspectionResult.execution_trace.map(e => e.sequence_id))
    : 0
  , [inspectionResult?.execution_trace]);

  const maxAvailableSeqId = useMemo(() => inspectionResult?.execution_trace?.length > 0
    ? Math.max(...inspectionResult.execution_trace.map(e => e.sequence_id))
    : 0
  , [inspectionResult?.execution_trace]);

  const getGroupColorClass = (group) => {
    switch (group) {
      case 'Intent': return 'border-green-500 text-green-800 bg-green-50';
      case 'Queue Entry': return 'border-yellow-500 text-yellow-800 bg-yellow-50';
      case 'Queue Progression': return 'border-orange-500 text-orange-800 bg-orange-50';
      case 'Execution': return 'border-blue-500 text-blue-800 bg-blue-50';
      default: return 'border-gray-500 text-gray-800 bg-gray-50';
    }
  };

  const isDualMode = (inspectionResult && inspectionResult2);

  const currentMaxSeqId = selectedMaxSeqId !== null ? selectedMaxSeqId : maxAvailableSeqId;
  const timelineBanner = `Replay Position: Seq ${currentMaxSeqId} / ${maxAvailableSeqId}`;

  const divergenceStatements = useMemo(() => isDualMode
    ? compareNarrativeBlocks(allNarrativeBlocks1, allNarrativeBlocks2)
    : [], [isDualMode, allNarrativeBlocks1, allNarrativeBlocks2]);

  const executionSummary1 = useMemo(() => getExecutionSummary(allNarrativeBlocks1), [allNarrativeBlocks1]);
  const executionSummary2 = useMemo(() => getExecutionSummary(allNarrativeBlocks2), [allNarrativeBlocks2]);

  let finalVerdict = 'No clear execution advantage — trade-offs observed.';
  let confidenceLevel = CONFIDENCE_LEVELS.LOW;
  let confidenceColorClass = 'text-red-700';
  let confidenceReason = '';

  const s1Completed = executionSummary1.totalFills > 0;
  const s2Completed = executionSummary2.totalFills > 0;

  const s1StrictlyBetter =
    executionSummary1.totalSteps <= executionSummary2.totalSteps &&
    executionSummary1.partialFills <= executionSummary2.partialFills &&
    (
      executionSummary1.totalSteps < executionSummary2.totalSteps ||
      executionSummary1.partialFills < executionSummary2.partialFills
    );

  const s2StrictlyBetter =
    executionSummary2.totalSteps <= executionSummary1.totalSteps &&
    executionSummary2.partialFills <= executionSummary1.partialFills &&
    (
      executionSummary2.totalSteps < executionSummary1.totalSteps ||
      executionSummary2.partialFills < executionSummary1.partialFills
    );

  // Check for HIGH_CONFIDENCE (Clear dominance)
  if (s1Completed && !s2Completed) {
    finalVerdict = 'Strategy 1 provides more complete execution overall.';
    confidenceLevel = CONFIDENCE_LEVELS.HIGH;
    confidenceColorClass = 'text-green-700';
    confidenceReason = 'Strategy 1 completed execution while Strategy 2 did not.';
  } else if (!s1Completed && s2Completed) {
    finalVerdict = 'Strategy 2 provides more complete execution overall.';
    confidenceLevel = CONFIDENCE_LEVELS.HIGH;
    confidenceColorClass = 'text-green-700';
    confidenceReason = 'Strategy 2 completed execution while Strategy 1 did not.';
  } else if (s1Completed && s2Completed) { // Both completed, compare speed then fragmentation
    if (s1StrictlyBetter) {
      finalVerdict = 'Strategy 1 provides more efficient execution overall (faster, less fragmented).';
      confidenceLevel = CONFIDENCE_LEVELS.HIGH;
      confidenceColorClass = 'text-green-700';
      confidenceReason = 'Strategy 1 strictly dominates Strategy 2 in execution speed and fragmentation.';
    } else if (s2StrictlyBetter) {
      finalVerdict = 'Strategy 2 provides more efficient execution overall (faster, less fragmented).';
      confidenceLevel = CONFIDENCE_LEVELS.HIGH;
      confidenceColorClass = 'text-green-700';
      confidenceReason = 'Strategy 2 strictly dominates Strategy 1 in execution speed and fragmentation.';
    }
  }

  // If not HIGH_CONFIDENCE, check for MEDIUM_CONFIDENCE (Trade-offs)
  if (confidenceLevel === CONFIDENCE_LEVELS.LOW && s1Completed === s2Completed) {
      // Case 1 — Speed vs Fragmentation Trade-off
      if (
        executionSummary1.totalSteps < executionSummary2.totalSteps &&
        executionSummary1.partialFills > executionSummary2.partialFills
      ) {
        confidenceReason = 'Strategy 1 is faster but more fragmented, while Strategy 2 is slower but cleaner.';
      }
      else if (
        executionSummary2.totalSteps < executionSummary1.totalSteps &&
        executionSummary2.partialFills > executionSummary1.partialFills
      ) {
        confidenceReason = 'Strategy 2 is faster but more fragmented, while Strategy 1 is slower but cleaner.';
      }
      // Case 2 — Queue Behavior Difference
      else if (
        executionSummary1.hasQueueProgression !== executionSummary2.hasQueueProgression
      ) {
        confidenceReason = 'Execution differs due to queue interaction — one strategy experienced queue delays while the other did not.';
      }
      // Case 3 — Same Speed, Different Fragmentation
      else if (
        executionSummary1.totalSteps === executionSummary2.totalSteps &&
        executionSummary1.partialFills !== executionSummary2.partialFills
      ) {
        confidenceReason = 'Both strategies executed in similar time, but differ in execution quality (fragmentation).';
      }
      // Case 4 — Fallback (Mixed Trade-offs)
      else {
        confidenceReason = 'Multiple trade-offs observed across execution speed, queue behavior, and fragmentation.';
      }

      finalVerdict = 'Trade-offs observed across execution characteristics.';
      confidenceLevel = CONFIDENCE_LEVELS.MEDIUM;
      confidenceColorClass = 'text-yellow-700';
  }
  // If still LOW_CONFIDENCE, it remains the default.
  if (confidenceLevel === CONFIDENCE_LEVELS.LOW) {
    confidenceReason = 'No clear dominance or significant trade-offs found in execution characteristics.';
  }
  
  return (
    <div className="p-4">
      <h2 className="text-2xl font-semibold mb-4">Strategy Inspector</h2>

      {isDualMode && (
        <h3 className="text-xl font-bold text-center mb-6 text-gray-800">
          Comparing Strategies: <span className="text-blue-600">{strategyId || 'N/A'}</span> vs <span className="text-blue-600">{strategyId2 || 'N/A'}</span>
        </h3>
      )}

      <div className="mb-4 grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="strategy_id">
            Strategy ID 1:
          </label>
          <input
            type="text"
            id="strategy_id"
            className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            value={strategyId}
            onChange={(e) => setStrategyId(e.target.value)}
            onBlur={() => handleInspectStrategy(1)}
          />
        </div>

        <div>
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="strategy_id_2">
            Strategy ID 2 (Optional):
          </label>
          <input
            type="text"
            id="strategy_id_2"
            className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            value={strategyId2}
            onChange={(e) => setStrategyId2(e.target.value)}
            onBlur={() => handleInspectStrategy(2)}
          />
        </div>
      </div>

      <div className="mb-6 grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="seed_inspect">
            Seed 1:
          </label>
          <input
            type="number"
            id="seed_inspect"
            className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            value={seed}
            onChange={(e) => setSeed(Number(e.target.value))}
            onBlur={() => handleInspectStrategy(1)}
          />
        </div>
        <div>
          <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="seed_inspect_2">
            Seed 2 (Optional):
          </label>
          <input
            type="number"
            id="seed_inspect_2"
            className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
            value={seed2}
            onChange={(e) => setSeed2(Number(e.target.value))}
            onBlur={() => handleInspectStrategy(2)}
          />
        </div>
      </div>

      <button
        className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline mb-4"
        onClick={() => { handleInspectStrategy(1); if (strategyId2) handleInspectStrategy(2); }}
        disabled={loading || loading2}
      >
        {(loading || loading2) ? 'Inspecting...' : (isDualMode ? 'Compare Executions' : 'Inspect Strategy')}
      </button>

      {error && <p className="text-red-500 mt-4">Error (Strategy 1): {error}</p>}
      {error2 && <p className="text-red-500 mt-2">Error (Strategy 2): {error2}</p>}

      {inspectionResult && (
        <>
          <div className="mt-6 space-y-6">

            <div className="border p-4 rounded-lg shadow-sm bg-gray-50">
              <h3 className="text-xl font-semibold mb-2">Execution Model</h3>
              <p className="text-sm text-gray-800">
                Decision → Queue → Market Interaction → Execution
              </p>
            </div>

            <div className="p-3 bg-blue-100 rounded-lg text-blue-800 font-medium text-center shadow-sm">
              {timelineBanner}
            </div>

            <div className="mb-4 flex items-center space-x-2">
              <label htmlFor="max-seq-id" className="text-sm text-gray-700">Replay Position (Seq ID):</label>
              <input
                type="range"
                id="max-seq-id"
                min={minSeqId}
                max={maxAvailableSeqId}
                value={selectedMaxSeqId !== null ? selectedMaxSeqId : maxAvailableSeqId}
                onChange={(e) => setSelectedMaxSeqId(Number(e.target.value))}
                className="w-full h-2 bg-blue-200 rounded-lg appearance-none cursor-pointer"
              />
              <span className="text-sm font-medium text-gray-800">
                {selectedMaxSeqId !== null ? selectedMaxSeqId : maxAvailableSeqId}
              </span>
            </div>
            <p className="text-sm text-gray-600 mb-4">
              Showing events from Seq {minSeqId} → Seq {currentMaxSeqId}
            </p>


            <div className="mb-4">
              <label className="inline-flex items-center">
                <input
                  type="checkbox"
                  className="form-checkbox text-blue-600"
                  checked={showRawEvents}
                  onChange={(e) => setShowRawEvents(e.target.checked)}
                />
                <span className="ml-2 text-gray-700">Show Raw Events (Debug Mode)</span>
              </label>
            </div>

            <div className={isDualMode ? "grid grid-cols-1 md:grid-cols-2 gap-6" : ""}>
              <div className={isDualMode ? "border-r border-gray-200 pr-3" : ""}>
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
              </div>

              {/* Strategy 2 Column */}
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
        </>
      )}
    </div>
  );
};

export default StrategyInspector;