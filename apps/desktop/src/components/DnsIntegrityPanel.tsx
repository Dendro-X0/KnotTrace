import { AlertTriangle } from "lucide-react";

import { DnsIntegrityBadge } from "@/components/DnsIntegrityBadge";
import { EmptyState } from "@/components/EmptyState";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { DnsIntegrityStatus } from "@/types";

interface DnsIntegrityPanelProps {
  integrity?: DnsIntegrityStatus | null;
}

export function DnsIntegrityPanel({ integrity }: DnsIntegrityPanelProps) {
  if (!integrity) {
    return (
      <EmptyState
        icon={AlertTriangle}
        title="DNS integrity not checked yet"
        description="Run a health check to compare your resolver answers against trusted public DNS."
      />
    );
  }

  return (
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="text-base">DNS integrity</CardTitle>
          <DnsIntegrityBadge integrity={integrity} />
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm leading-relaxed">{integrity.summary}</p>
        <div className="flex flex-wrap gap-2 text-xs">
          <Badge variant="muted">Checked {integrity.checked_domains} domains</Badge>
          <Badge variant="muted">Confidence {integrity.confidence}</Badge>
          {integrity.mismatch_count > 0 && (
            <Badge variant="caution">{integrity.mismatch_count} mismatch(es)</Badge>
          )}
        </div>

        {integrity.details.length > 0 ? (
          <ScrollArea className="max-h-44">
            <ul className="grid gap-2">
              {integrity.details.map((finding) => (
                <li
                  key={finding.domain}
                  className="rounded-lg border border-border/70 bg-background/40 p-2.5 text-sm"
                >
                  <p className="font-medium">{finding.domain}</p>
                  <p className="text-muted-foreground mt-1 text-xs">{finding.reason}</p>
                  {finding.local_answers.length > 0 && (
                    <p className="text-muted-foreground mt-1 text-xs">
                      Local: {finding.local_answers.join(", ")}
                    </p>
                  )}
                  {finding.trusted_answers.length > 0 && (
                    <p className="text-muted-foreground text-xs">
                      Trusted: {finding.trusted_answers.join(", ")}
                    </p>
                  )}
                  {finding.local_error && (
                    <p className="text-rose-600 dark:text-rose-300 mt-1 text-xs">
                      Local error: {finding.local_error}
                    </p>
                  )}
                </li>
              ))}
            </ul>
          </ScrollArea>
        ) : (
          <p className="text-muted-foreground text-xs">
            Local resolver answers matched trusted public DNS for all checked domains.
          </p>
        )}
      </CardContent>
    </Card>
  );
}
