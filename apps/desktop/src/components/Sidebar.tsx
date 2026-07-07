import type { ComponentType } from "react";
import { Activity, Globe, LayoutDashboard, Network, Shield } from "lucide-react";

import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import type { PageId } from "@/types";

const NAV_ITEMS: Array<{
  id: PageId;
  label: string;
  hint: string;
  icon: ComponentType<{ className?: string }>;
}> = [
  { id: "overview", label: "Overview", hint: "Health and trends", icon: LayoutDashboard },
  { id: "dns", label: "DNS Assist", hint: "Resolver tuning", icon: Globe },
  { id: "connect", label: "Connect", hint: "Proxy switching", icon: Activity },
  { id: "protect", label: "Protect", hint: "Trust and alerts", icon: Shield },
  { id: "network", label: "Network", hint: "Environment and probes", icon: Network },
];

interface SidebarProps {
  page: PageId;
  onNavigate: (page: PageId) => void;
  version?: string | null;
}

function NavButton({
  item,
  active,
  compact,
  onNavigate,
}: {
  item: (typeof NAV_ITEMS)[number];
  active: boolean;
  compact: boolean;
  onNavigate: (page: PageId) => void;
}) {
  const Icon = item.icon;

  return (
    <button
      type="button"
      className={cn(
        "border transition-all",
        compact
          ? "flex min-w-[4.25rem] shrink-0 snap-start flex-col items-center gap-0.5 rounded-lg px-2 py-1.5 text-center"
          : "w-full rounded-lg px-2.5 py-1.5 text-left",
        active
          ? "border-primary/30 bg-primary/10 text-primary shadow-sm"
          : "border-transparent text-foreground hover:border-border/60 hover:bg-muted/50",
      )}
      aria-current={active ? "page" : undefined}
      aria-label={compact ? `${item.label}: ${item.hint}` : undefined}
      onClick={() => onNavigate(item.id)}
    >
      {compact ? (
        <span className="flex flex-col items-center gap-0.5 text-[0.68rem] leading-tight font-medium">
          <Icon className="size-4 opacity-80" />
          <span className="max-w-[4.25rem] truncate">{item.label}</span>
        </span>
      ) : (
        <span className="flex min-w-0 items-center gap-2">
          <Icon className="size-4 shrink-0 opacity-80" />
          <span className="min-w-0 flex-1 truncate text-sm font-medium leading-none">{item.label}</span>
          <span className="text-muted-foreground hidden max-w-[7.5rem] shrink truncate text-[0.68rem] leading-none lg:inline">
            {item.hint}
          </span>
        </span>
      )}
    </button>
  );
}

export function Sidebar({ page, onNavigate, version }: SidebarProps) {
  return (
    <aside className="bg-sidebar text-sidebar-foreground flex shrink-0 flex-col gap-2 border-b border-sidebar-border px-2 py-2 backdrop-blur-xl sm:gap-3 sm:px-3 sm:py-3 lg:min-h-0 lg:border-r lg:border-b-0 lg:py-4">
      <div className="px-1 sm:px-2">
        <div className="flex items-center gap-2 sm:gap-3">
          <img
            src="/knottrace-icon.png"
            alt=""
            className="size-9 shrink-0 rounded-lg sm:size-10"
          />
          <div className="min-w-0">
            <p className="text-muted-foreground truncate text-[0.62rem] tracking-[0.08em] uppercase sm:text-[0.68rem]">
              KnotTrace
            </p>
            <h1 className="truncate text-sm font-semibold">Network companion</h1>
          </div>
        </div>
      </div>

      <nav
        className="flex gap-1.5 overflow-x-auto pb-0.5 [-ms-overflow-style:none] [scrollbar-width:none] lg:hidden [&::-webkit-scrollbar]:hidden"
        aria-label="Main navigation"
      >
        {NAV_ITEMS.map((item) => (
          <NavButton
            key={item.id}
            item={item}
            active={page === item.id}
            compact
            onNavigate={onNavigate}
          />
        ))}
      </nav>

      <Separator className="hidden lg:block" />

      <nav className="hidden min-h-0 flex-1 lg:grid lg:gap-1" aria-label="Main navigation">
        {NAV_ITEMS.map((item) => (
          <NavButton
            key={item.id}
            item={item}
            active={page === item.id}
            compact={false}
            onNavigate={onNavigate}
          />
        ))}
      </nav>

      <Separator className="hidden lg:block" />

      <p className="text-muted-foreground hidden px-2 text-[0.7rem] lg:block">
        {version ? `v${version} · ` : ""}
        Closing the window keeps the app running in the tray.
      </p>

      <p className="text-muted-foreground px-1 text-center text-[0.62rem] leading-snug sm:px-2 lg:hidden">
        {version ? `v${version} · ` : ""}
        <span className="hidden sm:inline">Closing the window keeps the app in the tray.</span>
        <span className="sm:hidden">Runs in the tray when closed.</span>
      </p>
    </aside>
  );
}
