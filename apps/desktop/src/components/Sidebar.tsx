import type { ComponentType } from "react";
import { Activity, Globe, LayoutDashboard, Network, Shield } from "lucide-react";

import { GlobalSearchTrigger } from "@/components/GlobalSearch";
import { ThemeToggle } from "@/components/ThemeToggle";
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
  onOpenSearch: () => void;
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
        "font-medium transition-colors",
        compact
          ? "flex min-w-[4.75rem] shrink-0 snap-start flex-col items-center justify-center gap-1 rounded-md px-2.5 py-2 text-center"
          : "flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left",
        active
          ? "bg-primary text-primary-foreground shadow-sm"
          : compact
            ? "text-muted-foreground hover:bg-muted/70 hover:text-foreground"
            : "text-foreground/90 hover:bg-muted/65",
      )}
      aria-current={active ? "page" : undefined}
      aria-label={compact ? `${item.label}: ${item.hint}` : undefined}
      title={compact ? undefined : item.hint}
      onClick={() => onNavigate(item.id)}
    >
      {compact ? (
        <>
          <Icon className={cn("size-4", active ? "opacity-100" : "opacity-75")} />
          <span className="max-w-[4.75rem] truncate text-[0.68rem] leading-tight">{item.label}</span>
        </>
      ) : (
        <>
          <Icon className={cn("size-[1.125rem] shrink-0", active ? "opacity-100" : "opacity-80")} />
          <span className="min-w-0 flex-1 truncate text-sm leading-none">{item.label}</span>
        </>
      )}
    </button>
  );
}

export function Sidebar({ page, onNavigate, onOpenSearch, version }: SidebarProps) {
  return (
    <aside className="surface-panel text-sidebar-foreground flex shrink-0 flex-col gap-2 overflow-x-hidden border-b border-sidebar-border py-2 sm:gap-3 sm:py-3 lg:min-h-0 lg:w-[220px] lg:border-r lg:border-b-0 lg:py-3">
      <div className="px-3">
        <div className="flex items-center gap-2.5">
          <img
            src="/knottrace-icon.png"
            alt=""
            className="size-9 shrink-0 rounded-md sm:size-10"
          />
          <div className="min-w-0">
            <p className="text-muted-foreground truncate text-[0.62rem] tracking-[0.08em] uppercase sm:text-[0.68rem]">
              KnotTrace
            </p>
            <h1 className="truncate text-sm font-semibold leading-tight">Network companion</h1>
          </div>
        </div>
      </div>

      <div className="px-2">
        <GlobalSearchTrigger onOpen={onOpenSearch} />
      </div>

      <nav
        className="flex gap-1 overflow-x-auto px-2 pb-0.5 [-ms-overflow-style:none] [scrollbar-width:none] lg:hidden [&::-webkit-scrollbar]:hidden"
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

      <div className="hidden px-3 lg:block" aria-hidden>
        <Separator />
      </div>

      <nav
        className="hidden min-h-0 flex-1 flex-col gap-0.5 px-2 lg:flex"
        aria-label="Main navigation"
      >
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

      <div className="hidden px-3 lg:block" aria-hidden>
        <Separator />
      </div>

      <div className="hidden px-2 lg:block">
        <ThemeToggle compact />
      </div>

      <p className="text-muted-foreground hidden px-3 text-[0.68rem] leading-snug lg:block">
        {version ? `v${version} · ` : ""}
        Closing the window keeps the app running in the tray.
      </p>

      <div className="px-2 lg:hidden">
        <ThemeToggle compact />
      </div>

      <p className="text-muted-foreground px-3 text-center text-[0.62rem] leading-snug lg:hidden">
        {version ? `v${version} · ` : ""}
        <span className="hidden sm:inline">Closing the window keeps the app in the tray.</span>
        <span className="sm:hidden">Runs in the tray when closed.</span>
      </p>
    </aside>
  );
}
