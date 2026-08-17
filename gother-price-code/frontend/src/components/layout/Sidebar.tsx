import { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { ChevronDown, ChevronRight } from 'lucide-react';
import { cn } from '@/utils';
import { listHotelGroups } from '@/api/hotelGroups';

interface NavItem {
  label: string;
  to: string;
  badge?: string;
  badgeVariant?: 'red' | 'amber';
  /** Only the canonical entry for a route should show the "you are here"
   * highlight — secondary shortcuts that happen to route to the same
   * place shouldn't also light up, or every item pointing at the same
   * route would highlight together. */
  highlight?: boolean;
}

function NavLink({ item, isActive }: { item: NavItem; isActive: boolean }) {
  const content = (
    <>
      <span>{item.label}</span>
      {item.badge && (
        <span
          className={cn(
            'ml-auto text-[10px] font-semibold rounded-full px-1.5 py-0.5',
            item.badgeVariant === 'amber' ? 'bg-amber-500 text-white' : 'bg-red-500 text-white'
          )}
        >
          {item.badge}
        </span>
      )}
    </>
  );

  return (
    <Link
      to={item.to}
      className={cn(
        'flex items-center gap-2 px-2 py-1.5 rounded-[7px] text-[13px] text-white/80 hover:bg-sidebar-hover hover:text-white transition-colors',
        isActive && 'bg-brand-600 text-white hover:bg-brand-600'
      )}
    >
      {content}
    </Link>
  );
}

export function Sidebar() {
  const location = useLocation();
  const [hotelsExpanded, setHotelsExpanded] = useState(true);
  const { data: groups } = useQuery({
    queryKey: ['hotelGroups'],
    queryFn: listHotelGroups,
  });

  const groupCount = groups?.length;

  // Analytics is a single entry, not a sub-menu: its three former children
  // (Market Overview / Parity Violations / Win Rate) all pointed at this
  // same page, so none of them ever navigated anywhere different.
  // Per-group analytics is reached from the group page instead.
  const hotelsItems: NavItem[] = [
    {
      label: 'New Price Search',
      to: '/',
      highlight: true,
      badge: groupCount != null ? String(groupCount) : undefined,
      badgeVariant: 'red',
    },
    { label: 'All Hotels', to: '/hotels', highlight: true },
    { label: 'Analytics', to: '/analytics', highlight: true },
  ];

  return (
    <aside className="w-[236px] shrink-0 bg-sidebar text-sidebar-foreground flex flex-col overflow-y-auto">
      <div className="px-3 py-3">
        <button
          type="button"
          onClick={() => setHotelsExpanded((e) => !e)}
          className="w-full flex items-center gap-2 px-2 py-2 rounded-[7px] bg-brand-600/90 text-white text-[13px] font-semibold"
        >
          <span>🏨 Hotels</span>
          {hotelsExpanded ? (
            <ChevronDown className="ml-auto h-3.5 w-3.5 text-white/70" />
          ) : (
            <ChevronRight className="ml-auto h-3.5 w-3.5 text-white/70" />
          )}
        </button>

        {hotelsExpanded && (
          <div className="mt-2">
            {hotelsItems.map((item) => (
              <NavLink
                key={item.label}
                item={item}
                isActive={!!(item.highlight && location.pathname === item.to)}
              />
            ))}
          </div>
        )}

        <div className="mt-4 pt-3 border-t border-white/10 space-y-1">
          <div className="flex items-center gap-2 px-2 py-1.5 rounded-[7px] text-[13px] text-white/40 opacity-40 pointer-events-none select-none">
            🎭 Experiences
            <span className="ml-auto text-[9px] rounded-full px-1.5 py-0.5 bg-white/10 text-white/60">Phase 2</span>
          </div>
          <div className="flex items-center gap-2 px-2 py-1.5 rounded-[7px] text-[13px] text-white/40 opacity-40 pointer-events-none select-none">
            ✈️ Flights
            <span className="ml-auto text-[9px] rounded-full px-1.5 py-0.5 bg-white/10 text-white/60">Phase 3</span>
          </div>
        </div>
      </div>

      <div className="mt-auto px-3 py-3 border-t border-white/10 text-[12px] text-white/50 space-y-1">
        <div className="px-2 py-1">⚙️ Settings</div>
        <div className="px-2 py-1">❓ Help &amp; Docs</div>
      </div>
    </aside>
  );
}
