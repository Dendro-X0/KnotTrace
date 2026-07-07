import { Gauge, Trash2 } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { BenchmarkSnapshot } from "@/types";

const SPEEDTEST_URL = "https://www.speedtest.net/";

interface BenchmarkPanelProps {
  snapshots: BenchmarkSnapshot[];
  saving?: boolean;
  error?: string | null;
  onSave: (input: {
    label: string;
    downloadMbps?: number;
    uploadMbps?: number;
    pingMs?: number;
    notes?: string;
  }) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

function formatMbps(value: number | null | undefined) {
  return value == null ? "—" : `${value.toFixed(0)} Mbps`;
}

function formatShape(value: BenchmarkSnapshot["slowdown_shape"]) {
  return value ? value.replace(/_/g, " ") : "general";
}

function compareSnapshots(latest: BenchmarkSnapshot, previous: BenchmarkSnapshot) {
  const lines: string[] = [];
  const latestDown = latest.external_speedtest?.download_mbps;
  const previousDown = previous.external_speedtest?.download_mbps;
  if (latestDown != null && previousDown != null) {
    const delta = latestDown - previousDown;
    lines.push(`Download: ${delta >= 0 ? "+" : ""}${delta.toFixed(0)} Mbps`);
  }

  const latestDns = latest.probe_summary.dns_latency_ms;
  const previousDns = previous.probe_summary.dns_latency_ms;
  if (latestDns != null && previousDns != null) {
    const delta = latestDns - previousDns;
    lines.push(`DNS latency: ${delta >= 0 ? "+" : ""}${delta.toFixed(0)} ms`);
  }

  const latestInternet = latest.probe_summary.internet_latency_ms;
  const previousInternet = previous.probe_summary.internet_latency_ms;
  if (latestInternet != null && previousInternet != null) {
    const delta = latestInternet - previousInternet;
    lines.push(`Internet RTT: ${delta >= 0 ? "+" : ""}${delta.toFixed(0)} ms`);
  }

  return lines;
}

export function BenchmarkPanel({
  snapshots,
  saving = false,
  error,
  onSave,
  onDelete,
}: BenchmarkPanelProps) {
  const [label, setLabel] = useState("baseline");
  const [downloadMbps, setDownloadMbps] = useState("");
  const [uploadMbps, setUploadMbps] = useState("");
  const [pingMs, setPingMs] = useState("");
  const [notes, setNotes] = useState("");

  const latest = snapshots[0];
  const previous = snapshots[1];
  const comparison = latest && previous ? compareSnapshots(latest, previous) : [];

  const handleSave = () => {
    void onSave({
      label,
      downloadMbps: downloadMbps ? Number(downloadMbps) : undefined,
      uploadMbps: uploadMbps ? Number(uploadMbps) : undefined,
      pingMs: pingMs ? Number(pingMs) : undefined,
      notes: notes.trim() || undefined,
    });
  };

  return (
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="text-base">Benchmark snapshots</CardTitle>
          <a
            href={SPEEDTEST_URL}
            target="_blank"
            rel="noreferrer"
            className="text-primary text-xs underline-offset-4 hover:underline"
          >
            Open Speedtest.net
          </a>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-muted-foreground text-xs leading-relaxed">
          Save app probe metrics plus optional Speedtest results before and after DNS or Connect
          changes.
        </p>

        <div className="grid gap-2 sm:grid-cols-2">
          <div className="space-y-1">
            <Label htmlFor="benchmark-label" className="text-xs">
              Label
            </Label>
            <Input
              id="benchmark-label"
              value={label}
              onChange={(event) => setLabel(event.target.value)}
              placeholder="baseline"
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="benchmark-download" className="text-xs">
              Download Mbps (optional)
            </Label>
            <Input
              id="benchmark-download"
              value={downloadMbps}
              onChange={(event) => setDownloadMbps(event.target.value)}
              inputMode="decimal"
              placeholder="e.g. 280"
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="benchmark-upload" className="text-xs">
              Upload Mbps (optional)
            </Label>
            <Input
              id="benchmark-upload"
              value={uploadMbps}
              onChange={(event) => setUploadMbps(event.target.value)}
              inputMode="decimal"
              placeholder="e.g. 40"
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="benchmark-ping" className="text-xs">
              Ping ms (optional)
            </Label>
            <Input
              id="benchmark-ping"
              value={pingMs}
              onChange={(event) => setPingMs(event.target.value)}
              inputMode="decimal"
              placeholder="e.g. 12"
            />
          </div>
        </div>

        <div className="space-y-1">
          <Label htmlFor="benchmark-notes" className="text-xs">
            Notes (optional)
          </Label>
          <Input
            id="benchmark-notes"
            value={notes}
            onChange={(event) => setNotes(event.target.value)}
            placeholder="VPN on, after DNS assist..."
          />
        </div>

        {error && <p className="text-rose-600 dark:text-rose-300 text-xs">{error}</p>}

        <Button size="sm" onClick={handleSave} disabled={saving || !label.trim()}>
          <Gauge className="mr-1 size-3.5" />
          {saving ? "Saving..." : "Save snapshot"}
        </Button>

        {comparison.length > 0 && (
          <div className="rounded-lg border border-border/70 bg-background/40 p-2.5 text-xs">
            <p className="mb-1 font-medium">
              Latest vs previous ({latest.label} vs {previous.label})
            </p>
            <ul className="text-muted-foreground space-y-1">
              {comparison.map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          </div>
        )}

        {snapshots.length > 0 && (
          <ScrollArea className="max-h-40">
            <ul className="grid gap-2">
              {snapshots.map((snapshot) => (
                <li
                  key={snapshot.id}
                  className="flex items-start justify-between gap-2 rounded-lg border border-border/70 bg-background/40 p-2.5 text-xs"
                >
                  <div>
                    <p className="font-medium">{snapshot.label}</p>
                    <p className="text-muted-foreground">
                      Score {snapshot.health_score} · DNS{" "}
                      {snapshot.probe_summary.dns_latency_ms?.toFixed(0) ?? "—"} ms · Down{" "}
                      {formatMbps(snapshot.external_speedtest?.download_mbps)}
                    </p>
                    <p className="text-muted-foreground">
                      Pattern: {formatShape(snapshot.slowdown_shape)}
                    </p>
                    <p className="text-muted-foreground">
                      {new Date(snapshot.timestamp).toLocaleString()}
                    </p>
                  </div>
                  <Button
                    size="sm"
                    variant="secondary"
                    className="size-7 shrink-0 px-2"
                    onClick={() => void onDelete(snapshot.id)}
                    aria-label={`Delete ${snapshot.label}`}
                  >
                    <Trash2 className="size-3.5" />
                  </Button>
                </li>
              ))}
            </ul>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  );
}
