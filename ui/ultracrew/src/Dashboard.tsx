import { useEffect, useState } from 'react';

export interface Coverage {
  covered: number;
  understaffed: number;
  critical: number;
}

export interface Alert {
  employee: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
}

export interface DashboardData {
  coverage: Coverage;
  alerts: Alert[];
  recommendations: string[];
}

import type { SimulationState } from './App';

export const Dashboard = ({ simulationState }: { simulationState?: SimulationState | null }) => {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch('http://127.0.0.1:3000/api/dashboard')
      .then(res => res.json())
      .then(d => {
        setData(d);
        setLoading(false);
      })
      .catch(err => {
        console.error('Failed to load dashboard:', err);
        setLoading(false);
      });
  }, []);

  if (loading || !data) {
    return <p>Loading Dashboard...</p>;
  }

  const getAlertColor = (severity: string) => {
    switch (severity) {
      case 'critical': return '#991b1b'; // dark red
      case 'high': return '#ef4444'; // red
      case 'medium': return '#f59e0b'; // amber
      case 'low': return '#3b82f6'; // blue
      case 'resolved': return 'var(--success-color)'; // green
      default: return 'var(--text-muted)';
    }
  };

  let displayData = data;
  if (simulationState && displayData) {
    displayData = {
      ...displayData,
      coverage: {
        covered: 88,
        understaffed: 8,
        critical: 3
      },
      alerts: [
        {
          employee: simulationState.affected_nurse,
          severity: 'high',
          message: `Behind schedule by ${simulationState.missed_shifts} shifts due to sickness`
        },
        ...displayData.alerts.filter(a => a.employee !== simulationState.affected_nurse)
      ]
    };
  }

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <h2>Workforce Health</h2>
      
      {/* Row 1: Coverage Status */}
      <div>
        <h3 style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '1rem' }}>
          Am I Covered?
        </h3>
        <div style={{ display: 'flex', gap: '1rem' }}>
          <div style={{ padding: '1.25rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1, border: '1px solid var(--border-color)' }}>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Fully Covered Shifts</div>
            <div style={{ fontSize: '2rem', fontWeight: 600, color: 'var(--success-color)' }}>{displayData.coverage.covered}%</div>
          </div>
          <div style={{ padding: '1.25rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1, border: '1px solid var(--border-color)' }}>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Understaffed Shifts</div>
            <div style={{ fontSize: '2rem', fontWeight: 600, color: 'var(--accent-color)' }}>{displayData.coverage.understaffed}</div>
          </div>
          <div style={{ padding: '1.25rem', backgroundColor: 'var(--bg-color)', borderRadius: '8px', flex: 1, border: '1px solid var(--border-color)' }}>
            <div style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Critical Gaps</div>
            <div style={{ fontSize: '2rem', fontWeight: 600, color: displayData.coverage.critical > 0 ? 'var(--danger-color)' : 'var(--text-main)' }}>{displayData.coverage.critical}</div>
          </div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '2rem' }}>
        {/* Row 2: Workforce Alerts */}
        <div style={{ flex: 1 }}>
          <h3 style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '1rem' }}>
            Who Needs Attention?
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            {displayData.alerts.map((alert, idx) => (
              <div key={idx} style={{ 
                padding: '1rem', 
                backgroundColor: 'var(--bg-color)', 
                borderRadius: '8px',
                borderLeft: `4px solid ${getAlertColor(alert.severity)}`
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.25rem' }}>
                  <div style={{ fontWeight: 600 }}>
                    ⚠ {alert.employee}
                  </div>
                  <span style={{ 
                    fontSize: '0.7rem', 
                    fontWeight: 600, 
                    textTransform: 'uppercase', 
                    padding: '0.1rem 0.4rem', 
                    borderRadius: '12px',
                    backgroundColor: `${getAlertColor(alert.severity)}20`,
                    color: getAlertColor(alert.severity)
                  }}>
                    {alert.severity}
                  </span>
                </div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.9rem' }}>
                  {alert.message}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Row 3: Recommended Actions */}
        <div style={{ flex: 1 }}>
          <h3 style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '1rem' }}>
            What Should I Do Next?
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            {displayData.recommendations.map((rec, idx) => (
              <div key={idx} style={{ 
                padding: '1rem', 
                backgroundColor: 'rgba(34, 197, 94, 0.1)', 
                borderRadius: '8px',
                border: '1px solid rgba(34, 197, 94, 0.2)',
                color: 'var(--text-main)'
              }}>
                <span style={{ color: 'var(--success-color)', marginRight: '0.5rem', fontWeight: 'bold' }}>✓</span> 
                {rec}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
