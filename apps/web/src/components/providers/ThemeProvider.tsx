"use client";

import { createContext, useContext, useEffect, useState } from "react";

interface ThemeContextType {
  mounted: boolean;
}

const ThemeContext = createContext<ThemeContextType>({
  mounted: false,
});

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // Defer the mounted state update to after paint
    // This prevents cascading renders from setState in effect
    const rafId = requestAnimationFrame(() => {
      setMounted(true);
    });

    // Always dark — set on html directly
    document.documentElement.setAttribute("data-theme", "dark");

    return () => cancelAnimationFrame(rafId);
  }, []);

  return (
    <ThemeContext.Provider value={{ mounted }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  return useContext(ThemeContext);
}
