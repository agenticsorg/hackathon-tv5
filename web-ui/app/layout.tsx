import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Entertainment Discovery - AI-Powered Recommendations",
  description: "Find what to watch in seconds with our 8-agent AI system",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="bg-bg-dark text-text-dark-primary antialiased">
        {children}
      </body>
    </html>
  );
}
