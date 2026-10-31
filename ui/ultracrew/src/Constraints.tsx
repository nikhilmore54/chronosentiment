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

interface ErrorState {
  status: number | null;
  title: string;
  message: string;
  submessage?: string;
  retryAfterSec?: number | null;
}

const DEFAULT_SCENARIO_DATA: ScenarioData = {
  contracts: [
    {
      id: "FullTime",
      minimumNumberOfAssignments: 16,
      maximumNumberOfAssignments: 22,
      minimumNumberOfConsecutiveWorkingDays: 2,
      maximumNumberOfConsecutiveWorkingDays: 6,
      minimumNumberOfConsecutiveDaysOff: 2,
      maximumNumberOfConsecutiveDaysOff: 4,
      maximumNumberOfWorkingWeekends: 2,
      completeWeekends: 1,
    },
    {
      id: "PartTime",
      minimumNumberOfAssignments: 8,
      maximumNumberOfAssignments: 12,
      minimumNumberOfConsecutiveWorkingDays: 2,
      maximumNumberOfConsecutiveWorkingDays: 4,
      minimumNumberOfConsecutiveDaysOff: 2,
      maximumNumberOfConsecutiveDaysOff: 5,
      maximumNumberOfWorkingWeekends: 1,
      completeWeekends: 1,
    },
    {
      id: "Night",
      minimumNumberOfAssignments: 14,
      maximumNumberOfAssignments: 18,
      minimumNumberOfConsecutiveWorkingDays: 2,
      maximumNumberOfConsecutiveWorkingDays: 5,
      minimumNumberOfConsecutiveDaysOff: 3,
      maximumNumberOfConsecutiveDaysOff: 5,
      maximumNumberOfWorkingWeekends: 2,
      completeWeekends: 1,
    },
  ],
  forbiddenShiftTypeSuccessions: [
    { precedingShiftType: "Late", succeedingShiftTypes: ["Early"] },
    { precedingShiftType: "Night", succeedingShiftTypes: ["Early", "Late"] },
  ],
};

export const Constraints = () => {
  const [data, setData] = useState<ScenarioData | null>(null);
  const [loading, setLoading] = useState(true);
  const [errorState, setErrorState] = useState<ErrorState | null>(null);
  const [showStandardFallback, setShowStandardFallback] = useState(false);
  const [countdown, setCountdown] = useState<number | null>(null);

  const fetchPolicies = () => {
    setLoading(true);
    setErrorState(null);

    fetch(`/api/scenario`)
      .then(res => {
        if (!res.ok) {
          if (res.status === 429) {
            const retryHeader = res.headers.get('Retry-After');
            const retrySec = retryHeader ? parseInt(retryHeader, 10) : null;
            throw {
              status: 429,
              title: "Staffing policies temporarily unavailable",
              message: "The server is reachable, but it is currently limiting requests. Please wait briefly before trying again.",
              submessage: "Your workforce data has not been changed.",
              retryAfterSec: retrySec && !isNaN(retrySec) ? retrySec : null,
            };
          }
          if (res.status === 503) {
            throw {
              status: 503,
              title: "Service temporarily unavailable",
              message: "The staffing policy service is temporarily unavailable. Please retry shortly.",
            };
          }
          throw {
            status: res.status,
            title: "Policy service returned error",
            message: `Server returned status ${res.status}. Please retry.`,
          };
        }
        return res.json();
      })
      .then(d => {
        setData(d);
        setErrorState(null);
        setLoading(false);
      })
      .catch(err => {
        if (err && typeof err === 'object' && 'title' in err) {
          setErrorState(err as ErrorState);
          if ((err as ErrorState).retryAfterSec) {
            setCountdown((err as ErrorState).retryAfterSec!);
          }
        } else {
          setErrorState({
            status: null,
            title: "Backend Unreachable",
            message: "Unable to connect to the policy service. Please check your network connection.",
          });
        }
        setLoading(false);
      });
  };

  useEffect(() => {
    fetchPolicies();
  }, []);

  useEffect(() => {
    if (countdown === null || countdown <= 0) return;
    const timer = setInterval(() => {
      setCountdown(prev => (prev && prev > 1 ? prev - 1 : null));
    }, 1000);
    return () => clearInterval(timer);
  }, [countdown]);

  if (loading) {
    return <p style={{ color: 'var(--text-muted)', padding: '1.5rem' }}>Loading Staffing Policies...</p>;
  }

  const effectiveData = data || (showStandardFallback ? DEFAULT_SCENARIO_DATA : null);

  if (errorState && !showStandardFallback) {
    const is429 = errorState.status === 429;
    const is503 = errorState.status === 503;
    const badgeColor = is429 ? '#f59e0b' : is503 ? '#64748b' : '#ef4444';

    return (
      <div style={{
        margin: '1.5rem',
        padding: '1.5rem 1.75rem',
        backgroundColor: 'var(--panel-bg)',
        border: `1px solid ${badgeColor}40`,
        borderRadius: '10px',
        color: 'var(--text-main)',
        boxShadow: '0 1px 3px 0 rgba(0, 0, 0, 0.05)',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span style={{
            fontSize: '0.75rem',
            fontWeight: 700,
            textTransform: 'uppercase',
            color: badgeColor,
            backgroundColor: `${badgeColor}15`,
            padding: '0.2rem 0.5rem',
            borderRadius: '4px',
          }}>
            {is429 ? 'Rate Limited' : is503 ? 'Service Busy' : 'Connection Error'}
          </span>
          <h3 style={{ margin: 0, fontSize: '1.1rem', color: 'var(--text-main)' }}>
            {errorState.title}
          </h3>
        </div>

        <p style={{ fontSize: '0.92rem', color: 'var(--text-muted)', margin: '0.5rem 0 0.75rem 0', lineHeight: 1.5 }}>
          {errorState.message}
        </p>

        {errorState.submessage && (
          <p style={{ fontSize: '0.85rem', color: 'var(--text-muted)', margin: '0 0 1.25rem 0', fontStyle: 'italic' }}>
            {errorState.submessage}
          </p>
        )}

        <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'center' }}>
          <button
            onClick={fetchPolicies}
            disabled={countdown !== null && countdown > 0}
            style={{
              backgroundColor: 'var(--primary-color)',
              color: 'white',
              border: 'none',
              padding: '0.55rem 1.15rem',
              borderRadius: '6px',
              fontWeight: 600,
              fontSize: '0.88rem',
              cursor: countdown !== null && countdown > 0 ? 'not-allowed' : 'pointer',
              opacity: countdown !== null && countdown > 0 ? 0.6 : 1,
            }}
          >
            {countdown !== null && countdown > 0
              ? `Retry available in ${countdown}s`
              : 'Retry loading policies'}
          </button>

          <button
            onClick={() => setShowStandardFallback(true)}
            style={{
              background: 'none',
              border: '1px solid var(--border-color)',
              color: 'var(--text-main)',
              padding: '0.55rem 1.15rem',
              borderRadius: '6px',
              fontWeight: 500,
              fontSize: '0.88rem',
              cursor: 'pointer',
            }}
          >
            View Standard Hospital Policies (Preview)
          </button>
        </div>
      </div>
    );
  }

  if (!effectiveData) {
    return null;
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      {showStandardFallback && (
        <div style={{
          backgroundColor: 'rgba(15, 118, 110, 0.08)',
          border: '1px solid rgba(15, 118, 110, 0.25)',
          borderRadius: '8px',
          padding: '0.75rem 1.25rem',
          fontSize: '0.85rem',
          color: 'var(--primary-color)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}>
          <span>ℹ Showing standard hospital workforce policies (demo preview). Server policy endpoint was rate-limited or busy.</span>
          <button
            onClick={fetchPolicies}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--primary-color)',
              fontWeight: 700,
              cursor: 'pointer',
              textDecoration: 'underline',
              padding: 0,
            }}
          >
            Retry Live Connection
          </button>
        </div>
      )}

      <div className="card">
        <h2>Operational Constraints</h2>
        <p style={{ color: 'var(--text-muted)', marginBottom: '1.5rem' }}>
          The following rules are actively enforced by the UltraCrew engine when generating the schedule.
        </p>

        <h3 style={{ color: 'var(--accent-color)', fontSize: '1.1rem', marginBottom: '1rem', borderBottom: '1px solid var(--border-color)', paddingBottom: '0.5rem' }}>
          Contract Types
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
          {effectiveData.contracts.map((c) => (
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
          {effectiveData.forbiddenShiftTypeSuccessions.filter(f => f.succeedingShiftTypes.length > 0).map((f, idx) => (
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
