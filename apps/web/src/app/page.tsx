import { Navbar } from "@/components/layout/Navbar";
import { Footer } from "@/components/layout/Footer";
import { Hero } from "@/components/sections/Hero";
import { HowItWorks } from "@/components/sections/HowItWorks";
import { Providers } from "@/components/sections/Providers";

export default function HomePage() {
  return (
    <div className="min-h-screen bg-[var(--bg)]">
      <Navbar />
      <main>
        <Hero />
        <HowItWorks />
        <Providers />
      </main>
      <Footer />
    </div>
  );
}