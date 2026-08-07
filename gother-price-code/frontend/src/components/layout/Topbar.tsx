export function Topbar() {
  return (
    <header className="h-[52px] shrink-0 bg-topbar text-white flex items-center px-4">
      <div className="flex items-baseline gap-2">
        <span className="text-[15px] font-extrabold text-sky-400">⚡ Gother</span>
        <span className="text-[13px] text-white/50">Price Intelligence</span>
      </div>
      <div className="ml-auto flex items-center gap-3 text-[12px] text-white/60">
        <span>v0.1 · Bangkok</span>
        <div className="h-7 w-7 rounded-full bg-slate-700 flex items-center justify-center text-[11px] font-semibold text-white">
          GT
        </div>
      </div>
    </header>
  );
}
