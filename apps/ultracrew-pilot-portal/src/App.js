import React, { useState, useEffect, useCallback } from 'react';
import { S } from './styles';
import Header from './components/Header';
import BenchmarkStudio from './components/BenchmarkStudio';
import SolverControlCenter from './components/SolverControlCenter';
import TwoTierAuditMatrix from './components/TwoTierAuditMatrix';
import GanttScheduleViewer from './components/GanttScheduleViewer';
import IropsConsole from './components/IropsConsole';
import EvidenceSession from './components/EvidenceModal';
import ConvergenceChart from './components/ConvergenceChart';
import {
  fetchBenchmarkCatalog,
  runCoralysOptimization,
  runIropsRecovery,
  submitPilotSession,
  FALLBACK_BENCHMARKS,
} from './services/ultracrewApi';

export default function App() {
  const [activeTab, setActiveTab] = useState('benchmark');
  const [catalog, setCatalog] = useState(FALLBACK_BENCHMARKS);
  const [selectedInstanceId, setSelectedInstanceId] = useState('instance1');
  
  // Solver Configuration
  const [mode, setMode] = useState('equity'); // equity | preference | deadhead
  const [generations, setGenerations] = useState(300);
  const [popSize, setPopSize] = useState(50);
  const [seed, setSeed] = useState(42);
  const [creditRigs, setCreditRigs] = useState(true);
  const [crewAvailability, setCrewAvailability] = useState(true);
  const [reserveRatio, setReserveRatio] = useState(0.08);

  // Execution State
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState(null);
  const [engineStatus, setEngineStatus] = useState('online');

  // IROPS Disruption State
  const [recovering, setRecovering] = useState(false);
  const [iropsResult, setIropsResult] = useState(null);

  // Evidence Session State
  const [sessionSubmitted, setSessionSubmitted] = useState(false);

  // Fetch benchmark catalog on load
  useEffect(() => {
    fetchBenchmarkCatalog()
      .then((data) => {
        if (data && data.length > 0) setCatalog(data);
      })
      .catch((e) => {
        console.warn('Using fallback catalog:', e);
      });
  }, []);

  const selectedInstance = catalog.find((i) => i.id === selectedInstanceId) || catalog[0];

  // Run Solver Handler
  const handleRunSolver = useCallback(async () => {
    setRunning(true);
    setIropsResult(null);
    try {
      const resp = await runCoralysOptimization({
        instance_id: selectedInstanceId,
        mode,
        generations,
        pop_size: popSize,
        seed,
        credit_rigs: creditRigs,
        crew_availability: crewAvailability,
        reserve_ratio: reserveRatio,
      });
      setResult(resp);
      setEngineStatus('online');
    } catch (e) {
      console.error('Optimization failed:', e);
      setEngineStatus('offline');
    } finally {
      setRunning(false);
    }
  }, [selectedInstanceId, mode, generations, popSize, seed, creditRigs, crewAvailability, reserveRatio]);

  // Run on initial instance change if no result yet
  useEffect(() => {
    handleRunSolver();
  }, [selectedInstanceId, mode, handleRunSolver]);

  // Run IROPS Handler
  const handleRunIropsRecovery = async (payload) => {
    setRecovering(true);
    try {
      const resp = await runIropsRecovery(payload);
      setIropsResult(resp);
    } catch (e) {
      console.error('IROPS Recovery failed:', e);
    } finally {
      setRecovering(false);
    }
  };

  // Submit Evidence Handler
  const handleSubmitSession = async (payload) => {
    await submitPilotSession({
      ...payload,
      instance_id: selectedInstanceId,
      result_summary: result?.audit_matrix,
    });
    setSessionSubmitted(true);
  };

  return (
    <div style={S.app}>
      <Header
        activeTab={activeTab}
        onSelectTab={setActiveTab}
        status={engineStatus}
      />

      <main style={S.container}>
        {/* TAB 1: BENCHMARK STUDIO */}
        {activeTab === 'benchmark' && (
          <BenchmarkStudio
            catalog={catalog}
            selectedInstanceId={selectedInstanceId}
            onSelectInstance={(id) => {
              setSelectedInstanceId(id);
            }}
            onLaunchSolver={() => {
              setActiveTab('solver');
            }}
          />
        )}

        {/* TAB 2: SOLVER & CONTROLS */}
        {activeTab === 'solver' && (
          <div>
            <SolverControlCenter
              selectedInstance={selectedInstance}
              mode={mode}
              setMode={setMode}
              generations={generations}
              setGenerations={setGenerations}
              popSize={popSize}
              setPopSize={setPopSize}
              seed={seed}
              setSeed={setSeed}
              creditRigs={creditRigs}
              setCreditRigs={setCreditRigs}
              crewAvailability={crewAvailability}
              setCrewAvailability={setCrewAvailability}
              reserveRatio={reserveRatio}
              setReserveRatio={setReserveRatio}
              running={running}
              onRunSolver={handleRunSolver}
              result={result}
            />

            {result && (
              <ConvergenceChart history={result.convergence_history} />
            )}
          </div>
        )}

        {/* TAB 3: TWO-TIER AUDIT */}
        {activeTab === 'audit' && (
          <TwoTierAuditMatrix
            auditMatrix={iropsResult?.delta_audit_matrix || result?.audit_matrix}
            result={result}
          />
        )}

        {/* TAB 4: ROSTER GANTT & PAIRINGS */}
        {activeTab === 'gantt' && (
          <GanttScheduleViewer
            rotations={iropsResult?.repaired_roster_rotations?.length ? iropsResult.repaired_roster_rotations : result?.rotations || []}
          />
        )}

        {/* TAB 5: OCC IROPS DISRUPTION */}
        {activeTab === 'irops' && (
          <IropsConsole
            rotations={result?.rotations || []}
            onRunIropsRecovery={handleRunIropsRecovery}
            iropsResult={iropsResult}
            recovering={recovering}
          />
        )}

        {/* TAB 6: COMMERCIAL SESSION */}
        {activeTab === 'evidence' && (
          <EvidenceSession
            result={result}
            selectedInstance={selectedInstance}
            onSubmitSession={handleSubmitSession}
            sessionSubmitted={sessionSubmitted}
          />
        )}
      </main>
    </div>
  );
}
