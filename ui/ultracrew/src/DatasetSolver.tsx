import React, { useState } from 'react';
import { API_BASE_URL } from './config/api';

interface Worker {
  id: number;
  skills: string[];
}

interface Shift {
  id: number;
  start_hour: number;
  duration_hours: number;
  required_skill: string;
}

interface ScheduleRequest {
  workers: Worker[];
  shifts: Shift[];
  historical_workloads?: Record<number, number[]>;
  rng_seed?: number;
}

interface ConstraintReport {
  fitness: number;
  is_valid: boolean;
  hard_violations: number;
  soft_violations: number;
  violated_constraints: string[];
  satisfied_constraints: string[];
  constraint_scores: Record<string, number>;
  warnings: string[];
}

interface Recommendation {
  constraint_id: string;
  severity: string;
  explanation: string;
  recommended_action: string;
}

interface TelemetryGeneration {
  generation: number;
  best_fitness: number;
  average_fitness: number;
  hard_violations: number;
  soft_violations: number;
  elapsed_time_ms: number;
}

interface TelemetryReport {
  generations: TelemetryGeneration[];
}

interface ScheduleResponse {
  schedule: Record<string, number>;
  metrics: Record<string, number>;
  constraint_report: ConstraintReport;
  recommendations: Recommendation[];
  telemetry: TelemetryReport | null;
}

export const DatasetSolver = () => {
  const [fileContent, setFileContent] = useState<ScheduleRequest | null>(null);
  const [fileName, setFileName] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>('');
  const [solution, setSolution] = useState<ScheduleResponse | null>(null);
  const [localAssignments, setLocalAssignments] = useState<Record<string, number>>({});
  const [lockedShiftIds, setLockedShiftIds] = useState<Set<number>>(new Set());

  // Handle file import
  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    setError('');
    const file = e.target.files?.[0];
    if (!file) return;

    setFileName(file.name);
    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const json = JSON.parse(event.target?.result as string);
        if (!json.workers || !json.shifts) {
          throw new Error('Invalid JSON structure: must contain workers and shifts fields.');
        }
        setFileContent(json);
        setSolution(null);
        setLocalAssignments({});
        setLockedShiftIds(new Set());
      } catch (err: any) {
        setError(err.message || 'Failed to parse JSON file.');
      }
    };
    reader.readAsText(file);
  };

  // Submit to POST /api/schedule
  const handleSolve = async () => {
    if (!fileContent) return;
    setLoading(true);
    setError('');
    try {
      const res = await fetch(`/api/schedule`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(fileContent),
      });

      if (!res.ok) {
        throw new Error(`Server returned status ${res.status}: ${await res.text()}`);
      }

      const data: ScheduleResponse = await res.json();
      setSolution(data);
      setLocalAssignments(data.schedule);
    } catch (err: any) {
      setError(err.message || 'Error occurred while solving schedule.');
    } finally {
      setLoading(false);
    }
  };

  // Submit to POST /api/validate
  const handleValidate = async () => {
    if (!fileContent || !solution) return;
    setLoading(true);
    setError('');
    try {
      const validatePayload = {
        request: fileContent,
        assignments: localAssignments,
      };

      const res = await fetch(`/api/validate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(validatePayload),
      });

      if (!res.ok) {
        throw new Error(`Server returned status ${res.status}: ${await res.text()}`);
      }

      const data: ScheduleResponse = await res.json();
      setSolution(data);
    } catch (err: any) {
      setError(err.message || 'Error occurred while validating schedule.');
    } finally {
      setLoading(false);
    }
  };

  // Submit to POST /api/reschedule
  const handleReschedule = async () => {
    if (!fileContent || !solution) return;
    setLoading(true);
    setError('');
    try {
      const reschedulePayload = {
        request: fileContent,
        existing_assignments: localAssignments,
        locked_shift_ids: Array.from(lockedShiftIds),
      };

      const res = await fetch(`/api/reschedule`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(reschedulePayload),
      });

      if (!res.ok) {
        throw new Error(`Server returned status ${res.status}: ${await res.text()}`);
      }

      const data: ScheduleResponse = await res.json();
      setSolution(data);
      setLocalAssignments(data.schedule);
    } catch (err: any) {
      setError(err.message || 'Error occurred during reschedule.');
    } finally {
      setLoading(false);
    }
  };

  // Toggle shift lock status
  const toggleShiftLock = (shiftId: number) => {
    const nextLocked = new Set(lockedShiftIds);
    if (nextLocked.has(shiftId)) {
      nextLocked.delete(shiftId);
    } else {
      nextLocked.add(shiftId);
    }
    setLockedShiftIds(nextLocked);
  };

  // Handle local assignment dropdown changes
  const handleAssignmentChange = (shiftId: number, workerId: number) => {
    setLocalAssignments(prev => ({
      ...prev,
      [shiftId.toString()]: workerId
    }));
  };

  // SVG Convergence Chart helper
  const renderConvergenceChart = (generations: TelemetryGeneration[]) => {
    if (!generations || generations.length === 0) return null;
    const width = 600;
    const height = 150;
    const padding = 20;

    const fitnessVals = generations.map(g => g.best_fitness);
    const maxFit = Math.max(...fitnessVals);
    const minFit = Math.min(...fitnessVals);
    const fitRange = maxFit - minFit || 1.0;

    const points = generations.map((g, idx) => {
      const x = padding + (idx / (generations.length - 1)) * (width - 2 * padding);
      const y = height - padding - ((g.best_fitness - minFit) / fitRange) * (height - 2 * padding);
      return `${x},${y}`;
    }).join(' ');

    return (
      <svg width="100%" height={height} viewBox={`0 0 ${width} ${height}`} style={{ background: '#1e293b', borderRadius: '8px', padding: '10px' }}>
        {/* Draw axes */}
        <line x1={padding} y1={height - padding} x2={width - padding} y2={height - padding} stroke="#475569" strokeWidth="1" />
        <line x1={padding} y1={padding} x2={padding} y2={height - padding} stroke="#475569" strokeWidth="1" />
        
        {/* Convergence Path */}
        <polyline fill="none" stroke="var(--accent-color)" strokeWidth="2.5" points={points} />
        
        {/* Draw bounds labels */}
        <text x={padding + 5} y={padding + 12} fill="#94a3b8" fontSize="10">{maxFit.toFixed(0)}</text>
        <text x={padding + 5} y={height - padding - 5} fill="#94a3b8" fontSize="10">{minFit.toFixed(0)}</text>
        <text x={width - padding - 60} y={height - padding - 5} fill="#94a3b8" fontSize="10">Gen {generations.length}</text>
      </svg>
    );
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div className="card" style={{ background: 'rgba(30, 41, 59, 0.7)', backdropFilter: 'blur(10px)' }}>
        <h2>Dataset Solver</h2>
        <p style={{ color: 'var(--text-muted)', marginBottom: '1.5rem' }}>
          Upload an INRC scheduling dataset (JSON format containing workers and shifts) to resolve assignments, perform static validation, or run partial re-optimization.
        </p>

        {/* Upload Form */}
        <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', flexWrap: 'wrap' }}>
          <label style={{
            backgroundColor: 'var(--primary-color)',
            color: 'white',
            padding: '0.75rem 1.5rem',
            borderRadius: '6px',
            cursor: 'pointer',
            fontWeight: 500,
            display: 'inline-block'
          }}>
            Browse JSON Dataset
            <input type="file" accept=".json" onChange={handleFileUpload} style={{ display: 'none' }} />
          </label>

          {fileName && <span style={{ color: 'var(--accent-color)', fontWeight: 500 }}>📁 {fileName}</span>}
          {fileContent && (
            <span style={{ color: 'var(--text-muted)', fontSize: '0.9rem' }}>
              ({fileContent.workers.length} workers, {fileContent.shifts.length} shifts loaded)
            </span>
          )}
        </div>

        {error && <div style={{ color: 'var(--danger-color)', marginTop: '1rem', fontWeight: 500 }}>⚠️ {error}</div>}

        {/* Solver Controls */}
        {fileContent && (
          <div style={{ display: 'flex', gap: '1rem', marginTop: '1.5rem', flexWrap: 'wrap' }}>
            <button
              onClick={handleSolve}
              disabled={loading}
              style={{
                backgroundColor: 'var(--primary-color)',
                color: 'white',
                border: 'none',
                padding: '0.75rem 1.5rem',
                borderRadius: '6px',
                fontWeight: 600,
                cursor: 'pointer',
                opacity: loading ? 0.6 : 1
              }}
            >
              Solve Schedule
            </button>

            <button
              onClick={handleReschedule}
              disabled={loading || !solution}
              style={{
                backgroundColor: '#f59e0b',
                color: '#1e1b4b',
                border: 'none',
                padding: '0.75rem 1.5rem',
                borderRadius: '6px',
                fontWeight: 600,
                cursor: 'pointer',
                opacity: (loading || !solution) ? 0.5 : 1
              }}
            >
              Reschedule ({lockedShiftIds.size} locked)
            </button>

            <button
              onClick={handleValidate}
              disabled={loading || !solution}
              style={{
                backgroundColor: 'var(--success-color)',
                color: 'white',
                border: 'none',
                padding: '0.75rem 1.5rem',
                borderRadius: '6px',
                fontWeight: 600,
                cursor: 'pointer',
                opacity: (loading || !solution) ? 0.5 : 1
              }}
            >
              Validate Changes
            </button>
          </div>
        )}
      </div>

      {loading && (
        <div className="card" style={{ textAlign: 'center', padding: '3rem' }}>
          <div style={{
            border: '4px solid rgba(56, 189, 248, 0.1)',
            borderLeftColor: 'var(--accent-color)',
            borderRadius: '50%',
            width: '40px',
            height: '40px',
            animation: 'spin 1s linear infinite',
            margin: '0 auto 1.5rem'
          }} />
          <p style={{ color: 'var(--accent-color)', fontWeight: 600 }}>Running Optimization Engine...</p>
        </div>
      )}

      {solution && !loading && (
        <>
          {/* Metrics Panel */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '1rem' }}>
            <div className="card" style={{ margin: 0, textAlign: 'center' }}>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Fitness Score</div>
              <div style={{
                fontSize: '1.75rem',
                fontWeight: 700,
                marginTop: '0.5rem',
                color: solution.metrics.fitness >= 9000 ? 'var(--success-color)' : 'var(--danger-color)'
              }}>{solution.metrics.fitness?.toFixed(1) || '0.0'}</div>
            </div>

            <div className="card" style={{ margin: 0, textAlign: 'center' }}>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Hard Violations</div>
              <div style={{
                fontSize: '1.75rem',
                fontWeight: 700,
                marginTop: '0.5rem',
                color: solution.metrics.hard_violations === 0 ? 'var(--success-color)' : 'var(--danger-color)'
              }}>{solution.metrics.hard_violations}</div>
            </div>

            <div className="card" style={{ margin: 0, textAlign: 'center' }}>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Soft Violations</div>
              <div style={{
                fontSize: '1.75rem',
                fontWeight: 700,
                marginTop: '0.5rem',
                color: '#38bdf8'
              }}>{solution.constraint_report.soft_violations}</div>
            </div>

            <div className="card" style={{ margin: 0, textAlign: 'center' }}>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Fairness Penalty</div>
              <div style={{
                fontSize: '1.75rem',
                fontWeight: 700,
                marginTop: '0.5rem',
                color: '#fb7185'
              }}>{solution.metrics.fairness_penalty?.toFixed(1) || '0.0'}</div>
            </div>

            <div className="card" style={{ margin: 0, textAlign: 'center' }}>
              <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Workload Penalty</div>
              <div style={{
                fontSize: '1.75rem',
                fontWeight: 700,
                marginTop: '0.5rem',
                color: '#fb7185'
              }}>{solution.metrics.fatigue_penalty?.toFixed(1) || '0.0'}</div>
            </div>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))', gap: '1.5rem' }}>
            {/* Constraint Panel */}
            <div className="card" style={{ margin: 0 }}>
              <h2>Constraints Status</h2>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <div>
                  <h4 style={{ margin: '0 0 0.5rem 0', color: 'var(--danger-color)' }}>Violated Constraints</h4>
                  {solution.constraint_report.violated_constraints.length === 0 ? (
                    <p style={{ color: 'var(--success-color)', fontSize: '0.9rem' }}>None</p>
                  ) : (
                    <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                      {solution.constraint_report.violated_constraints.map(c => (
                        <span key={c} style={{
                          backgroundColor: 'rgba(239, 68, 68, 0.1)',
                          border: '1px solid var(--danger-color)',
                          color: 'var(--danger-color)',
                          padding: '0.25rem 0.5rem',
                          borderRadius: '4px',
                          fontSize: '0.8rem',
                          fontWeight: 600
                        }}>{c}</span>
                      ))}
                    </div>
                  )}
                </div>

                <div>
                  <h4 style={{ margin: '0 0 0.5rem 0', color: 'var(--success-color)' }}>Satisfied Constraints</h4>
                  <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                    {solution.constraint_report.satisfied_constraints.map(c => (
                      <span key={c} style={{
                        backgroundColor: 'rgba(34, 197, 94, 0.1)',
                        border: '1px solid var(--success-color)',
                        color: 'var(--success-color)',
                        padding: '0.25rem 0.5rem',
                        borderRadius: '4px',
                        fontSize: '0.8rem',
                        fontWeight: 600
                      }}>{c}</span>
                    ))}
                  </div>
                </div>

                {solution.constraint_report.warnings.length > 0 && (
                  <div>
                    <h4 style={{ margin: '0 0 0.5rem 0', color: '#f59e0b' }}>Warnings</h4>
                    <ul style={{ margin: 0, paddingLeft: '1.25rem', color: '#f59e0b', fontSize: '0.85rem' }}>
                      {solution.constraint_report.warnings.map((w, idx) => (
                        <li key={idx} style={{ marginBottom: '0.25rem' }}>{w}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </div>

            {/* Recommendations Panel */}
            <div className="card" style={{ margin: 0 }}>
              <h2>Recommendations</h2>
              {solution.recommendations.length === 0 ? (
                <div style={{ color: 'var(--success-color)', fontWeight: 500 }}>
                  🎉 Roster meets all hard and soft requirements perfectly!
                </div>
              ) : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', maxHeight: '250px', overflowY: 'auto' }}>
                  {solution.recommendations.map((rec, idx) => (
                    <div key={idx} style={{
                      borderLeft: `3px solid ${rec.severity === 'Hard' ? 'var(--danger-color)' : '#f59e0b'}`,
                      backgroundColor: 'rgba(30, 41, 59, 0.5)',
                      padding: '0.75rem',
                      borderRadius: '4px'
                    }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.25rem' }}>
                        <span style={{ fontWeight: 600, fontSize: '0.9rem' }}>{rec.constraint_id}</span>
                        <span style={{
                          fontSize: '0.75rem',
                          fontWeight: 700,
                          color: rec.severity === 'Hard' ? 'var(--danger-color)' : '#f59e0b'
                        }}>{rec.severity.toUpperCase()}</span>
                      </div>
                      <div style={{ fontSize: '0.85rem', color: 'var(--text-main)', marginBottom: '0.5rem' }}>
                        {rec.explanation}
                      </div>
                      <div style={{ fontSize: '0.8rem', color: 'var(--accent-color)', fontStyle: 'italic' }}>
                        Action: {rec.recommended_action}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Telemetry Observer Panel */}
          {solution.telemetry && (
            <div className="card">
              <h2>Optimizer Observatory (Convergence Profile)</h2>
              <div style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
                <div style={{ flex: '1', minWidth: '300px' }}>
                  {renderConvergenceChart(solution.telemetry.generations)}
                </div>
                <div style={{ width: '200px', display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
                  <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Initial Fitness:</div>
                  <div style={{ fontWeight: 600, color: 'var(--danger-color)', marginBottom: '0.75rem' }}>
                    {solution.telemetry.generations[0]?.best_fitness.toFixed(1)}
                  </div>
                  <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Converged Fitness:</div>
                  <div style={{ fontWeight: 600, color: 'var(--success-color)' }}>
                    {solution.telemetry.generations[solution.telemetry.generations.length - 1]?.best_fitness.toFixed(1)}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Schedule Assignments Grid / Table */}
          <div className="card">
            <h2>Schedule Assignments</h2>
            <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', color: 'var(--text-main)', fontSize: '0.9rem' }}>
                <thead>
                  <tr style={{ borderBottom: '2px solid var(--border-color)', textAlign: 'left' }}>
                    <th style={{ padding: '0.75rem' }}>Lock</th>
                    <th style={{ padding: '0.75rem' }}>Shift ID</th>
                    <th style={{ padding: '0.75rem' }}>Start Time (Hour)</th>
                    <th style={{ padding: '0.75rem' }}>Duration</th>
                    <th style={{ padding: '0.75rem' }}>Required Skill</th>
                    <th style={{ padding: '0.75rem' }}>Assigned Worker</th>
                  </tr>
                </thead>
                <tbody>
                  {fileContent?.shifts.map(shift => {
                    const assignedWorkerId = localAssignments[shift.id.toString()];
                    const isLocked = lockedShiftIds.has(shift.id);
                    return (
                      <tr key={shift.id} style={{
                        borderBottom: '1px solid var(--border-color)',
                        backgroundColor: isLocked ? 'rgba(245, 158, 11, 0.05)' : 'transparent'
                      }}>
                        <td style={{ padding: '0.75rem' }}>
                          <input
                            type="checkbox"
                            checked={isLocked}
                            onChange={() => toggleShiftLock(shift.id)}
                            style={{ cursor: 'pointer' }}
                          />
                        </td>
                        <td style={{ padding: '0.75rem', fontWeight: 600 }}>{shift.id}</td>
                        <td style={{ padding: '0.75rem' }}>Hour {shift.start_hour}</td>
                        <td style={{ padding: '0.75rem' }}>{shift.duration_hours} hrs</td>
                        <td style={{ padding: '0.75rem' }}>
                          <span style={{
                            backgroundColor: 'rgba(56, 189, 248, 0.1)',
                            color: 'var(--accent-color)',
                            padding: '0.2rem 0.4rem',
                            borderRadius: '4px',
                            fontSize: '0.8rem',
                            fontWeight: 500
                          }}>{shift.required_skill}</span>
                        </td>
                        <td style={{ padding: '0.75rem' }}>
                          <select
                            value={assignedWorkerId || ''}
                            onChange={(e) => handleAssignmentChange(shift.id, parseInt(e.target.value))}
                            style={{
                              backgroundColor: '#1e293b',
                              color: 'var(--text-main)',
                              border: '1px solid var(--border-color)',
                              padding: '0.25rem 0.5rem',
                              borderRadius: '4px',
                              cursor: 'pointer'
                            }}
                          >
                            <option value="">Unassigned</option>
                            {fileContent?.workers.map(w => (
                              <option key={w.id} value={w.id}>
                                Worker {w.id} ({w.skills.join(', ')})
                              </option>
                            ))}
                          </select>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}
    </div>
  );
};
