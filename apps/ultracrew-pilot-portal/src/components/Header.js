import React from 'react';
import { S, theme } from '../styles';

export default function Header({ activeTab, onSelectTab, status = 'online' }) {
  return (
    <header style={S.header}>
      <div style={S.headerBrand}>
        <div style={S.logoBadge}>✈</div>
        <div>
          <div style={S.headerTitle}>UltraCrew AOCC</div>
          <div style={S.headerSub}>Coralys v2.0 Airline Crew Optimization Suite</div>
        </div>
      </div>

      <nav style={S.navTabs}>
        <button
          style={S.navTab(activeTab === 'benchmark')}
          onClick={() => onSelectTab('benchmark')}
        >
          <span>🗂</span> Benchmarks
        </button>
        <button
          style={S.navTab(activeTab === 'solver')}
          onClick={() => onSelectTab('solver')}
        >
          <span>⚡</span> Solver & Controls
        </button>
        <button
          style={S.navTab(activeTab === 'audit')}
          onClick={() => onSelectTab('audit')}
        >
          <span>🛡</span> Two-Tier Audit
        </button>
        <button
          style={S.navTab(activeTab === 'gantt')}
          onClick={() => onSelectTab('gantt')}
        >
          <span>🗓</span> Roster Gantt
        </button>
        <button
          style={S.navTab(activeTab === 'irops')}
          onClick={() => onSelectTab('irops')}
        >
          <span>🚨</span> OCC IROPS Disruption
        </button>
        <button
          style={S.navTab(activeTab === 'evidence')}
          onClick={() => onSelectTab('evidence')}
        >
          <span>📈</span> Commercial Session
        </button>
      </nav>

      <div style={S.headerBadges}>
        <span style={S.statusChip(status === 'online' ? 'active' : 'idle')}>
          <span
            style={{
              width: '6px',
              height: '6px',
              borderRadius: '50%',
              background: status === 'online' ? theme.success : theme.indigo,
            }}
          />
          {status === 'online' ? 'ENGINE READY' : 'OFFLINE'}
        </span>
        <span style={S.badge('success')}>TC CAR 700</span>
      </div>
    </header>
  );
}
