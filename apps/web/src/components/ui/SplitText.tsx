"use client";

import { motion } from "framer-motion";

export function SplitText({ 
  children, 
  delay = 0, 
  className = "" 
}: { 
  children: string, 
  delay?: number, 
  className?: string 
}) {
  const words = children.split(" ");
  return (
    <span className={className}>
      {words.map((word, i) => (
        <span key={i} className="inline-block overflow-hidden pb-1 -mb-1">
          <motion.span
            initial={{ y: "120%", opacity: 0, rotateZ: 3 }}
            animate={{ y: 0, opacity: 1, rotateZ: 0 }}
            transition={{
              duration: 1.1,
              delay: delay + i * 0.05,
              ease: [0.16, 1, 0.3, 1], // Warp-like cinematic easing
            }}
            className="inline-block origin-bottom-left"
          >
            {word}
          </motion.span>
          <span className="inline-block">&nbsp;</span>
        </span>
      ))}
    </span>
  );
}