"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";

const operateLinks = [
  { href: "/live", label: "Live" },
  { href: "/decisions", label: "Decisions" },
  { href: "/portfolio", label: "Portfolio" },
];

const proveLinks = [
  { href: "/", label: "Observatory", exact: true },
  { href: "/replay", label: "Replay" },
  { href: "/audit", label: "Audit" },
  { href: "/provenance", label: "Provenance" },
];

export default function Nav() {
  const pathname = usePathname();

  function isActive(href: string, exact?: boolean) {
    if (exact) return pathname === href;
    return pathname === href || (href !== "/" && pathname.startsWith(href));
  }

  return (
    <nav
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        zIndex: 100,
        height: "60px",
        background: "rgba(10, 12, 15, 0.95)",
        backdropFilter: "blur(12px)",
        borderBottom: "1px solid var(--border)",
        display: "flex",
        alignItems: "center",
        padding: "0 24px",
        gap: "0",
      }}
    >
      {/* Logo */}
      <Link href="/live" style={{ textDecoration: "none", marginRight: "24px", flexShrink: 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <div
            style={{
              width: "28px",
              height: "28px",
              borderRadius: "6px",
              background: "linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: "14px",
              fontWeight: "700",
              color: "white",
            }}
          >
            CS
          </div>
          <span style={{ fontSize: "14px", fontWeight: "600", color: "var(--text-primary)", letterSpacing: "-0.01em" }}>
            ChronoSentiment
          </span>
        </div>
      </Link>

      {/* OPERATE group */}
      <div style={{ display: "flex", alignItems: "center", gap: "2px", marginRight: "8px" }}>
        <span style={{ fontSize: "9px", fontWeight: "700", color: "#3b82f6", letterSpacing: "0.1em", textTransform: "uppercase", marginRight: "6px" }}>OPERATE</span>
        {operateLinks.map((link) => {
          const active = isActive(link.href);
          return (
            <Link
              key={link.href}
              href={link.href}
              style={{
                textDecoration: "none",
                padding: "5px 11px",
                borderRadius: "6px",
                fontSize: "13px",
                fontWeight: active ? "600" : "400",
                color: active ? "#3b82f6" : "var(--text-secondary)",
                background: active ? "rgba(59,130,246,0.1)" : "transparent",
                border: active ? "1px solid rgba(59,130,246,0.25)" : "1px solid transparent",
                transition: "all 0.15s ease",
                display: "flex",
                alignItems: "center",
                gap: "5px",
              }}
            >
              {link.href === "/live" && (
                <div style={{ width: "5px", height: "5px", borderRadius: "50%", background: "#3b82f6", flexShrink: 0 }} />
              )}
              {link.label}
            </Link>
          );
        })}
      </div>

      {/* Divider */}
      <div style={{ width: "1px", height: "20px", background: "var(--border)", margin: "0 12px", flexShrink: 0 }} />

      {/* PROVE group */}
      <div style={{ display: "flex", alignItems: "center", gap: "2px", flex: 1 }}>
        <span style={{ fontSize: "9px", fontWeight: "700", color: "#10b981", letterSpacing: "0.1em", textTransform: "uppercase", marginRight: "6px" }}>PROVE</span>
        {proveLinks.map((link) => {
          const active = isActive(link.href, link.exact);
          return (
            <Link
              key={link.href}
              href={link.href}
              style={{
                textDecoration: "none",
                padding: "5px 11px",
                borderRadius: "6px",
                fontSize: "13px",
                fontWeight: active ? "600" : "400",
                color: active ? "var(--text-primary)" : "var(--text-secondary)",
                background: active ? "var(--bg-card)" : "transparent",
                border: active ? "1px solid var(--border)" : "1px solid transparent",
                transition: "all 0.15s ease",
              }}
            >
              {link.label}
            </Link>
          );
        })}
      </div>

      {/* Right side */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px", flexShrink: 0 }}>
        <div
          style={{
            padding: "4px 10px",
            borderRadius: "4px",
            background: "rgba(245, 158, 11, 0.1)",
            border: "1px solid rgba(245, 158, 11, 0.2)",
            fontSize: "11px",
            fontWeight: "600",
            color: "#f59e0b",
            letterSpacing: "0.05em",
            textTransform: "uppercase",
          }}
        >
          Paper / Research
        </div>
      </div>
    </nav>
  );
}