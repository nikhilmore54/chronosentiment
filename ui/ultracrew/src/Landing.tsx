import React from 'react';

interface LandingProps {
  onStartDemo: () => void;
}

export const Landing: React.FC<LandingProps> = ({ onStartDemo }) => {
  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: '100vh',
      backgroundColor: 'var(--bg-color)',
      color: 'var(--text-main)',
      padding: '2.5rem 1.5rem',
      textAlign: 'center'
    }}>
      <div style={{ maxWidth: '960px', width: '100%' }}>
        {/* Synthetic Demo Disclaimer Badge */}
        <div style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: '0.5rem',
          backgroundColor: 'rgba(15, 118, 110, 0.08)',
          border: '1px solid rgba(15, 118, 110, 0.25)',
          color: 'var(--primary-color)',
          fontSize: '0.8rem',
          fontWeight: 600,
          padding: '0.35rem 0.9rem',
          borderRadius: '9999px',
          marginBottom: '1.75rem',
        }}>
          <span>ℹ</span>
          <span>Demonstration Environment — Synthetic Hospital Data Only</span>
        </div>

        {/* Product Identity */}
        <div style={{
          fontSize: '0.95rem',
          fontWeight: 700,
          color: 'var(--primary-color)',
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          marginBottom: '0.5rem'
        }}>
          UltraRoster · Specialist Nursing Workforce Planning
        </div>

        {/* Focused Headline */}
        <h1 style={{
          fontSize: '2.75rem',
          color: 'var(--text-main)',
          marginBottom: '1.25rem',
          fontWeight: 800,
          letterSpacing: '-0.03em',
          lineHeight: 1.25
        }}>
          Plan your nursing workforce.<br />Make every roster decision clearer.
        </h1>

        {/* Positioning Subtitle */}
        <p style={{
          fontSize: '1.15rem',
          color: 'var(--text-muted)',
          maxWidth: '720px',
          margin: '0 auto 2.5rem auto',
          lineHeight: 1.6
        }}>
          Specialist multi-week rostering and workforce decision support for hospitals' existing nursing teams.
          Balance complex clinical constraints, maintain supervisory coverage, and evaluate independently verified
          roster alternatives with complete nursing leadership control.
        </p>

        {/* Primary CTA */}
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.75rem', marginBottom: '3.5rem' }}>
          <button 
            onClick={onStartDemo}
            style={{
              backgroundColor: 'var(--primary-color)',
              color: 'white',
              border: 'none',
              padding: '1rem 2.25rem',
              borderRadius: '8px',
              fontSize: '1.15rem',
              fontWeight: 600,
              cursor: 'pointer',
              boxShadow: '0 4px 6px -1px rgba(15, 118, 110, 0.25)',
              transition: 'transform 0.15s ease-in-out, background-color 0.15s',
            }}
            onMouseOver={(e) => {
              e.currentTarget.style.transform = 'scale(1.02)';
              e.currentTarget.style.backgroundColor = 'var(--secondary-color)';
            }}
            onMouseOut={(e) => {
              e.currentTarget.style.transform = 'scale(1)';
              e.currentTarget.style.backgroundColor = 'var(--primary-color)';
            }}
          >
            Explore a sample hospital
          </button>
          <span style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>
            Experience UltraRoster with preloaded synthetic ward data
          </span>
        </div>

        {/* Three Core Capability Sections */}
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
          gap: '1.5rem',
          textAlign: 'left',
          marginBottom: '3rem'
        }}>
          <div style={{
            backgroundColor: 'var(--panel-bg)',
            border: '1px solid var(--border-color)',
            borderRadius: '10px',
            padding: '1.5rem',
            boxShadow: '0 1px 3px 0 rgba(0, 0, 0, 0.05)',
          }}>
            <div style={{
              display: 'inline-flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '36px',
              height: '36px',
              borderRadius: '8px',
              backgroundColor: 'rgba(15, 118, 110, 0.1)',
              color: 'var(--primary-color)',
              fontWeight: 700,
              marginBottom: '1rem',
              fontSize: '1.1rem'
            }}>
              ⚖
            </div>
            <h3 style={{ margin: '0 0 0.5rem 0', fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-main)' }}>
              Hospital-Approved Constraints
            </h3>
            <p style={{ margin: 0, fontSize: '0.9rem', color: 'var(--text-muted)', lineHeight: 1.55 }}>
              Enforce mandatory 10-hour rest turnarounds, contractual weekly hour limits, and supervisory HeadNurse presence across all clinical shifts without manual scheduling errors.
            </p>
          </div>

          <div style={{
            backgroundColor: 'var(--panel-bg)',
            border: '1px solid var(--border-color)',
            borderRadius: '10px',
            padding: '1.5rem',
            boxShadow: '0 1px 3px 0 rgba(0, 0, 0, 0.05)',
          }}>
            <div style={{
              display: 'inline-flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '36px',
              height: '36px',
              borderRadius: '8px',
              backgroundColor: 'rgba(2, 132, 199, 0.1)',
              color: '#0284c7',
              fontWeight: 700,
              marginBottom: '1rem',
              fontSize: '1.1rem'
            }}>
              ◈
            </div>
            <h3 style={{ margin: '0 0 0.5rem 0', fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-main)' }}>
              Decision Assurance &amp; Alternatives
            </h3>
            <p style={{ margin: 0, fontSize: '0.9rem', color: 'var(--text-muted)', lineHeight: 1.55 }}>
              Compare mathematically verified roster candidates side-by-side. Inspect coverage, workload fairness population variance, and trade-offs before committing to a schedule.
            </p>
          </div>

          <div style={{
            backgroundColor: 'var(--panel-bg)',
            border: '1px solid var(--border-color)',
            borderRadius: '10px',
            padding: '1.5rem',
            boxShadow: '0 1px 3px 0 rgba(0, 0, 0, 0.05)',
          }}>
            <div style={{
              display: 'inline-flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '36px',
              height: '36px',
              borderRadius: '8px',
              backgroundColor: 'rgba(16, 185, 129, 0.1)',
              color: '#10b981',
              fontWeight: 700,
              marginBottom: '1rem',
              fontSize: '1.1rem'
            }}>
              ✓
            </div>
            <h3 style={{ margin: '0 0 0.5rem 0', fontSize: '1.1rem', fontWeight: 700, color: 'var(--text-main)' }}>
              Clinical Leadership Control
            </h3>
            <p style={{ margin: 0, fontSize: '0.9rem', color: 'var(--text-muted)', lineHeight: 1.55 }}>
              Built specifically for matrons and ward managers. Retain final approval authority, make locked manual adjustments with constraint guardrails, and export authoritative audits.
            </p>
          </div>
        </div>

        {/* Feature Highlights Strip */}
        <div style={{
          display: 'flex',
          gap: '1.5rem',
          justifyContent: 'center',
          alignItems: 'center',
          flexWrap: 'wrap',
          fontSize: '0.92rem',
          color: 'var(--text-muted)',
          borderTop: '1px solid var(--border-color)',
          paddingTop: '1.75rem'
        }}>
          <span style={{ fontWeight: 500 }}>Multi-week roster optimization</span>
          <span style={{ opacity: 0.35 }}>·</span>
          <span style={{ fontWeight: 500 }}>Independent validator certification</span>
          <span style={{ opacity: 0.35 }}>·</span>
          <span style={{ fontWeight: 500 }}>Workload &amp; weekend fairness</span>
          <span style={{ opacity: 0.35 }}>·</span>
          <span style={{ fontWeight: 500 }}>Zero black-box staffing</span>
        </div>
      </div>
    </div>
  );
};
