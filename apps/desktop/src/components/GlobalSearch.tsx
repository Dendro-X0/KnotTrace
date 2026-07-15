import { useEffect, useMemo, useRef, useState, type ComponentType } from "react";
import {
  Activity,
  Globe,
  LayoutDashboard,
  Network,
  RefreshCw,
  Search,
  Shield,
} from "lucide-react";

import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import type { PageId } from "@/types";

export interface GlobalSearchAction {
  id: string;
  label: string;
  hint: string;
  keywords: string[];
  icon: ComponentType<{ className?: string }>;
  run: () => void;
}

interface GlobalSearchProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onNavigate: (page: PageId) => void;
  onRunCheck: () => void;
}

function buildActions(
  onNavigate: (page: PageId) => void,
  onRunCheck: () => void,
): GlobalSearchAction[] {
  return [
    {
      id: "overview",
      label: "Overview",
      hint: "Health, trends, and next steps",
      keywords: ["home", "dashboard", "health", "trends"],
      icon: LayoutDashboard,
      run: () => onNavigate("overview"),
    },
    {
      id: "dns",
      label: "DNS Assist",
      hint: "Resolver recommendations and restore",
      keywords: ["dns", "resolver", "assist"],
      icon: Globe,
      run: () => onNavigate("dns"),
    },
    {
      id: "connect",
      label: "Connect",
      hint: "Proxy switching and node comparison",
      keywords: ["connect", "proxy", "clash", "mihomo"],
      icon: Activity,
      run: () => onNavigate("connect"),
    },
    {
      id: "protect",
      label: "Protect",
      hint: "Alerts, Do Not Disturb, quiet hours",
      keywords: ["protect", "dnd", "quiet", "notify", "alerts"],
      icon: Shield,
      run: () => onNavigate("protect"),
    },
    {
      id: "network",
      label: "Network",
      hint: "Environment, probes, and throughput",
      keywords: ["network", "environment", "probe", "throughput"],
      icon: Network,
      run: () => onNavigate("network"),
    },
    {
      id: "run-check",
      label: "Run health check",
      hint: "Start a manual fast check now",
      keywords: ["check", "scan", "refresh", "health"],
      icon: RefreshCw,
      run: () => onRunCheck(),
    },
  ];
}

function matchesQuery(action: GlobalSearchAction, query: string): boolean {
  const haystack = [action.label, action.hint, ...action.keywords]
    .join(" ")
    .toLowerCase();
  return query
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
    .every((token) => haystack.includes(token));
}

export function useGlobalSearchShortcut(onOpen: () => void) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const typing =
        target?.tagName === "INPUT" ||
        target?.tagName === "TEXTAREA" ||
        target?.isContentEditable;

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        onOpen();
        return;
      }

      if (!typing && event.key === "/" && !event.metaKey && !event.ctrlKey && !event.altKey) {
        event.preventDefault();
        onOpen();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onOpen]);
}

export function GlobalSearchTrigger({ onOpen }: { onOpen: () => void }) {
  const isMac =
    typeof navigator !== "undefined" && /Mac|iPhone|iPad/i.test(navigator.platform);

  return (
    <button
      type="button"
      onClick={onOpen}
      className="text-muted-foreground hover:bg-muted/65 hover:text-foreground flex w-full items-center gap-2 rounded-md border border-border/60 bg-muted/20 px-2.5 py-2 text-left text-xs transition-colors"
      aria-label="Open search"
    >
      <Search className="size-3.5 shrink-0 opacity-80" />
      <span className="min-w-0 flex-1 truncate">Search…</span>
      <kbd className="bg-muted/50 text-muted-foreground hidden rounded px-1.5 py-0.5 font-mono text-[0.62rem] sm:inline">
        {isMac ? "⌘K" : "Ctrl+K"}
      </kbd>
    </button>
  );
}

export function GlobalSearch({
  open,
  onOpenChange,
  onNavigate,
  onRunCheck,
}: GlobalSearchProps) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const actions = useMemo(
    () => buildActions(onNavigate, onRunCheck),
    [onNavigate, onRunCheck],
  );

  const filtered = useMemo(() => {
    const trimmed = query.trim();
    if (!trimmed) return actions;
    return actions.filter((action) => matchesQuery(action, trimmed));
  }, [actions, query]);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setActiveIndex(0);
    const id = window.setTimeout(() => inputRef.current?.focus(), 0);
    return () => window.clearTimeout(id);
  }, [open]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onOpenChange(false);
        return;
      }
      if (event.key === "ArrowDown") {
        event.preventDefault();
        setActiveIndex((index) => Math.min(index + 1, Math.max(filtered.length - 1, 0)));
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        setActiveIndex((index) => Math.max(index - 1, 0));
        return;
      }
      if (event.key === "Enter") {
        const action = filtered[activeIndex];
        if (!action) return;
        event.preventDefault();
        action.run();
        onOpenChange(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [activeIndex, filtered, open, onOpenChange]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/45 px-3 pt-[min(18vh,8rem)] backdrop-blur-[2px]"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onOpenChange(false);
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Global search"
        className="surface-panel border-border/70 w-full max-w-md overflow-hidden rounded-xl border shadow-lg"
      >
        <div className="flex items-center gap-2 border-b border-border/60 px-3 py-2.5">
          <Search className="text-muted-foreground size-4 shrink-0" />
          <Input
            ref={inputRef}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search pages and actions…"
            className="border-0 bg-transparent shadow-none focus-visible:ring-0"
            aria-autocomplete="list"
            aria-controls="global-search-results"
          />
          <kbd className="text-muted-foreground rounded border border-border/60 px-1.5 py-0.5 font-mono text-[0.62rem]">
            Esc
          </kbd>
        </div>

        <ul id="global-search-results" className="max-h-72 overflow-y-auto p-1.5" role="listbox">
          {filtered.length === 0 ? (
            <li className="text-muted-foreground px-3 py-6 text-center text-sm">
              No matching pages or actions.
            </li>
          ) : (
            filtered.map((action, index) => {
              const Icon = action.icon;
              const active = index === activeIndex;
              return (
                <li key={action.id} role="option" aria-selected={active}>
                  <button
                    type="button"
                    className={cn(
                      "flex w-full items-center gap-3 rounded-md px-2.5 py-2 text-left transition-colors",
                      active
                        ? "bg-primary text-primary-foreground"
                        : "hover:bg-muted/70 text-foreground",
                    )}
                    onMouseEnter={() => setActiveIndex(index)}
                    onClick={() => {
                      action.run();
                      onOpenChange(false);
                    }}
                  >
                    <Icon className={cn("size-4 shrink-0", active ? "opacity-100" : "opacity-80")} />
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-sm font-medium">{action.label}</span>
                      <span
                        className={cn(
                          "block truncate text-[0.68rem]",
                          active ? "text-primary-foreground/80" : "text-muted-foreground",
                        )}
                      >
                        {action.hint}
                      </span>
                    </span>
                  </button>
                </li>
              );
            })
          )}
        </ul>
      </div>
    </div>
  );
}
