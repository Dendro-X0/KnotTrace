import type { ReactNode } from "react";
import { ArrowRight, Info, Monitor, Palette, Shield, PlugZap } from "lucide-react";

import { DensityToggle } from "@/components/DensityToggle";
import { LanguageToggle } from "@/components/LanguageToggle";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { useLocale } from "@/hooks/useLocale";
import type { CompanionState } from "@/hooks/useCompanion";

interface SettingsPageProps {
  state: CompanionState;
}

function SettingsSection({
  icon: Icon,
  title,
  description,
  children,
}: {
  icon: typeof Palette;
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <Card className="border-border/60 bg-card/70">
      <CardContent className="space-y-4 py-4 sm:py-5">
        <div className="flex items-start gap-3">
          <div className="bg-muted/80 text-foreground/80 flex size-9 shrink-0 items-center justify-center rounded-xl">
            <Icon className="size-4" />
          </div>
          <div className="min-w-0">
            <h2 className="text-sm font-semibold tracking-tight">{title}</h2>
            <p className="text-muted-foreground mt-0.5 text-xs leading-relaxed">{description}</p>
          </div>
        </div>
        {children}
      </CardContent>
    </Card>
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
      className="hover:bg-muted/40 flex w-full items-center justify-between gap-3 rounded-xl border border-border/50 bg-muted/10 px-3 py-2.5 text-left transition-colors"
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
  const { t } = useLocale();
  const monitorActive = state.monitorStatus?.enabled ?? true;
  const pollSecs = state.monitorStatus?.poll_interval_secs;

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-3">
      <SettingsSection
        icon={Palette}
        title={t("settings.appearanceTitle")}
        description={t("settings.appearanceDescription")}
      >
        <div className="space-y-3">
          <div className="space-y-1.5">
            <Label className="text-xs">{t("theme.label")}</Label>
            <ThemeToggle />
          </div>
          <div className="space-y-1.5">
            <Label className="text-xs">{t("density.label")}</Label>
            <DensityToggle />
            <p className="text-muted-foreground text-[0.68rem] leading-relaxed">
              {t("density.help")}
            </p>
          </div>
          <div className="space-y-1.5">
            <Label className="text-xs">{t("language.label")}</Label>
            <LanguageToggle />
            <p className="text-muted-foreground text-[0.68rem] leading-relaxed">
              {t("language.help")}
            </p>
          </div>
        </div>
      </SettingsSection>

      <SettingsSection
        icon={Monitor}
        title={t("settings.monitoringTitle")}
        description={t("settings.monitoringDescription")}
      >
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border/50 bg-muted/10 px-3 py-2.5">
          <div className="min-w-0">
            <Label htmlFor="settings-monitor" className="text-sm">
              {t("settings.monitorLabel")}
            </Label>
            <p className="text-muted-foreground mt-0.5 text-xs leading-relaxed">
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
      </SettingsSection>

      <SettingsSection
        icon={Shield}
        title={t("settings.relatedTitle")}
        description={t("settings.relatedDescription")}
      >
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
      </SettingsSection>

      <SettingsSection
        icon={Info}
        title={t("settings.aboutTitle")}
        description={t("settings.aboutDescription")}
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
              <Button size="sm" variant="secondary" onClick={() => void state.openUpdateRelease()}>
                {t("settings.releaseNotes")}
              </Button>
            )}
          </div>
          {state.updateError && <p className="text-destructive text-xs">{state.updateError}</p>}
          {state.updateCheck && !state.updateCheck.available && !state.updateChecking && (
            <p className="text-muted-foreground text-xs">{t("settings.upToDate")}</p>
          )}
        </div>
      </SettingsSection>

      <p className="text-muted-foreground flex items-center gap-1.5 px-1 text-[0.68rem]">
        <PlugZap className="size-3" />
        {t("brand.disclaimer")}
      </p>
    </div>
  );
}
