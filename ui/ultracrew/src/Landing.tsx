interface LandingProps {
  onStartDemo: () => void;
}

export const Landing = ({ onStartDemo }: LandingProps) => {
  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: '100vh',
      backgroundColor: 'var(--bg-color)',
      color: 'var(--text-main)',
      padding: '2rem',
      textAlign: 'center'
    }}>
      <div style={{ maxWidth: '820px' }}>
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

        <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--primary-color)', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: '0.5rem' }}>
          UltraRoster · Nursing Workforce Planning
        </div>

        <h1 style={{ fontSize: '2.8rem', color: 'var(--text-main)', marginBottom: '1.25rem', fontWeight: 800, letterSpacing: '-0.03em', lineHeight: 1.25 }}>
          Better rosters. Clearer decisions.<br />More balanced teams.
        </h1>

        <p style={{ fontSize: '1.2rem', color: 'var(--text-muted)', maxWidth: '640px', margin: '0 auto 2.5rem auto', lineHeight: 1.55 }}>
          Plan nursing workforce schedules with hospital-approved constraints, transparent alternatives and human oversight.
        </p>

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
            Experience UltraRoster with synthetic data
          </span>
        </div>

        <div style={{ display: 'flex', gap: '1.25rem', justifyContent: 'center', alignItems: 'center', flexWrap: 'wrap', fontSize: '0.95rem', color: 'var(--text-muted)' }}>
          <span style={{ fontWeight: 500 }}>Policy-aware planning</span>
          <span style={{ opacity: 0.4 }}>·</span>
          <span style={{ fontWeight: 500 }}>Fairness visibility</span>
          <span style={{ opacity: 0.4 }}>·</span>
          <span style={{ fontWeight: 500 }}>Explainable decisions</span>
        </div>
      </div>
    </div>
  );
};
