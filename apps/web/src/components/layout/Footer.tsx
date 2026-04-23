import Image from "next/image";
import Link from "next/link";

export function Footer() {
  return (
    <footer className="border-t border-[rgba(226,226,226,0.06)] mt-auto w-full">
      <div className="w-full max-w-7xl mx-auto px-6 lg:px-8">
        <div className="flex flex-wrap items-center justify-between py-4 md:py-0 md:h-14 gap-y-2">
          <div className="flex items-center gap-3">
            <Link href="/" className="flex items-center group flex-shrink-0 text-[var(--text-muted)] hover:text-[var(--text)] transition-colors duration-200">
              <Image
                src="/logo.svg"
                alt=""
                width={18}
                height={18}
                aria-hidden="true"
                className="h-[18px] w-auto flex-shrink-0 relative z-10 bg-transparent opacity-60 group-hover:opacity-100 transition-opacity duration-200"
                style={{ display: "block" }}
              />
              <span
                className="ml-1.5 block text-[16px] leading-none tracking-tight text-current whitespace-nowrap relative -top-[0.5px]"
                style={{ fontFamily: "var(--font-sans)", fontWeight: 600 }}
              >
                Gunmetal
              </span>
            </Link>
            <span className="text-[11px] text-[var(--text-muted)] border-l border-[rgba(226,226,226,0.1)] pl-2.5 ml-0.5">
              &copy; 2026
            </span>
          </div>

          <a
            href="https://github.com/Dhruv2mars/gunmetal"
            target="_blank"
            rel="noreferrer"
            className="group flex items-center gap-1 text-[12px] text-[var(--text-muted)] hover:text-[var(--text)] transition-colors duration-200"
            style={{ fontFamily: "var(--font-sans)", fontWeight: 500 }}
          >
            GitHub
            <svg 
              className="w-3 h-3 text-[var(--text-muted)] group-hover:text-[var(--text)] transition-all duration-300 group-hover:translate-x-0.5 group-hover:-translate-y-0.5" 
              fill="none" 
              viewBox="0 0 24 24" 
              stroke="currentColor" 
              strokeWidth={2}
            >
              <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 19.5l15-15m0 0H8.25m11.25 0v11.25" />
            </svg>
          </a>
        </div>
      </div>
    </footer>
  );
}
