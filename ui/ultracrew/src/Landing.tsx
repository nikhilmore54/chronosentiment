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
      <div style={{ maxWidth: '800px' }}>
        <h1 style={{ fontSize: '3rem', color: 'var(--accent-color)', marginBottom: '2rem', fontWeight: 700, letterSpacing: '-0.02em' }}>
          UltraCrew
        </h1>
        
        <div style={{ fontSize: '2rem', fontWeight: 600, lineHeight: 1.4, marginBottom: '3rem' }}>
          <span style={{ color: 'var(--text-muted)' }}>Employee was sick.</span><br />
          <span style={{ color: 'var(--text-muted)' }}>Traditional schedulers forget.</span><br />
          <span style={{ color: 'var(--text-main)' }}>UltraCrew remembers.</span>
        </div>

        <button 
          onClick={onStartDemo}
          style={{
            backgroundColor: 'var(--primary-color)',
            color: 'white',
            border: 'none',
            padding: '1rem 2.5rem',
            borderRadius: '8px',
            fontSize: '1.25rem',
            fontWeight: 600,
            cursor: 'pointer',
            marginBottom: '4rem',
            boxShadow: '0 4px 6px -1px rgba(37, 99, 235, 0.4)',
            transition: 'transform 0.1s ease-in-out'
          }}
          onMouseOver={(e) => e.currentTarget.style.transform = 'scale(1.05)'}
          onMouseOut={(e) => e.currentTarget.style.transform = 'scale(1)'}
        >
          Watch 60 Second Demo
        </button>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', alignItems: 'center', fontSize: '1.1rem', color: 'var(--text-muted)' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
            <span style={{ color: 'var(--success-color)', fontWeight: 'bold' }}>✓</span> Fair workload recovery
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
            <span style={{ color: 'var(--success-color)', fontWeight: 'bold' }}>✓</span> Reduced scheduling complaints
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
            <span style={{ color: 'var(--success-color)', fontWeight: 'bold' }}>✓</span> Historical balancing
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
            <span style={{ color: 'var(--success-color)', fontWeight: 'bold' }}>✓</span> Explainable assignments
          </div>
        </div>
      </div>
    </div>
  );
};
