import { OperatorShell } from './operator';

function App() {
  return (
    <div
      style={{
        padding: '20px',
        fontFamily: 'IBM Plex Sans, Segoe UI, sans-serif',
        backgroundColor: '#0a0a0a',
        color: '#e5e5e5',
        minHeight: '100vh',
      }}
    >
      <header
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '30px',
          borderBottom: '1px solid #333',
          paddingBottom: '20px',
          flexWrap: 'wrap',
          gap: '15px',
        }}
      >
        <div>
          <h1
            style={{
              margin: 0,
              fontSize: '24px',
              fontWeight: 700,
              letterSpacing: '-0.02em',
              color: '#2dd4bf',
            }}
          >
            Gate5 Operator
          </h1>
          <p style={{ margin: '5px 0 0 0', color: '#888', fontSize: '14px' }}>
            Pi telemetry · 0% authority · Crypto · NSE · Coralys input gate
          </p>
        </div>
      </header>

      <main
        style={{
          backgroundColor: '#111',
          padding: '20px',
          borderRadius: '8px',
          border: '1px solid #222',
        }}
      >
        <OperatorShell />
      </main>
    </div>
  );
}

export default App;
