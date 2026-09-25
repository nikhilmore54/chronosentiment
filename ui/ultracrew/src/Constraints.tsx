import { useEffect, useState } from 'react';


interface Contract {
  id: string;
  minimumNumberOfAssignments: number;
  maximumNumberOfAssignments: number;
  minimumNumberOfConsecutiveWorkingDays: number;
  maximumNumberOfConsecutiveWorkingDays: number;
  minimumNumberOfConsecutiveDaysOff: number;
  maximumNumberOfConsecutiveDaysOff: number;
  maximumNumberOfWorkingWeekends: number;
  completeWeekends: number;
}

interface ForbiddenSuccession {
  precedingShiftType: string;
  succeedingShiftTypes: string[];
}

interface ScenarioData {
  contracts: Contract[];
  forbiddenShiftTypeSuccessions: ForbiddenSuccession[];
}

export const Constraints = () => {
  const [data, setData] = useState<ScenarioData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch(`/api/scenario`)
      .then(res => {
        if (!res.ok) throw new Error(`Server returned ${res.status}`);
        return res.json();
      })
      .then(d => {
        setData(d);
        setLoading(false);
      })
      .catch(err => {
        console.error('Failed to load constraints:', err);
        setError(err.message ?? 'Unknown error');
        setLoading(false);
      });
  }, []);

  if (loading) {
    return <p style={{ color: 'var(--text-muted)', padding: '1.5rem' }}>Loading Constraints...</p>;
  }

  if (error || !data) {
    return (
      <div style={{
        margin: '1.5rem',
        padding: '1.25rem 1.5rem',
        backgroundColor: 'rgba(239, 68, 68, 0.08)',
        border: '1px solid rgba(239, 68, 68, 0.3)',
        borderRadius: '8px',
        color: 'var(--text-main)',
      }}>
        <div style={{ fontWeight: 700, marginBottom: '0.5rem', color: '#ef4444' }}>⚠ Could not load constraints</div>
        <div style={{ fontSize: '0.9rem', color: 'var(--text-muted)', marginBottom: '0.75rem' }}>
          {error ?? 'No data returned from server.'}
        </div>
        <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>
          Check that the UltraCrew server is running on port 3001.
          Start it with: <code style={{ color: 'var(--accent-color)' }}>cargo run -p ultracrew_server</code>
        </div>
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div className="card">
        <h2>Operational Constraints</h2>
        <p style={{ color: 'var(--text-muted)', marginBottom: '1.5rem' }}>
          The following rules are actively enforced by the UltraCrew engine when generating the schedule.
        </p>

        <h3 style={{ color: 'var(--accent-color)', fontSize: '1.1rem', marginBottom: '1rem', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
          Contract Types
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
          {data.contracts.map((c) => (
            <div key={c.id} style={{ backgroundColor: 'var(--bg-color)', padding: '1.5rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
              <h4 style={{ margin: '0 0 1rem 0', fontSize: '1.1rem' }}>{c.id} Contract</h4>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '1rem' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Assignments per Period</span>
                  <span style={{ fontWeight: 600 }}>{c.minimumNumberOfAssignments} - {c.maximumNumberOfAssignments}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Consecutive Working Days</span>
                  <span style={{ fontWeight: 600 }}>{c.minimumNumberOfConsecutiveWorkingDays} - {c.maximumNumberOfConsecutiveWorkingDays}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Consecutive Days Off</span>
                  <span style={{ fontWeight: 600 }}>{c.minimumNumberOfConsecutiveDaysOff} - {c.maximumNumberOfConsecutiveDaysOff}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Max Working Weekends</span>
                  <span style={{ fontWeight: 600 }}>{c.maximumNumberOfWorkingWeekends}</span>
                </div>
                <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Complete Weekends Required</span>
                  <span style={{ fontWeight: 600, color: c.completeWeekends === 1 ? 'var(--success-color)' : 'inherit' }}>
                    {c.completeWeekends === 1 ? 'Yes' : 'No'}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="card">
        <h3 style={{ color: 'var(--danger-color)', fontSize: '1.1rem', marginBottom: '1rem', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
          Forbidden Shift Successions (Rest Violations)
        </h3>
        <p style={{ color: 'var(--text-muted)', marginBottom: '1.5rem', fontSize: '0.9rem' }}>
          Nurses working a specific shift cannot immediately work the forbidden succeeding shifts on the following day.
        </p>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))', gap: '1rem' }}>
          {data.forbiddenShiftTypeSuccessions.filter(f => f.succeedingShiftTypes.length > 0).map((f, idx) => (
            <div key={idx} style={{ backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--border-color)' }}>
              <div style={{ fontWeight: 600, marginBottom: '0.5rem', color: 'var(--text-main)' }}>
                After working <span style={{ color: 'var(--accent-color)' }}>{f.precedingShiftType}</span>:
              </div>
              <ul style={{ margin: 0, paddingLeft: '1.5rem', color: 'var(--danger-color)' }}>
                {f.succeedingShiftTypes.map((succ, sIdx) => (
                  <li key={sIdx}>Cannot work {succ}</li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
