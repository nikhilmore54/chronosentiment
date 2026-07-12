import React, { useRef, useState } from 'react';
import type { StaffMember } from './WorkflowTypes';
import { SAMPLE_CSV } from './WorkflowTypes';
import { parseStaffCSV, buildImportSummary } from './WorkflowUtils';
import {
  primaryBtnStyle, ghostBtnStyle, accentGhostBtnStyle,
  codeBoxStyle, textareaStyle, errorBoxStyle,
  SummaryRow, SkillBadge,
} from './WorkflowComponents';

export const ImportStaff: React.FC<{
  staff: StaffMember[];
  onStaffChange: (s: StaffMember[]) => void;
  onNext: () => void;
}> = ({ staff, onStaffChange, onNext }) => {
  const fileRef = useRef<HTMLInputElement>(null);
  const [errors, setErrors] = useState<string[]>([]);
  const [showFormat, setShowFormat] = useState(false);

  const processText = (text: string) => {
    const { staff: parsed, errors: errs } = parseStaffCSV(text);
    setErrors(errs);
    if (parsed.length > 0) onStaffChange(parsed);
  };

  const summary = staff.length > 0 ? buildImportSummary(staff) : null;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Import Staff</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Upload a CSV file with your staff roster. Required columns: <code>id</code>, <code>contract</code>, <code>skills</code>.
        </p>
      </div>

      <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'center' }}>
        <label style={primaryBtnStyle}>
          Upload CSV
          <input ref={fileRef} type="file" accept=".csv,.txt" onChange={e => {
            const file = e.target.files?.[0];
            if (!file) return;
            const reader = new FileReader();
            reader.onload = ev => processText(ev.target?.result as string);
            reader.readAsText(file);
            e.target.value = '';
          }} style={{ display: 'none' }} />
        </label>

        <button onClick={() => setShowFormat(s => !s)} style={ghostBtnStyle}>
          {showFormat ? 'Hide Format' : 'Show CSV Format'}
        </button>

        <button onClick={() => {
          const { staff: parsed } = parseStaffCSV(SAMPLE_CSV);
          setErrors([]);
          onStaffChange(parsed);
        }} style={accentGhostBtnStyle}>
          Load Sample (8 staff)
        </button>

        {staff.length > 0 && (
          <span style={{ color: 'var(--success-color)', fontSize: '0.9rem', fontWeight: 600 }}>
            ✓ {staff.length} staff members loaded
          </span>
        )}
      </div>

      {showFormat && (
        <div style={codeBoxStyle}>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginBottom: '0.5rem' }}>
            CSV format — paste below or upload file:
          </div>
          <pre style={{ margin: 0, fontSize: '0.8rem', color: 'var(--accent-color)', fontFamily: 'monospace' }}>
            {SAMPLE_CSV}
          </pre>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '0.5rem' }}>
            Skills: semicolon-separated (e.g. <code>HeadNurse;Nurse</code>). Contracts: FullTime / PartTime / Night.
          </div>
          <textarea
            placeholder="Paste CSV here..."
            onPaste={e => processText(e.clipboardData.getData('text'))}
            style={{ ...textareaStyle, marginTop: '0.75rem' }}
          />
        </div>
      )}

      {errors.length > 0 && (
        <div style={errorBoxStyle}>
          {errors.map((e, i) => <div key={i} style={{ fontSize: '0.85rem' }}>⚠ {e}</div>)}
        </div>
      )}

      {summary && (
        <div style={{
          backgroundColor: 'rgba(34, 197, 94, 0.06)',
          border: '1px solid rgba(34, 197, 94, 0.3)',
          borderRadius: '8px',
          padding: '1rem 1.25rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.5rem',
        }}>
          <div style={{ fontWeight: 700, color: 'var(--success-color)', marginBottom: '0.25rem' }}>Import validated</div>
          <SummaryRow icon="✓" label={`${summary.staffCount} staff members`} color="var(--success-color)" />
          <SummaryRow icon="✓" label={`${summary.contracts.length} contract types: ${summary.contracts.join(', ')}`} color="var(--success-color)" />
          <SummaryRow icon="✓" label={`${summary.skills.length} skills: ${summary.skills.join(', ')}`} color="var(--success-color)" />
          {summary.warnings.map((w, i) => <SummaryRow key={i} icon="⚠" label={w} color="#f59e0b" />)}
        </div>
      )}

      {staff.length > 0 && (
        <div style={{ overflowX: 'auto', maxHeight: '280px', overflowY: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.88rem' }}>
            <thead>
              <tr style={{ borderBottom: '2px solid var(--border-color)', textAlign: 'left' }}>
                {['ID', 'Contract', 'Skills'].map(h => (
                  <th key={h} style={{ padding: '0.5rem 0.75rem', color: 'var(--text-muted)', fontWeight: 600 }}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {staff.map((s, i) => (
                <tr key={i} style={{ borderBottom: '1px solid var(--border-color)' }}>
                  <td style={{ padding: '0.5rem 0.75rem', fontWeight: 600 }}>{s.id}</td>
                  <td style={{ padding: '0.5rem 0.75rem', color: 'var(--text-muted)' }}>{s.contract}</td>
                  <td style={{ padding: '0.5rem 0.75rem' }}>{s.skills.map(sk => <SkillBadge key={sk} skill={sk} />)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
        <button
          onClick={onNext}
          disabled={staff.length === 0}
          style={{ ...primaryBtnStyle, opacity: staff.length === 0 ? 0.4 : 1, cursor: staff.length === 0 ? 'not-allowed' : 'pointer' }}
        >
          Next: Select Rules →
        </button>
      </div>
    </div>
  );
};