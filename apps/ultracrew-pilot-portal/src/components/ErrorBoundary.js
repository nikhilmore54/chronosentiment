import React from 'react';

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    console.error("ErrorBoundary caught an error:", error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ padding: '20px', background: '#450a0a', border: '1px solid #7f1d1d', borderRadius: '8px', color: '#fca5a5' }}>
          <h3 style={{ marginTop: 0 }}>Something went wrong</h3>
          <p style={{ fontSize: '14px', margin: '8px 0' }}>{this.state.error && this.state.error.toString()}</p>
          <button 
            onClick={() => this.setState({ hasError: false })}
            style={{ padding: '6px 12px', background: '#7f1d1d', color: '#fff', border: 'none', borderRadius: '4px', cursor: 'pointer', marginTop: '8px' }}
          >
            Try again
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

export default ErrorBoundary;
