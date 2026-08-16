import type { Metadata } from "next";
import "./globals.css";
import Nav from "@/components/Nav";

export const metadata: Metadata = {
  title: "ChronoSentiment — Decision Observatory",
  description: "Decisions are sealed when made. Evidence arrives later.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body style={{ background: "var(--bg-primary)", color: "var(--text-primary)", minHeight: "100vh" }}>
        <Nav />
        <main style={{ paddingTop: "60px" }}>
          {children}
        </main>
      </body>
    </html>
  );
}
