import { useMemo, useState, type ReactNode } from "react";
import {
  ArrowRight,
  Info,
  Monitor,
  Palette,
  PlugZap,
  Search,
  Shield,
  Sparkles,
} from "lucide-react";

import { DensityToggle } from "@/components/DensityToggle";
import { LanguageToggle } from "@/components/LanguageToggle";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { useLocale } from "@/hooks/useLocale";
import { useTheme } from "@/hooks/useTheme";
import { useUiDensity } from "@/hooks/useUiDensity";
import type { CompanionState } from "@/hooks/useCompanion";
import {
  applyReduceMotion,
  getStoredReduceMotion,
  storeReduceMotion,
} from "@/lib/motionPref";
import { cn } from "@/lib/utils";

interface SettingsPageProps {
  state: CompanionState;
}

type SettingsCategoryId = "appearance" | "monitoring" | "related" | "about";

interface SettingsCategory {
  id: SettingsCategoryId;
  groupKey: string;
  labelKey: string;
  keywords: string[];
  icon: typeof Palette;
}

const CATEGORIES: SettingsCategory[] = [
  {
    id: "appearance",
    groupKey: "settings.groupGeneral",
    labelKey: "settings.appearanceTitle",
    keywords: ["theme", "dark", "light", "language", "density", "quick", "motion", "外观", "主题", "语言"],
    icon: Palette,
  },
  {
    id: "monitoring",
    groupKey: "settings.groupGeneral",
    labelKey: "settings.monitoringTitle",
    keywords: ["monitor", "tray", "background", "poll", "监视", "后台"],
    icon: Monitor,
  },
  {
    id: "related",
    groupKey: "settings.groupTools",
    labelKey: "settings.relatedTitle",
    keywords: ["protect", "connect", "dns", "proxy", "防护", "连接"],
    icon: Shield,
  },
  {
    id: "about",
    groupKey: "settings.groupAbout",
    labelKey: "settings.aboutTitle",
    keywords: ["version", "update", "about", "release", "版本", "更新"],
    icon: Info,
  },
];

function SettingCard({
  title,
  current,
  onReset,
  resetDisabled,
  children,
}: {
  title: string;
  current: string;
  onReset?: () => void;
  resetDisabled?: boolean;
  children: ReactNode;
}) {
  const { t } = useLocale();
  return (
    <section className="surface-card space-y-3 rounded-2xl border border-border/60 p-4 sm:p-5">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <h3 className="text-sm font-semibold tracking-tight">{title}</h3>
          <p className="text-muted-foreground text-[0.65rem] font-medium tracking-[0.12em] uppercase">
            {t("settings.current", { value: current })}
          </p>
        </div>
        {onReset && (
          <Button
            type="button"
            size="sm"
            variant="secondary"
            className="h-7 shrink-0 text-[0.65rem] tracking-wide uppercase"
            disabled={resetDisabled}
            onClick={onReset}
          >
            {t("settings.reset")}
          </Button>
        )}
      </div>
      {children}
    </section>
  );
}

function LinkRow({
  label,
  hint,
  onClick,
}: {
  label: string;
  hint: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="hub-nav-item hover:border-primary/30 hover:bg-muted/40 flex w-full items-center justify-between gap-3 rounded-xl border border-border/50 bg-muted/10 px-3.5 py-3 text-left"
    >
      <span className="min-w-0">
        <span className="block text-sm font-medium">{label}</span>
        <span className="text-muted-foreground block text-xs leading-relaxed">{hint}</span>
      </span>
      <ArrowRight className="text-muted-foreground size-4 shrink-0" />
    </button>
  );
}

export function SettingsPage({ state }: SettingsPageProps) {
  const { t, locale, setLocale, options } = useLocale();
  const { mode: themeMode, setMode: setThemeMode } = useTheme();
  const { mode: densityMode, setMode: setDensityMode } = useUiDensity();
  const [category, setCategory] = useState<SettingsCategoryId>("appearance");
  const [query, setQuery] = useState("");
  const [reduceMotion, setReduceMotion] = useState(() => getStoredReduceMotion());

  const monitorActive = state.monitorStatus?.enabled ?? true;
  const pollSecs = state.monitorStatus?.poll_interval_secs;

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return CATEGORIES;
    return CATEGORIES.filter((item) => {
      const label = t(item.labelKey).toLowerCase();
      return label.includes(q) || item.keywords.some((kw) => kw.toLowerCase().includes(q) || q.includes(kw.toLowerCase()));
    });
  }, [query, t]);

  const active = filtered.find((item) => item.id === category) ?? filtered[0] ?? null;
  const activeId = active?.id ?? null;

  const groups = useMemo(() => {
    const order = ["settings.groupGeneral", "settings.groupTools", "settings.groupAbout"];
    return order
      .map((groupKey) => ({
        groupKey,
        items: filtered.filter((item) => item.groupKey === groupKey),
      }))
      .filter((group) => group.items.length > 0);
  }, [filtered]);

  const themeCurrent =
    themeMode === "system"
      ? t("theme.system")
      : themeMode === "dark"
        ? t("theme.dark")
        : t("theme.light");
  const densityCurrent = densityMode === "quick" ? t("density.quick") : t("density.advanced");
  const languageCurrent =
    options.find((option) => option.code === locale)?.nativeName ?? locale.toUpperCase();

  return (
    <div className="flex min-h-0 flex-col gap-3 lg:h-full lg:flex-row lg:gap-4">
      <aside className="flex shrink-0 flex-col gap-2 lg:w-[220px]">
        <div className="relative">
          <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2" />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("settings.searchPlaceholder")}
            className="h-9 border-border/60 bg-muted/20 pl-8 text-xs"
            aria-label={t("settings.searchPlaceholder")}
          />
        </div>

        <nav
          className="flex gap-1 overflow-x-auto pb-0.5 lg:hidden [-ms-overflow-style:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
          aria-label={t("settings.categories")}
        >
          {filtered.map((item) => {
            const Icon = item.icon;
            const selected = item.id === activeId;
            return (
              <button
                key={item.id}
                type="button"
                className={cn(
                  "hub-nav-item flex shrink-0 items-center gap-1.5 rounded-lg px-2.5 py-2 text-xs font-medium",
                  selected
                    ? "settings-hub-nav-active bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-muted/60 hover:text-foreground",
                )}
                aria-current={selected ? "page" : undefined}
                onClick={() => setCategory(item.id)}
              >
                <Icon className="size-3.5" />
                {t(item.labelKey)}
              </button>
            );
          })}
        </nav>

        <div className="hidden min-h-0 flex-1 flex-col gap-3 lg:flex">
          {groups.map((group) => (
            <div key={group.groupKey} className="space-y-1">
              <p className="text-muted-foreground px-2 text-[0.62rem] font-medium tracking-[0.14em] uppercase">
                {t(group.groupKey)}
              </p>
              <div className="grid gap-0.5">
                {group.items.map((item) => {
                  const Icon = item.icon;
                  const selected = item.id === activeId;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      className={cn(
                        "hub-nav-item flex w-full items-center gap-2.5 rounded-xl px-2.5 py-2.5 text-left text-sm font-medium",
                        selected
                          ? "settings-hub-nav-active bg-primary text-primary-foreground"
                          : "text-foreground/90 hover:bg-muted/55",
                      )}
                      aria-current={selected ? "page" : undefined}
                      onClick={() => setCategory(item.id)}
                    >
                      <Icon className={cn("size-4 shrink-0", selected ? "opacity-100" : "opacity-75")} />
                      <span className="truncate">{t(item.labelKey)}</span>
                    </button>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      </aside>

      <div className="hub-stage motion-page-enter min-h-0 flex-1 space-y-4 overflow-y-auto rounded-2xl border border-border/50 p-3 sm:p-5">
        {!active && (
          <p className="text-muted-foreground text-sm">{t("settings.searchEmpty")}</p>
        )}

        {active && (
          <header className="space-y-1">
            <div className="flex items-center gap-2">
              <Sparkles className="text-primary size-4" />
              <h2 className="text-lg font-semibold tracking-tight">{t(active.labelKey)}</h2>
            </div>
            <p className="text-muted-foreground max-w-2xl text-sm leading-relaxed">
              {activeId === "appearance" && t("settings.appearanceDescription")}
              {activeId === "monitoring" && t("settings.monitoringDescription")}
              {activeId === "related" && t("settings.relatedDescription")}
              {activeId === "about" && t("settings.aboutDescription")}
            </p>
          </header>
        )}

        {activeId === "appearance" && (
          <div className="grid gap-3">
            <SettingCard
              title={t("language.label")}
              current={languageCurrent}
              resetDisabled={locale === "en"}
              onReset={() => setLocale("en")}
            >
              <LanguageToggle />
              <p className="text-muted-foreground text-[0.68rem] leading-relaxed">
                {t("language.help")}
              </p>
            </SettingCard>

            <SettingCard
              title={t("theme.label")}
              current={themeCurrent}
              resetDisabled={themeMode === "system"}
              onReset={() => setThemeMode("system")}
            >
              <ThemeToggle />
            </SettingCard>

            <SettingCard
              title={t("density.label")}
              current={densityCurrent}
              resetDisabled={densityMode === "quick"}
              onReset={() => setDensityMode("quick")}
            >
              <DensityToggle />
              <p className="text-muted-foreground text-[0.68rem] leading-relaxed">
                {t("density.help")}
              </p>
            </SettingCard>

            <SettingCard
              title={t("settings.motionTitle")}
              current={reduceMotion ? t("settings.motionOn") : t("settings.motionOff")}
              resetDisabled={!reduceMotion}
              onReset={() => {
                setReduceMotion(false);
                storeReduceMotion(false);
                applyReduceMotion(false);
              }}
            >
              <div className="flex items-center justify-between gap-3 rounded-xl border border-border/50 bg-muted/15 px-3 py-2.5">
                <div className="min-w-0">
                  <Label htmlFor="settings-reduce-motion" className="text-sm">
                    {t("settings.motionLabel")}
                  </Label>
                  <p className="text-muted-foreground mt-0.5 text-xs leading-relaxed">
                    {t("settings.motionHelp")}
                  </p>
                </div>
                <Switch
                  id="settings-reduce-motion"
                  checked={reduceMotion}
                  onCheckedChange={(checked) => {
                    setReduceMotion(checked);
                    storeReduceMotion(checked);
                    applyReduceMotion(checked);
                  }}
                />
              </div>
            </SettingCard>
          </div>
        )}

        {activeId === "monitoring" && (
          <SettingCard
            title={t("settings.monitorLabel")}
            current={monitorActive ? t("header.monitorActive") : t("header.monitorPaused")}
          >
            <div className="flex items-center justify-between gap-3 rounded-xl border border-border/50 bg-muted/15 px-3 py-2.5">
              <div className="min-w-0">
                <p className="text-muted-foreground text-xs leading-relaxed">
                  {pollSecs != null
                    ? t("settings.monitorPoll", { secs: pollSecs })
                    : t("settings.monitorHelp")}
                </p>
                {state.monitorError && (
                  <p className="text-destructive mt-1 text-xs">{state.monitorError}</p>
                )}
              </div>
              <Switch
                id="settings-monitor"
                checked={monitorActive}
                onCheckedChange={(checked) => {
                  void state.setMonitorEnabled(checked).catch(() => undefined);
                }}
              />
            </div>
          </SettingCard>
        )}

        {activeId === "related" && (
          <div className="grid gap-2">
            <LinkRow
              label={t("settings.linkProtect")}
              hint={t("settings.linkProtectHint")}
              onClick={() => state.navigate("protect")}
            />
            <LinkRow
              label={t("settings.linkConnect")}
              hint={t("settings.linkConnectHint")}
              onClick={() => state.navigate("connect")}
            />
            <LinkRow
              label={t("settings.linkDns")}
              hint={t("settings.linkDnsHint")}
              onClick={() => state.navigate("dns")}
            />
          </div>
        )}

        {activeId === "about" && (
          <div className="grid gap-3">
            <SettingCard
              title={t("settings.aboutTitle")}
              current={
                state.appVersion
                  ? t("common.version", { version: state.appVersion })
                  : t("common.versionUnknown")
              }
            >
              <div className="space-y-3">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge variant="muted">
                    {state.appVersion
                      ? t("common.version", { version: state.appVersion })
                      : t("common.versionUnknown")}
                  </Badge>
                  <span className="text-muted-foreground text-xs">{t("settings.trayNote")}</span>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    variant="secondary"
                    disabled={state.updateChecking || state.updateInstalling}
                    onClick={() => void state.checkForUpdates()}
                  >
                    {state.updateChecking ? t("settings.checkingUpdates") : t("settings.checkUpdates")}
                  </Button>
                  {state.updateCheck?.available && (
                    <Button
                      size="sm"
                      className="hub-cta"
                      disabled={state.updateInstalling || !state.updateCheck.can_install_in_app}
                      onClick={() => void state.installUpdate()}
                    >
                      {state.updateInstalling
                        ? t("settings.installing")
                        : t("settings.installUpdate", {
                            version: state.updateCheck.latest_version ?? "update",
                          })}
                    </Button>
                  )}
                  {state.updateCheck?.release_url && (
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => void state.openUpdateRelease()}
                    >
                      {t("settings.releaseNotes")}
                    </Button>
                  )}
                </div>
                {state.updateError && <p className="text-destructive text-xs">{state.updateError}</p>}
                {state.updateCheck && !state.updateCheck.available && !state.updateChecking && (
                  <p className="text-muted-foreground text-xs">{t("settings.upToDate")}</p>
                )}
              </div>
            </SettingCard>

            <p className="text-muted-foreground flex items-center gap-1.5 px-1 text-[0.68rem]">
              <PlugZap className="size-3" />
              {t("brand.disclaimer")}
            </p>
          </div>
        )}

      </div>
    </div>
  );
}
