"use client";

import { motion } from "framer-motion";
import { cn } from "@/lib/utils";

interface AuroraProps {
  className?: string;
  colorStops?: string[];
  amplitude?: number;
}

export function Aurora({
  className,
  colorStops = ["#2a2a29", "#1a1a19", "#353534"],
}: AuroraProps) {
  return (
    <div
      className={cn(
        "absolute inset-0 overflow-hidden opacity-30 pointer-events-none",
        className
      )}
    >
      <div className="absolute -inset-[100%] opacity-50">
        <motion.div
          animate={{
            rotate: [0, 360],
            scale: [1, 1.1, 1],
          }}
          transition={{
            duration: 20,
            repeat: Infinity,
            ease: "linear",
          }}
          className="absolute inset-0 bg-gradient-to-tr"
          style={{
            backgroundImage: `radial-gradient(ellipse at 50% 50%, ${colorStops[0]} 0%, transparent 50%), radial-gradient(ellipse at 100% 0%, ${colorStops[1]} 0%, transparent 50%), radial-gradient(ellipse at 0% 100%, ${colorStops[2]} 0%, transparent 50%)`,
            filter: "blur(60px)",
          }}
        />
      </div>
    </div>
  );
}