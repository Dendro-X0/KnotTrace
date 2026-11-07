import type { ReactNode } from "react";
import { Network, Radar } from "lucide-react";

import { EmptyState } from "@/components/EmptyState";
import { EnglishProbeNote } from "@/components/EnglishProbeNote";
import { FactList } from "@/components/shared";
import { StabilityProbesPanel } from "@/components/StabilityProbesPanel";
import { LinkFactsPanel } from "@/components/LinkFactsPanel";
import { LocalCapsPanel } from "@/components/LocalCapsPanel";
import { LocalRepairPanel } from "@/components/LocalRepairPanel";
import { MtuAssistPanel } from "@/components/MtuAssistPanel";
import { TunnelComparePanel } from "@/components/TunnelComparePanel";
import { UpstreamPoolPanel } from "@/components/UpstreamPoolPanel";
import { NetworkInsightsPanel } from "@/components/NetworkInsightsPanel";
import { PathBreakPanel } from "@/components/PathBreakPanel";
import { IntegrityBoardPanel } from "@/components/IntegrityBoardPanel";
import { EgressTimelinePanel } from "@/components/EgressTimelinePanel";
import { ProxyPathPanel } from "@/components/ProxyPathPanel";
import { ThroughputPanel } from "@/components/ThroughputPanel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { useLocale } from "@/hooks/useLocale";
import { useUiDensity } from "@/hooks/useUiDensity";
import type { CompanionState } from "@/hooks/useCompanion";
import {
  pathEvidenceFocusFromReport,
  type PathEvidenceFocusId,
} from "@/lib/pathEvidenceFocus";
import { cn } from "@/lib/utils";
import { formatTag } from "@/types";

function PathQuietDetails({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <details className="rounded-xl border border-border/60 bg-muted/10 xl:col-span-2">
      <summary className="text-muted-foreground hover:text-foreground cursor-pointer px-3 py-2.5 text-sm font-medium">
        {title}
      </summary>
      <div className="border-border/50 border-t p-2">{children}</div>
    </details>
  );
}

interface NetworkPageProps {
  state: CompanionState;
}

const ADVANCED_SECTIONS = [
  { id: "network-link", label: "Link & local" },
  { id: "network-path", label: "Path" },
  { id: "network-integrity", label: "Integrity" },
  { id: "network-throughput", label: "Throughput" },
] as const;

function scrollToSection(id: string) {
  document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
}

export function NetworkPage({ state }: NetworkPageProps) {
  const { isQuick } = useUiDensity();
  const { t } = useLocale();
  const env = state.report?.environment;
  const probe = state.report?.probe;
  const score = state.report?.score;
  const loading = state.bootstrapping && !state.report;
  const pathFocus = pathEvidenceFocusFromReport(state.report);

  const probeCard = (
    <Card className="min-h-0 border-border/70 bg-card/80">
      <CardHeader>
        <div className="flex items-center gap-2">
          <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
            <Radar className="size-4" />
          </div>
          <CardTitle>Probe results</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="min-h-0">
        {loading ? (
          <div className="space-y-2">
            {Array.from({ length: 4 }).map((_, index) => (
              <Skeleton key={index} className="h-4 w-full" />
            ))}
          </div>
        ) : probe ? (
          <ScrollArea className="max-h-[28rem]">
            <div>
              <FactList
                rows={[
                  {
                    label: "Internet latency",
                    value: probe.internet
                      ? `${probe.internet.avg_ms.toFixed(0)} ms`
                      : "Unavailable",
                  },
                  {
                    label: "Gateway latency",
                    value: probe.gateway
                      ? `${probe.gateway.avg_ms.toFixed(0)} ms`
                      : "Not measured",
                  },
                  {
                    label: "DNS latency",
                    value:
                      probe.dns.find((sample) => sample.success)?.latency_ms.toFixed(0) ??
                      "Unavailable",
                  },
                  { label: "Probe duration", value: `${probe.duration_ms} ms` },
                ]}
              />
              {!isQuick && score && score.reasons.length > 0 && (
                <ul className="text-muted-foreground mt-3 list-disc space-y-1 pl-4 text-xs">
                  {score.reasons.map((reason) => (
                    <li key={reason}>{reason}</li>
                  ))}
                </ul>
              )}
            </div>
          </ScrollArea>
        ) : (
          <EmptyState
            icon={Radar}
            title="No probe data yet"
            description="Latency and DNS measurements will appear here after your first health check."
            action={
              <Button size="sm" disabled={state.checking} onClick={() => void state.runCheck()}>
                {state.checking ? "Checking…" : "Run health check"}
              </Button>
            }
          />
        )}
      </CardContent>
    </Card>
  );

  const internetMs = probe?.internet?.avg_ms;
  const gatewayMs = probe?.gateway?.avg_ms;
  const dnsMs = probe?.dns.find((sample) => sample.success)?.latency_ms;
  const quickSummary = loading
    ? null
    : [
        internetMs == null ? "Internet unavailable" : `Internet ${internetMs.toFixed(0)} ms`,
        gatewayMs == null ? "Gateway not measured" : `Gateway ${gatewayMs.toFixed(0)} ms`,
        dnsMs == null ? "DNS unavailable" : `DNS ${dnsMs.toFixed(0)} ms`,
      ].join(" · ");

  if (isQuick) {
    return (
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-3 xl:grid-cols-2">
        <Card className="border-border/60 bg-card/70 xl:col-span-2">
          <CardContent className="flex flex-col gap-1 py-4 sm:flex-row sm:items-baseline sm:justify-between sm:gap-4">
            <div>
              <p className="text-sm font-medium">Link snapshot</p>
              <p className="text-muted-foreground text-sm leading-relaxed">
                {quickSummary ?? "Run a health check to measure this link."}
              </p>
            </div>
            {score && (
              <Badge variant={score.grade === "good" ? "good" : score.grade === "fair" ? "fair" : "poor"}>
                {score.grade.toUpperCase()} · {score.score}/100
              </Badge>
            )}
          </CardContent>
        </Card>
        {probeCard}
        <LinkFactsPanel loading={loading} report={state.report?.link_facts} />
        <LocalRepairPanel
          loading={loading}
          report={state.report?.local_repair}
          runningAction={state.localRepairRunning}
          error={state.localRepairError}
          onRun={(action) => void state.runLocalRepair(action)}
        />
        <Card className="border-border/50 bg-card/50 xl:col-span-2">
          <CardContent className="text-muted-foreground py-3 text-xs leading-relaxed">
            Quick mode shows link health and common local assists. Switch to{" "}
            <span className="text-foreground font-medium">Advanced</span> in the sidebar for MTU,
            caps, path break, integrity evidence, and throughput.
          </CardContent>
        </Card>
      </div>
    );
  }

  const environmentCard = (
    <Card className="min-h-0 border-border/70 bg-card/80">
      <CardHeader>
        <div className="flex items-center gap-2">
          <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
            <Network className="size-4" />
          </div>
          <CardTitle>Environment</CardTitle>
        </div>
        <div className="flex flex-wrap justify-end gap-1">
          {(env?.tags ?? []).map((tag) => (
            <Badge key={tag} variant="info">
              {formatTag(tag)}
            </Badge>
          ))}
        </div>
      </CardHeader>
      <CardContent className="min-h-0">
        {loading ? (
          <div className="space-y-2">
            {Array.from({ length: 5 }).map((_, index) => (
              <Skeleton key={index} className="h-4 w-full" />
            ))}
          </div>
        ) : env ? (
          <ScrollArea className="max-h-[28rem]">
            <FactList
              rows={[
                { label: "Host", value: env.hostname },
                { label: "Active interface", value: env.active_interface ?? "Unknown" },
                { label: "Default gateway", value: env.default_gateway ?? "Not detected" },
                {
                  label: "Proxy",
                  value: env.proxy.enabled
                    ? `${env.proxy.server ?? "enabled"} (${env.proxy.source})`
                    : "Off",
                },
                {
                  label: "DNS servers",
                  value:
                    env.dns_servers.map((server) => server.address).join(", ") || "System default",
                },
                {
                  label: "Tor",
                  value: env.tor?.detected
                    ? `${env.tor.socks_endpoint ?? "detected"} (${env.tor.source})${
                        env.tor.socks_reachable ? " · reachable" : " · unreachable"
                      }`
                    : "Not detected",
                },
              ]}
            />
          </ScrollArea>
        ) : (
          <EmptyState
            icon={Network}
            title="No environment snapshot"
            description="Run a health check to capture interface, gateway, proxy, and DNS details."
            action={
              <Button size="sm" disabled={state.checking} onClick={() => void state.runCheck()}>
                {state.checking ? "Checking…" : "Run health check"}
              </Button>
            }
          />
        )}
      </CardContent>
    </Card>
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <nav
        aria-label="Network sections"
        className="bg-background/85 sticky top-0 z-10 -mx-0.5 flex flex-wrap gap-1.5 rounded-xl border border-border/50 px-2 py-2 backdrop-blur-sm"
      >
        {ADVANCED_SECTIONS.map((section) => (
          <button
            key={section.id}
            type="button"
            className={cn(
              "rounded-lg px-2.5 py-1.5 text-xs font-medium transition-colors",
              "text-muted-foreground hover:bg-muted/60 hover:text-foreground",
            )}
            onClick={() => scrollToSection(section.id)}
          >
            {section.label}
          </button>
        ))}
      </nav>

      <div className="flex min-h-0 flex-col gap-5">
        <section id="network-link" className="scroll-mt-14 space-y-2">
          <h2 className="text-muted-foreground px-0.5 text-[0.68rem] font-semibold tracking-wide uppercase">
            Link & local
          </h2>
          <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
            {environmentCard}
            {probeCard}
            <StabilityProbesPanel stability={state.report?.stability} />
            <LinkFactsPanel loading={loading} report={state.report?.link_facts} />
            <LocalRepairPanel
              loading={loading}
              report={state.report?.local_repair}
              runningAction={state.localRepairRunning}
              error={state.localRepairError}
              onRun={(action) => void state.runLocalRepair(action)}
            />
            <LocalCapsPanel
              loading={loading}
              report={state.report?.local_caps}
              state={state.localCapsState}
              applying={state.localCapsApplying}
              restoring={state.localCapsRestoring}
              error={state.localCapsError}
              onApply={() => void state.applyLocalCaps()}
              onRestore={() => void state.restoreLocalCaps()}
            />
            <MtuAssistPanel
              loading={loading}
              report={state.report?.mtu_assist}
              state={state.mtuAssistState}
              applying={state.mtuAssistApplying}
              restoring={state.mtuAssistRestoring}
              error={state.mtuAssistError}
              onApply={() => void state.applyMtuAssist()}
              onRestore={() => void state.restoreMtuAssist()}
            />
          </div>
        </section>

        <section id="network-path" className="scroll-mt-14 space-y-2">
          <h2 className="text-muted-foreground px-0.5 text-[0.68rem] font-semibold tracking-wide uppercase">
            Path
          </h2>
          <Card className="border-border/50 bg-muted/10">
            <CardContent className="space-y-1 py-3">
              <div className="flex flex-wrap items-center gap-2">
                <p className="text-xs font-medium tracking-wide uppercase text-muted-foreground">
                  {t("pathFocus.startHere")}
                </p>
                <EnglishProbeNote />
              </div>
              <p className="text-sm leading-relaxed">{t(pathFocus.reasonKey)}</p>
            </CardContent>
          </Card>
          <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
            {(() => {
              const panels: Array<{
                id: PathEvidenceFocusId | "network_insights";
                title: string;
                node: ReactNode;
              }> = [
                {
                  id: "network_insights",
                  title: t("pathFocus.panel.insights"),
                  node: (
                    <NetworkInsightsPanel
                      loading={loading}
                      egress={state.report?.egress}
                      networkContext={state.report?.network_context}
                      recommendations={state.report?.recommendations}
                    />
                  ),
                },
                {
                  id: "path_break",
                  title: t("pathFocus.panel.pathBreak"),
                  node: (
                    <PathBreakPanel loading={loading} report={state.report?.path_break} />
                  ),
                },
                {
                  id: "proxy_path",
                  title: t("pathFocus.panel.proxyPath"),
                  node: (
                    <ProxyPathPanel
                      loading={loading}
                      proxyEnabled={env?.proxy.enabled}
                      report={state.report?.proxy_path_report}
                    />
                  ),
                },
                {
                  id: "upstream_pool",
                  title: t("pathFocus.panel.upstreamPool"),
                  node: (
                    <UpstreamPoolPanel
                      loading={loading}
                      proxyEnabled={env?.proxy.enabled}
                      proof={state.report?.upstream_pool}
                    />
                  ),
                },
                {
                  id: "tunnel_compare",
                  title: t("pathFocus.panel.tunnelCompare"),
                  node: (
                    <TunnelComparePanel
                      loading={loading}
                      report={state.report?.tunnel_compare}
                    />
                  ),
                },
                {
                  id: "egress_timeline",
                  title: t("pathFocus.panel.egressTimeline"),
                  node: (
                    <EgressTimelinePanel
                      loading={loading}
                      report={state.report?.egress_timeline}
                    />
                  ),
                },
              ];

              const focused = panels.find((panel) => panel.id === pathFocus.id);
              const rest = panels.filter((panel) => panel.id !== pathFocus.id);

              return (
                <>
                  {focused ? (
                    <div className="xl:col-span-2" data-path-focus={focused.id}>
                      {focused.node}
                    </div>
                  ) : null}
                  {rest.map((panel) => (
                    <PathQuietDetails key={panel.id} title={panel.title}>
                      {panel.node}
                    </PathQuietDetails>
                  ))}
                </>
              );
            })()}
          </div>
        </section>

        <section id="network-integrity" className="scroll-mt-14 space-y-2">
          <h2 className="text-muted-foreground px-0.5 text-[0.68rem] font-semibold tracking-wide uppercase">
            Integrity
          </h2>
          <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
            <IntegrityBoardPanel
              loading={loading}
              report={state.report?.integrity}
              checking={state.checking}
              onRunCheck={() => void state.runCheck()}
            />
          </div>
        </section>

        <section id="network-throughput" className="scroll-mt-14 space-y-2">
          <h2 className="text-muted-foreground px-0.5 text-[0.68rem] font-semibold tracking-wide uppercase">
            Throughput
          </h2>
          <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
            <ThroughputPanel state={state} />
          </div>
        </section>
      </div>
    </div>
  );
}
