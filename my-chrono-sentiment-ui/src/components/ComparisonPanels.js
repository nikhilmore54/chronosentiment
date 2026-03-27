import React from 'react';

const ComparisonPanels = ({
  isDualMode,
  allNarrativeBlocks1,
  allNarrativeBlocks2,
  strategyId,
  strategyId2,
  executionSummary1,
  executionSummary2,
  finalVerdict,
  confidenceLevel,
  confidenceColorClass,
  confidenceReason,
  divergenceStatements,
}) => {
  return (
    <>
      {isDualMode && (allNarrativeBlocks1?.length > 0 || allNarrativeBlocks2?.length > 0) && (
        <div className="mt-6 border p-4 rounded-lg shadow-sm bg-gray-50">
          <h3 className="text-xl font-semibold mb-2">Execution Summary Comparison</h3>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p className="font-bold">Strategy 1 ({strategyId || 'N/A'}):</p>
              <ul className="list-disc list-inside ml-4">
                <li>Steps: {executionSummary1.totalSteps}</li>
                <li>Partial Fills: {executionSummary1.partialFills}</li>
                <li>Queue Progression: {executionSummary1.hasQueueProgression ? 'Yes' : 'No'}</li>
                <li>Full Fills: {executionSummary1.totalFills}</li>
              </ul>
            </div>
            <div>
              <p className="font-bold">Strategy 2 ({strategyId2 || 'N/A'}):</p>
              <ul className="list-disc list-inside ml-4">
                <li>Steps: {executionSummary2.totalSteps}</li>
                <li>Partial Fills: {executionSummary2.partialFills}</li>
                <li>Queue Progression: {executionSummary2.hasQueueProgression ? 'Yes' : 'No'}</li>
                <li>Full Fills: {executionSummary2.totalFills}</li>
              </ul>
            </div>
          </div>
        </div>
      )}

      {isDualMode && (allNarrativeBlocks1?.length > 0 || allNarrativeBlocks2?.length > 0) && (
        <div className="mt-6 border p-4 rounded-lg shadow-sm bg-gray-50">
          <h3 className="text-xl font-semibold mb-2">Execution Insights</h3>
          <ul className="list-disc list-inside text-sm text-gray-800">
            {/* Dynamically generated insights */}
            {executionSummary1.totalSteps < executionSummary2.totalSteps && (
              <li>Strategy 1 executed faster (fewer steps).</li>
            )}
            {executionSummary2.totalSteps < executionSummary1.totalSteps && (
              <li>Strategy 2 executed faster (fewer steps).</li>
            )}
            {executionSummary1.totalSteps > 0 && executionSummary1.totalSteps === executionSummary2.totalSteps && (
              <li>Both strategies executed in the same number of steps.</li>
            )}

            {executionSummary1.hasQueueProgression && !executionSummary2.hasQueueProgression && (
              <li>Strategy 1 experienced queue delay while Strategy 2 did not.</li>
            )}
            {!executionSummary1.hasQueueProgression && executionSummary2.hasQueueProgression && (
              <li>Strategy 2 experienced queue delay while Strategy 1 did not.</li>
            )}
            {executionSummary1.hasQueueProgression && executionSummary2.hasQueueProgression && (
              <li>Both strategies experienced queue delays.</li>
            )}
            {!executionSummary1.hasQueueProgression && !executionSummary2.hasQueueProgression && (
              <li>Neither strategy experienced significant queue delays.</li>
            )}

            {executionSummary1.partialFills > executionSummary2.partialFills && (
              <li>Strategy 1 experienced more fragmented execution (more partial fills).</li>
            )}
            {executionSummary2.partialFills > executionSummary1.partialFills && (
              <li>Strategy 2 experienced more fragmented execution (more partial fills).</li>
            )}
            {executionSummary1.partialFills === 0 && executionSummary2.partialFills === 0 && (
              <li>Neither strategy experienced partial fills.</li>
            )}
            {executionSummary1.partialFills > 0 && executionSummary1.partialFills === executionSummary2.partialFills && (
              <li>Both strategies experienced the same number of partial fills.</li>
            )}
          </ul>
        </div>
      )}

      {isDualMode && (allNarrativeBlocks1?.length > 0 || allNarrativeBlocks2?.length > 0) && (
        <div className={`mt-6 border p-4 rounded-lg shadow-sm ${confidenceColorClass === 'text-green-700' ? 'bg-green-50' : confidenceColorClass === 'text-yellow-700' ? 'bg-yellow-50' : 'bg-red-50'}`}>
          <h3 className="text-xl font-semibold mb-2">Final Execution Verdict</h3>
          <p className={`text-sm font-medium ${confidenceColorClass}`}>{finalVerdict}</p>
          <p className={`text-sm font-bold mt-2 ${confidenceColorClass}`}>Confidence Level: {confidenceLevel}</p>
          <p className="text-xs text-gray-600 mt-1">
            Reason: {confidenceReason}
          </p>
        </div>
      )}

      {isDualMode && (allNarrativeBlocks1?.length > 0 || allNarrativeBlocks2?.length > 0) && (
        <div className="mt-6 border p-4 rounded-lg shadow-sm bg-red-50">
          <h3 className="text-xl font-semibold mb-2 text-red-800">Execution Divergence Analysis Summary</h3>
          <ul className="list-disc list-inside text-sm text-red-700">
            {divergenceStatements.length > 0 ? (
              divergenceStatements.map((d, index) => <li key={index}>{d.message}</li>)
            ) : (
              <li>No significant execution divergences detected in visible events.</li>
            )}
          </ul>
        </div>
      )}
    </>
  );
};

export default ComparisonPanels;