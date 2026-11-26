// UltraCrew AOCC Design System — Dark Aerospace / Operations Intelligence

export const theme = {
  bg: '#080c14',
  surface: '#0f172a',
  surfaceCard: '#131e36',
  surfaceCardHover: '#182747',
  surfaceBorder: '#1e2d4d',
  surfaceBorderActive: '#0284c7',
  
  textPrimary: '#f8fafc',
  textSecondary: '#94a3b8',
  textMuted: '#64748b',
  
  primary: '#0ea5e9',
  primaryHover: '#38bdf8',
  primaryGlow: 'rgba(14, 165, 233, 0.15)',
  
  success: '#10b981',
  successBg: 'rgba(16, 185, 129, 0.12)',
  successBorder: 'rgba(16, 185, 129, 0.3)',
  
  warning: '#f59e0b',
  warningBg: 'rgba(245, 158, 11, 0.12)',
  warningBorder: 'rgba(245, 158, 11, 0.3)',
  
  danger: '#ef4444',
  dangerBg: 'rgba(239, 68, 68, 0.12)',
  dangerBorder: 'rgba(239, 68, 68, 0.3)',
  
  indigo: '#6366f1',
  indigoBg: 'rgba(99, 102, 241, 0.12)',
  
  fontMono: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace",
  fontSans: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
};

export const S = {
  app: {
    minHeight: '100vh',
    background: 'radial-gradient(ellipse at 50% 0%, #0d1b33 0%, #080c14 70%)',
    color: theme.textPrimary,
    fontFamily: theme.fontSans,
    display: 'flex',
    flexDirection: 'column',
    boxSizing: 'border-box',
  },
  
  header: {
    background: 'rgba(15, 23, 42, 0.85)',
    backdropFilter: 'blur(12px)',
    borderBottom: `1px solid ${theme.surfaceBorder}`,
    padding: '14px 28px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    position: 'sticky',
    top: 0,
    zIndex: 100,
  },
  
  headerBrand: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  
  logoBadge: {
    width: '36px',
    height: '36px',
    borderRadius: '8px',
    background: 'linear-gradient(135deg, #0284c7 0%, #6366f1 100%)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontWeight: '900',
    fontSize: '18px',
    color: '#fff',
    boxShadow: '0 0 16px rgba(14, 165, 233, 0.4)',
  },
  
  headerTitle: {
    fontSize: '17px',
    fontWeight: '700',
    letterSpacing: '-0.02em',
    color: '#f8fafc',
  },
  
  headerSub: {
    fontSize: '12px',
    color: theme.textMuted,
    fontFamily: theme.fontMono,
  },
  
  headerBadges: {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
  },
  
  statusChip: (status) => ({
    display: 'inline-flex',
    alignItems: 'center',
    gap: '6px',
    padding: '4px 10px',
    borderRadius: '20px',
    fontSize: '11px',
    fontWeight: '600',
    fontFamily: theme.fontMono,
    background: status === 'active' ? theme.successBg : theme.indigoBg,
    color: status === 'active' ? theme.success : theme.indigo,
    border: `1px solid ${status === 'active' ? theme.successBorder : 'rgba(99, 102, 241, 0.3)'}`,
  }),
  
  navTabs: {
    display: 'flex',
    gap: '4px',
    padding: '4px',
    background: 'rgba(15, 23, 42, 0.6)',
    borderRadius: '10px',
    border: `1px solid ${theme.surfaceBorder}`,
  },
  
  navTab: (active) => ({
    padding: '8px 16px',
    borderRadius: '8px',
    fontSize: '13px',
    fontWeight: '600',
    cursor: 'pointer',
    background: active ? 'linear-gradient(135deg, #0284c7 0%, #2563eb 100%)' : 'transparent',
    color: active ? '#ffffff' : theme.textSecondary,
    border: 'none',
    transition: 'all 0.15s ease-in-out',
    boxShadow: active ? '0 2px 10px rgba(14, 165, 233, 0.35)' : 'none',
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  }),
  
  container: {
    maxWidth: '1440px',
    width: '100%',
    margin: '0 auto',
    padding: '24px',
    flex: 1,
    boxSizing: 'border-box',
  },
  
  card: {
    background: theme.surfaceCard,
    border: `1px solid ${theme.surfaceBorder}`,
    borderRadius: '14px',
    padding: '24px',
    marginBottom: '24px',
    boxShadow: '0 8px 24px rgba(0, 0, 0, 0.3)',
    position: 'relative',
    overflow: 'hidden',
  },
  
  cardHeader: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: '20px',
  },
  
  cardTitle: {
    fontSize: '16px',
    fontWeight: '700',
    color: '#f8fafc',
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
  },
  
  cardSubtitle: {
    fontSize: '13px',
    color: theme.textSecondary,
    marginTop: '4px',
  },
  
  grid2: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(480px, 1fr))',
    gap: '20px',
    marginBottom: '24px',
  },
  
  grid3: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
    gap: '16px',
    marginBottom: '24px',
  },
  
  grid4: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
    gap: '14px',
    marginBottom: '24px',
  },
  
  statBox: (variant = 'primary') => ({
    background: 'rgba(15, 23, 42, 0.75)',
    border: `1px solid ${variant === 'success' ? theme.successBorder : variant === 'warning' ? theme.warningBorder : theme.surfaceBorder}`,
    borderRadius: '10px',
    padding: '16px',
    display: 'flex',
    flexDirection: 'column',
    position: 'relative',
  }),
  
  statLabel: {
    fontSize: '11px',
    fontWeight: '600',
    textTransform: 'uppercase',
    letterSpacing: '0.06em',
    color: theme.textMuted,
    marginBottom: '6px',
  },
  
  statValue: (variant = 'primary') => ({
    fontSize: '24px',
    fontWeight: '800',
    fontFamily: theme.fontMono,
    color: variant === 'success' ? theme.success : variant === 'warning' ? theme.warning : variant === 'danger' ? theme.danger : theme.primaryHover,
  }),
  
  statMeta: {
    fontSize: '11px',
    color: theme.textSecondary,
    marginTop: '6px',
  },
  
  btn: (variant = 'primary', size = 'md') => ({
    padding: size === 'sm' ? '6px 12px' : size === 'lg' ? '12px 24px' : '10px 18px',
    fontSize: size === 'sm' ? '12px' : size === 'lg' ? '15px' : '13px',
    fontWeight: '700',
    borderRadius: '8px',
    border: 'none',
    cursor: 'pointer',
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '8px',
    transition: 'all 0.15s ease-in-out',
    color: '#ffffff',
    background: variant === 'primary'
      ? 'linear-gradient(135deg, #0284c7 0%, #2563eb 100%)'
      : variant === 'success'
      ? 'linear-gradient(135deg, #10b981 0%, #059669 100%)'
      : variant === 'danger'
      ? 'linear-gradient(135deg, #ef4444 0%, #dc2626 100%)'
      : 'rgba(30, 41, 59, 0.8)',
    boxShadow: variant === 'primary' ? '0 4px 14px rgba(14, 165, 233, 0.4)' : 'none',
  }),
  
  input: {
    width: '100%',
    background: 'rgba(15, 23, 42, 0.9)',
    border: `1px solid ${theme.surfaceBorder}`,
    borderRadius: '8px',
    padding: '10px 14px',
    color: '#f8fafc',
    fontSize: '13px',
    fontFamily: 'inherit',
    outline: 'none',
    boxSizing: 'border-box',
  },
  
  select: {
    width: '100%',
    background: '#0f172a',
    border: `1px solid ${theme.surfaceBorder}`,
    borderRadius: '8px',
    padding: '10px 14px',
    color: '#f8fafc',
    fontSize: '13px',
    fontFamily: 'inherit',
    outline: 'none',
    boxSizing: 'border-box',
    cursor: 'pointer',
  },
  
  table: {
    width: '100%',
    borderCollapse: 'collapse',
    fontSize: '13px',
    textAlign: 'left',
  },
  
  th: {
    padding: '12px 14px',
    background: 'rgba(15, 23, 42, 0.9)',
    borderBottom: `2px solid ${theme.surfaceBorder}`,
    color: theme.textMuted,
    fontWeight: '700',
    fontSize: '11px',
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
  },
  
  td: {
    padding: '12px 14px',
    borderBottom: `1px solid ${theme.surfaceBorder}`,
    color: theme.textPrimary,
  },
  
  badge: (type = 'neutral') => ({
    display: 'inline-flex',
    alignItems: 'center',
    gap: '4px',
    padding: '2px 8px',
    borderRadius: '4px',
    fontSize: '11px',
    fontWeight: '700',
    fontFamily: theme.fontMono,
    background: type === 'success' ? theme.successBg : type === 'warning' ? theme.warningBg : type === 'danger' ? theme.dangerBg : 'rgba(255, 255, 255, 0.08)',
    color: type === 'success' ? theme.success : type === 'warning' ? theme.warning : type === 'danger' ? theme.danger : theme.textSecondary,
    border: `1px solid ${type === 'success' ? theme.successBorder : type === 'warning' ? theme.warningBorder : type === 'danger' ? theme.dangerBorder : 'rgba(255, 255, 255, 0.12)'}`,
  }),
};
