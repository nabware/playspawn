import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Playspawn",
  description: "The Playspawn website.",
  viewport: "width=device-width, initial-scale=1.0",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        {children}
      </body>
    </html>
  );
}
