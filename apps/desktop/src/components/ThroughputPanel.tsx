import { ArrowDownUp, Gauge, Route } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { CompanionState } from "@/hooks/useCompanion";
import { cn } from "@/lib/utils";

interface ThroughputPanelProps {
  state: CompanionState;
}

function bytesToMb(bytes: number) {
  return (bytes / 1_000_000).toFixed(1);
}

function mbToBytes(value: string) {
  const parsed = Number.parseFloat(value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }
  return Math.round(parsed * 1_000_000);
}

export function ThroughputPanel({ state }: ThroughputPanelProps) {
  const groups = state.connectRecommendation?.groups ?? [];
  const [downloadMb, setDownloadMb] = useState("5.0");
  const [uploadMb, setUploadMb] = useState("2.0");
  const [selectedGroup, setSelectedGroup] = useState("");

  useEffect(() => {
    if (!selectedGroup && groups.length > 0) {
      setSelectedGroup(groups[0].name);
    }
  }, [groups, selectedGroup]);

  useEffect(() => {
    if (!state.throughputSettings) return;
    setDownloadMb(bytesToMb(state.throughputSettings.download_bytes));
    setUploadMb(bytesToMb(state.throughputSettings.upload_bytes));
  }, [state.throughputSettings]);

  const throughputResult = state.throughputResult;
  const proxyComparison = state.proxyComparison;

  const sortedSamples = useMemo(
    () =>
      [...(proxyComparison?.samples ?? [])].sort(
        (left, right) => (left.delay_ms ?? 9999) - (right.delay_ms ?? 9999),
      ),
    [proxyComparison?.samples],
  );

  const saveSettings = async () => {
    const downloadBytes = mbToBytes(downloadMb);
    const uploadBytes = mbToBytes(uploadMb);
    if (downloadBytes == null || uploadBytes == null) {
      return;
    }
    await state.saveThroughputSettings(downloadBytes, uploadBytes);
  };

  return (
    <Card className="border-border/70 bg-muted/10 xl:col-span-2">
      <CardHeader className="pb-2">
        <CardTitle className="text-base">Throughput & proxy paths</CardTitle>
        <p className="text-muted-foreground text-xs">
          On-demand samples only — not part of background monitoring.
        </p>
      </CardHeader>
      <CardContent className="grid gap-4 lg:grid-cols-2">
        <div className="space-y-3 rounded-lg border border-border/70 bg-background/40 p-3">
          <div className="flex items-center gap-2">
            <Gauge className="text-muted-foreground size-4" />
            <span className="text-sm font-medium">Throughput sample</span>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            <div className="space-y-1">
              <Label htmlFor="download-mb">Download (MB)</Label>
              <Input
                id="download-mb"
                inputMode="decimal"
                value={downloadMb}
                onChange={(event) => setDownloadMb(event.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="upload-mb">Upload (MB)</Label>
              <Input
                id="upload-mb"
                inputMode="decimal"
                value={uploadMb}
                onChange={(event) => setUploadMb(event.target.value)}
              />
            </div>
          </div>

          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant="secondary"
              disabled={state.throughputSaving}
              onClick={() => void saveSettings()}
            >
              {state.throughputSaving ? "Saving..." : "Save sizes"}
            </Button>
            <Button
              size="sm"
              disabled={state.throughputRunning}
              onClick={() => void state.runThroughputTest()}
            >
              {state.throughputRunning ? "Measuring..." : "Run throughput test"}
            </Button>
          </div>

          {state.throughputError && (
            <p className="text-destructive text-xs">{state.throughputError}</p>
          )}

          {throughputResult ? (
            <div className="text-muted-foreground space-y-1 text-xs">
              <p>{throughputResult.summary}</p>
              <p>
                Down{" "}
                {throughputResult.download_mbps != null
                  ? `${throughputResult.download_mbps.toFixed(1)} Mbps`
                  : "failed"}{" "}
                · Up{" "}
                {throughputResult.upload_mbps != null
                  ? `${throughputResult.upload_mbps.toFixed(1)} Mbps`
                  : "failed"}{" "}
                · {throughputResult.duration_ms} ms
              </p>
            </div>
          ) : (
            <p className="text-muted-foreground text-xs">
              Uses Cloudflare speed endpoints with your configured sample sizes (max 20 MB down / 10
              MB up).
            </p>
          )}
        </div>

        <div className="space-y-3 rounded-lg border border-border/70 bg-background/40 p-3">
          <div className="flex items-center gap-2">
            <Route className="text-muted-foreground size-4" />
            <span className="text-sm font-medium">Proxy path comparison</span>
          </div>

          {groups.length === 0 ? (
            <p className="text-muted-foreground text-xs">
              Connect Assist must detect a Mihomo or sing-box API before comparing proxy nodes.
            </p>
          ) : (
            <>
              <div className="space-y-1">
                <Label htmlFor="proxy-group">Proxy group</Label>
                <select
                  id="proxy-group"
                  value={selectedGroup}
                  onChange={(event) => setSelectedGroup(event.target.value)}
                  className={cn(
                    "border-input bg-background ring-offset-background focus-visible:ring-ring flex h-9 w-full rounded-md border px-3 py-1 text-sm shadow-xs focus-visible:ring-2 focus-visible:outline-hidden",
                  )}
                >
                  {groups.map((group) => (
                    <option key={group.name} value={group.name}>
                      {group.name}
                    </option>
                  ))}
                </select>
              </div>

              <Button
                size="sm"
                variant="secondary"
                disabled={!selectedGroup || state.proxyComparing}
                onClick={() => selectedGroup && void state.compareProxyPaths(selectedGroup)}
              >
                <ArrowDownUp className="mr-1 size-3.5" />
                {state.proxyComparing ? "Testing nodes..." : "Compare node delays"}
              </Button>

              {state.proxyComparisonError && (
                <p className="text-destructive text-xs">{state.proxyComparisonError}</p>
              )}

              {proxyComparison && (
                <div className="space-y-2">
                  <p className="text-muted-foreground text-xs">{proxyComparison.summary}</p>
                  <div className="space-y-1">
                    {sortedSamples.map((sample) => (
                      <div
                        key={sample.proxy_name}
                        className="flex items-center justify-between gap-2 text-xs"
                      >
                        <span className="truncate">
                          {sample.proxy_name}
                          {sample.is_current && (
                            <Badge variant="info" className="ml-2">
                              current
                            </Badge>
                          )}
                          {proxyComparison.fastest_proxy === sample.proxy_name && (
                            <Badge variant="active" className="ml-2">
                              fastest
                            </Badge>
                          )}
                        </span>
                        <span className="text-muted-foreground shrink-0">
                          {sample.delay_ms != null ? `${sample.delay_ms} ms` : "timeout"}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
