import React, { useEffect, useRef } from 'react';
import { Cpu, ShieldCheck, Sparkle, HardDrives } from '@phosphor-icons/react';

interface LoadingScreenProps {
  statusMessage?: string;
  isError?: boolean;
}

export const LoadingScreen: React.FC<LoadingScreenProps> = ({
  statusMessage = 'Initializing neural embedding engine...',
  isError = false,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Animated background neural network particles
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animId: number;
    let width = (canvas.width = window.innerWidth);
    let height = (canvas.height = window.innerHeight);

    const handleResize = () => {
      if (!canvas) return;
      width = canvas.width = window.innerWidth;
      height = canvas.height = window.innerHeight;
    };
    window.addEventListener('resize', handleResize);

    const particles: { x: number; y: number; vx: number; vy: number; radius: number }[] = [];
    const count = 45;

    for (let i = 0; i < count; i++) {
      particles.push({
        x: Math.random() * width,
        y: Math.random() * height,
        vx: (Math.random() - 0.5) * 0.6,
        vy: (Math.random() - 0.5) * 0.6,
        radius: Math.random() * 2 + 1.5,
      });
    }

    const render = () => {
      ctx.clearRect(0, 0, width, height);

      // Draw connections
      for (let i = 0; i < count; i++) {
        for (let j = i + 1; j < count; j++) {
          const dx = particles[i].x - particles[j].x;
          const dy = particles[i].y - particles[j].y;
          const dist = Math.sqrt(dx * dx + dy * dy);

          if (dist < 140) {
            const alpha = (1 - dist / 140) * 0.22;
            ctx.strokeStyle = `rgba(99, 102, 241, ${alpha})`;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(particles[i].x, particles[i].y);
            ctx.lineTo(particles[j].x, particles[j].y);
            ctx.stroke();
          }
        }
      }

      // Draw & update particles
      for (const p of particles) {
        p.x += p.vx;
        p.y += p.vy;

        if (p.x < 0 || p.x > width) p.vx *= -1;
        if (p.y < 0 || p.y > height) p.vy *= -1;

        ctx.beginPath();
        ctx.arc(p.x, p.y, p.radius, 0, 2 * Math.PI);
        ctx.fillStyle = '#6366f1';
        ctx.shadowColor = '#6366f1';
        ctx.shadowBlur = 8;
        ctx.fill();
        ctx.shadowBlur = 0;
      }

      animId = requestAnimationFrame(render);
    };

    render();

    return () => {
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(animId);
    };
  }, []);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[#090a0f] overflow-hidden select-none">
      {/* Animated neural particles canvas in background */}
      <canvas ref={canvasRef} className="absolute inset-0 pointer-events-none opacity-40" />

      {/* Radial spotlight effect */}
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(99,102,241,0.12)_0,rgba(9,10,15,0.95)_70%)] pointer-events-none" />

      {/* Main glass card container */}
      <div className="relative z-10 w-full max-w-md mx-4 p-8 rounded-3xl bg-[#10131d]/85 border border-[#232838] shadow-[0_25px_60px_-15px_rgba(0,0,0,0.8)] backdrop-blur-2xl flex flex-col items-center text-center">
        {/* Glowing Brand Icon Capsule */}
        <div className="relative mb-6">
          <div className="w-16 h-16 rounded-2xl bg-gradient-to-tr from-indigo-600 to-indigo-400 flex items-center justify-center shadow-[0_0_30px_rgba(99,102,241,0.5)]">
            <Cpu size={32} weight="duotone" className="text-white" />
          </div>
          <div className="absolute -inset-1 rounded-2xl bg-indigo-500/20 blur-md -z-10 animate-pulse" />
        </div>

        {/* Title & Tagline */}
        <h1 className="text-xl font-bold tracking-tight text-[#f8fafc] mb-1 flex items-center gap-2">
          <span>GraphFinder</span>
          <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-indigo-500/10 border border-indigo-500/25 text-indigo-400">
            v0.0.1
          </span>
        </h1>
        <p className="text-xs text-[#8b9bb4] mb-6">
          Neural Knowledge Graph &amp; Semantic Vault
        </p>

        {/* Dynamic Status Display */}
        <div className="w-full p-4 rounded-2xl bg-[#0a0c12]/90 border border-[#1f2434] mb-6">
          <div className="flex items-center justify-between text-xs mb-2.5">
            <span className="flex items-center gap-2 text-indigo-300 font-medium">
              <Sparkle size={14} className="text-indigo-400 animate-spin" />
              <span>{isError ? 'Initialization Error' : 'Initializing Environment'}</span>
            </span>
            <span className="font-mono text-[10px] text-emerald-400 font-semibold">
              OFFLINE / LOCAL
            </span>
          </div>

          {/* Progress Shimmer Bar */}
          <div className="w-full h-1.5 rounded-full bg-[#161a26] overflow-hidden relative">
            <div className="absolute top-0 bottom-0 left-0 right-0 bg-gradient-to-r from-indigo-500 via-purple-500 to-emerald-400 animate-pulse" />
          </div>

          <p className="text-[11.5px] text-[#94a3b8] mt-2.5 leading-relaxed text-left">
            {statusMessage}
          </p>
        </div>

        {/* Architecture Badges & Privacy Guarantee */}
        <div className="grid grid-cols-2 gap-2 w-full text-left text-[11px] text-[#64748b]">
          <div className="flex items-center gap-1.5 p-2 rounded-xl bg-[#141824]/60 border border-[#1e2333]">
            <ShieldCheck size={14} className="text-emerald-400 shrink-0" />
            <span className="truncate">100% Local-First</span>
          </div>
          <div className="flex items-center gap-1.5 p-2 rounded-xl bg-[#141824]/60 border border-[#1e2333]">
            <HardDrives size={14} className="text-indigo-400 shrink-0" />
            <span className="truncate">FastEmbed ONNX</span>
          </div>
        </div>

        <div className="mt-5 text-[10.5px] text-[#475569]">
          Model download only occurs on initial launch.
        </div>
      </div>
    </div>
  );
};
