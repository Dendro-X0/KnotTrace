import type { ComponentType } from "react";
import { Activity, Globe, LayoutDashboard, Network, Shield } from "lucide-react";

import { ScrollArea } from "@/components/ui/scroll-area";
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
}

export function Sidebar({ page, onNavigate }: SidebarProps) {
  return (
    <aside className="bg-sidebar text-sidebar-foreground flex min-h-0 flex-col gap-3 border-b border-sidebar-border px-3 py-3 backdrop-blur-xl lg:border-r lg:border-b-0 lg:px-3 lg:py-4">
      <div className="px-2">
        <div className="flex items-center gap-3">
          <div className="from-primary/20 to-accent text-primary flex size-10 items-center justify-center rounded-xl border border-border/50 bg-gradient-to-br shadow-sm">
            <Activity className="size-5" />
          </div>
          <div>
            <p className="text-muted-foreground text-[0.68rem] tracking-[0.08em] uppercase">
              Network Companion
            </p>
            <h1 className="text-sm font-semibold">Desktop dashboard</h1>
          </div>
        </div>
      </div>

      <Separator className="hidden lg:block" />

      <ScrollArea className="min-h-0 flex-1">
        <nav className="flex gap-2 pb-1 lg:grid lg:gap-1" aria-label="Main navigation">
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon;
            const active = page === item.id;
            return (
              <button
                key={item.id}
                type="button"
                className={cn(
                  "min-w-40 rounded-xl border px-3 py-2.5 text-left transition-all lg:min-w-0",
                  active
                    ? "border-primary/30 bg-primary/10 text-primary shadow-sm"
                    : "border-transparent text-foreground hover:border-border/60 hover:bg-muted/50",
                )}
                aria-current={active ? "page" : undefined}
                onClick={() => onNavigate(item.id)}
              >
                <span className="flex items-center gap-2 text-sm font-medium">
                  <Icon className="size-4 opacity-80" />
                  {item.label}
                </span>
                <span className="text-muted-foreground mt-0.5 block pl-6 text-[0.68rem]">
                  {item.hint}
                </span>
              </button>
            );
          })}
        </nav>
      </ScrollArea>

      <Separator className="hidden lg:block" />

      <p className="text-muted-foreground hidden px-2 text-[0.7rem] lg:block">
        Closing the window keeps the app running in the tray.
      </p>
    </aside>
  );
}
