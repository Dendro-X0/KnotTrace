import type { ComponentType } from "react";
import { Activity, Globe, LayoutDashboard, Network, Settings, Shield } from "lucide-react";

import { GlobalSearchTrigger } from "@/components/GlobalSearch";
import { Separator } from "@/components/ui/separator";
import { useLocale } from "@/hooks/useLocale";
import { cn } from "@/lib/utils";
import type { PageId } from "@/types";

const NAV_IDS: Array<{
  id: PageId;
  labelKey: string;
  hintKey: string;
  icon: ComponentType<{ className?: string }>;
}> = [
  { id: "overview", labelKey: "nav.overview", hintKey: "nav.hintOverview", icon: LayoutDashboard },
  { id: "dns", labelKey: "nav.dns", hintKey: "nav.hintDns", icon: Globe },
  { id: "connect", labelKey: "nav.connect", hintKey: "nav.hintConnect", icon: Activity },
  { id: "protect", labelKey: "nav.protect", hintKey: "nav.hintProtect", icon: Shield },
  { id: "network", labelKey: "nav.network", hintKey: "nav.hintNetwork", icon: Network },
];

const SETTINGS_META = {
  id: "settings" as const,
  labelKey: "nav.settings",
  hintKey: "nav.hintSettings",
  icon: Settings,
};

interface SidebarProps {
  page: PageId;
  onNavigate: (page: PageId) => void;
  onOpenSearch: () => void;
  version?: string | null;
}

function NavButton({
  id,
  label,
  hint,
  icon: Icon,
  active,
  compact,
  onNavigate,
}: {
  id: PageId;
  label: string;
  hint: string;
  icon: ComponentType<{ className?: string }>;
  active: boolean;
  compact: boolean;
  onNavigate: (page: PageId) => void;
}) {
  return (
    <button
      type="button"
      className={cn(
        "hub-nav-item font-medium",
        compact
          ? "flex min-w-[4.75rem] shrink-0 snap-start flex-col items-center justify-center gap-1 rounded-lg px-2.5 py-2 text-center"
          : "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left",
        active
          ? "hub-rail-active bg-primary text-primary-foreground"
          : compact
            ? "text-muted-foreground hover:bg-muted/70 hover:text-foreground"
            : "text-foreground/90 hover:bg-muted/65 hover:shadow-[inset_2px_0_0_0] hover:shadow-primary/50",
      )}
      aria-current={active ? "page" : undefined}
      aria-label={compact ? `${label}: ${hint}` : undefined}
      title={compact ? undefined : hint}
      onClick={() => onNavigate(id)}
    >
      {compact ? (
        <>
          <Icon className={cn("size-4", active ? "opacity-100" : "opacity-75")} />
          <span className="max-w-[4.75rem] truncate text-[0.68rem] leading-tight">{label}</span>
        </>
      ) : (
        <>
          <Icon className={cn("size-[1.125rem] shrink-0", active ? "opacity-100" : "opacity-80")} />
          <span className="min-w-0 flex-1 truncate text-sm leading-none">{label}</span>
        </>
      )}
    </button>
  );
}

export function Sidebar({ page, onNavigate, onOpenSearch, version }: SidebarProps) {
  const { t } = useLocale();

  return (
    <aside className="text-sidebar-foreground flex shrink-0 flex-col gap-2 overflow-x-hidden border-b border-sidebar-border/70 py-2 sm:gap-3 sm:py-3 lg:min-h-0 lg:w-[220px] lg:border-r lg:border-b-0 lg:py-3">
      <div className="px-3 lg:hidden">
        <div className="flex items-center gap-2.5">
          <img
            src="/knottrace-icon.png"
            alt=""
            className="size-9 shrink-0 rounded-md shadow-[0_0_16px_-4px] shadow-primary/35 sm:size-10"
          />
          <div className="min-w-0">
            <p className="text-muted-foreground truncate text-[0.62rem] tracking-[0.08em] uppercase sm:text-[0.68rem]">
              {t("brand.name")}
            </p>
            <h1 className="truncate text-sm font-semibold leading-tight">{t("brand.tagline")}</h1>
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
        {NAV_IDS.map((item) => (
          <NavButton
            key={item.id}
            id={item.id}
            label={t(item.labelKey)}
            hint={t(item.hintKey)}
            icon={item.icon}
            active={page === item.id}
            compact
            onNavigate={onNavigate}
          />
        ))}
        <NavButton
          id={SETTINGS_META.id}
          label={t(SETTINGS_META.labelKey)}
          hint={t(SETTINGS_META.hintKey)}
          icon={SETTINGS_META.icon}
          active={page === "settings"}
          compact
          onNavigate={onNavigate}
        />
      </nav>

      <div className="hidden px-3 lg:block" aria-hidden>
        <Separator />
      </div>

      <nav
        className="hidden min-h-0 flex-1 flex-col gap-0.5 px-2 lg:flex"
        aria-label="Main navigation"
      >
        {NAV_IDS.map((item) => (
          <NavButton
            key={item.id}
            id={item.id}
            label={t(item.labelKey)}
            hint={t(item.hintKey)}
            icon={item.icon}
            active={page === item.id}
            compact={false}
            onNavigate={onNavigate}
          />
        ))}
      </nav>

      <div className="hidden px-3 lg:block" aria-hidden>
        <Separator />
      </div>

      <div className="hidden space-y-2 px-2 lg:block">
        <NavButton
          id={SETTINGS_META.id}
          label={t(SETTINGS_META.labelKey)}
          hint={t(SETTINGS_META.hintKey)}
          icon={SETTINGS_META.icon}
          active={page === "settings"}
          compact={false}
          onNavigate={onNavigate}
        />
        <p className="text-muted-foreground px-1 text-[0.68rem] leading-snug">
          {version ? `${t("common.version", { version })} · ` : ""}
          {t("brand.trayNote")}
        </p>
      </div>

      <p className="text-muted-foreground px-3 text-center text-[0.62rem] leading-snug lg:hidden">
        {version ? `${t("common.version", { version })} · ` : ""}
        <span className="hidden sm:inline">{t("brand.trayNote")}</span>
        <span className="sm:hidden">{t("brand.trayNoteShort")}</span>
      </p>
    </aside>
  );
}
