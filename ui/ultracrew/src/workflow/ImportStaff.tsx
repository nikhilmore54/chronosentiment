import React, { useRef, useState } from 'react';
import type { StaffMember } from './WorkflowTypes';
import { SAMPLE_CSV } from './WorkflowTypes';
import {
  parseStaffCSV,
  parseShiftsCSV,
  buildImportSummary,
  type CustomShiftInput,
  type ShiftSlotMeta,
} from './WorkflowUtils';
import {
  primaryBtnStyle, ghostBtnStyle, accentGhostBtnStyle,
  codeBoxStyle, textareaStyle, errorBoxStyle,
  SummaryRow, SkillBadge,
} from './WorkflowComponents';

function generateVariableWeekendFixture(): ShiftSlotMeta[] {
  const shifts: ShiftSlotMeta[] = [];
  let slotId = 1;
  for (let day = 0; day < 28; day++) {
    const isWeekend = (day % 7 === 5) || (day % 7 === 6); // Sat (5), Sun (6)
    if (isWeekend) {
      // Reduced weekend demand: 4 positions (2 Early, 1 Late, 1 Night)
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'Nurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'HeadNurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Late', skill: 'Nurse', startHour: day * 24 + 14, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Night', skill: 'Nurse', startHour: day * 24 + 22, durationHours: 8 });
    } else {
      // Standard weekday demand: 7 positions (3 Early, 2 Late, 2 Night)
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'Nurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'Nurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Early', skill: 'HeadNurse', startHour: day * 24 + 7, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Late', skill: 'Nurse', startHour: day * 24 + 14, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Late', skill: 'Nurse', startHour: day * 24 + 14, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Night', skill: 'Nurse', startHour: day * 24 + 22, durationHours: 8 });
      shifts.push({ shiftId: slotId++, dayIdx: day, label: 'Night', skill: 'Nurse', startHour: day * 24 + 22, durationHours: 8 });
    }
  }
  return shifts;
}

export const ImportStaff: React.FC<{
  staff: StaffMember[];
  onStaffChange: (s: StaffMember[]) => void;
  customShifts?: CustomShiftInput[];
  onCustomShiftsChange?: (shifts: CustomShiftInput[]) => void;
  onNext: () => void;
}> = ({ staff, onStaffChange, customShifts, onCustomShiftsChange, onNext }) => {
  const fileRef = useRef<HTMLInputElement>(null);
  const shiftFileRef = useRef<HTMLInputElement>(null);
  const [errors, setErrors] = useState<string[]>([]);
  const [shiftErrors, setShiftErrors] = useState<string[]>([]);
  const [showFormat, setShowFormat] = useState(false);

  const processText = (text: string) => {
    const { staff: parsed, errors: errs } = parseStaffCSV(text);
    setErrors(errs);
    if (parsed.length > 0) onStaffChange(parsed);
  };

  const processShiftsText = (text: string) => {
    const { shifts: parsed, errors: errs } = parseShiftsCSV(text);
    setShiftErrors(errs);
    if (parsed.length > 0 && onCustomShiftsChange) {
      onCustomShiftsChange(parsed);
    }
  };

  const summary = staff.length > 0 ? buildImportSummary(staff) : null;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div>
        <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--text-main)' }}>Import Staff &amp; Demand</h3>
        <p style={{ margin: 0, color: 'var(--text-muted)', fontSize: '0.9rem' }}>
          Upload your hospital staff roster and optional department shift demand.
        </p>
      </div>

      <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'center' }}>
        <label style={primaryBtnStyle}>
          Upload Staff CSV
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
          Load Sample Staff (20 staff)
        </button>

        <button onClick={() => {
          const template = `id,contract,skills\nAlice_Smith,FullTime,Nurse\nBob_Jones,PartTime,HeadNurse;Nurse\nCarla_Night,Night,Nurse`;
          const blob = new Blob([template], { type: 'text/csv' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = 'ultracrew_staff_template.csv';
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          URL.revokeObjectURL(url);
        }} style={ghostBtnStyle}>
          ⬇ Staff Template
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
            Staff CSV format — paste below or upload file:
          </div>
          <pre style={{ margin: 0, fontSize: '0.8rem', color: 'var(--accent-color)', fontFamily: 'monospace' }}>
            {SAMPLE_CSV}
          </pre>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '0.5rem' }}>
            Skills: semicolon-separated (e.g. <code>HeadNurse;Nurse</code>). Contracts: FullTime / PartTime / Night.
          </div>
          <textarea
            placeholder="Paste staff CSV here..."
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

      {/* ── Optional Shift Demand Section ── */}
      <div style={{
        backgroundColor: 'var(--panel-bg)',
        border: '1px solid var(--border-color)',
        borderRadius: '8px',
        padding: '1.25rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.85rem',
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
          <div>
            <div style={{ fontWeight: 700, color: 'var(--text-main)', fontSize: '0.95rem' }}>
              Shift Demand &amp; Requirements
            </div>
            <div style={{ fontSize: '0.82rem', color: 'var(--text-muted)' }}>
              Configure hospital shift requirements or use standard hospital baseline (196 shifts across 28 days).
            </div>
          </div>
          {customShifts && customShifts.length > 0 ? (
            <span style={{ color: 'var(--accent-color)', fontSize: '0.85rem', fontWeight: 600 }}>
              ⚡ Custom demand active: {customShifts.length} shifts
            </span>
          ) : (
            <span style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
              Standard fixture active (196 shifts: 7 daily)
            </span>
          )}
        </div>

        <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap', alignItems: 'center' }}>
          <label style={{ ...accentGhostBtnStyle, cursor: 'pointer' }}>
            Upload shifts.csv
            <input ref={shiftFileRef} type="file" accept=".csv,.txt" onChange={e => {
              const file = e.target.files?.[0];
              if (!file) return;
              const reader = new FileReader();
              reader.onload = ev => processShiftsText(ev.target?.result as string);
              reader.readAsText(file);
              e.target.value = '';
            }} style={{ display: 'none' }} />
          </label>

          <button onClick={() => {
            const varWeekend = generateVariableWeekendFixture();
            setShiftErrors([]);
            if (onCustomShiftsChange) onCustomShiftsChange(varWeekend);
          }} style={ghostBtnStyle}>
            Load Variable Weekend Demand (172 shifts)
          </button>

          {customShifts && customShifts.length > 0 && (
            <button onClick={() => {
              setShiftErrors([]);
              if (onCustomShiftsChange) onCustomShiftsChange([]);
            }} style={{ ...ghostBtnStyle, color: '#f59e0b' }}>
              Reset to Standard (196 shifts)
            </button>
          )}

          <button onClick={() => {
            const template = `id,start_hour,duration_hours,required_skill\n0,7,8,Nurse\n1,7,8,HeadNurse\n2,14,8,Nurse\n3,22,8,Nurse`;
            const blob = new Blob([template], { type: 'text/csv' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'ultracrew_shifts_template.csv';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
          }} style={ghostBtnStyle}>
            ⬇ Shifts Template
          </button>
        </div>

        {shiftErrors.length > 0 && (
          <div style={errorBoxStyle}>
            {shiftErrors.map((e, i) => <div key={i} style={{ fontSize: '0.85rem' }}>⚠ {e}</div>)}
          </div>
        )}
      </div>

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
        <div style={{ overflowX: 'auto', maxHeight: '220px', overflowY: 'auto', borderRadius: '6px', border: '1px solid var(--border-color)' }}>
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
