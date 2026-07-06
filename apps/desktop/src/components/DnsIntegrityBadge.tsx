import { ShieldAlert } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { integrityBadgeVariant, integrityLabel } from "@/components/shared";
import type { DnsIntegrityStatus } from "@/types";

interface DnsIntegrityBadgeProps {
  integrity?: DnsIntegrityStatus | null;
  compact?: boolean;
}

export function DnsIntegrityBadge({ integrity, compact = false }: DnsIntegrityBadgeProps) {
  if (!integrity) {
    return (
      <Badge variant="muted" className="gap-1">
        <ShieldAlert className="size-3" />
        DNS integrity pending
      </Badge>
    );
  }

  return (
    <Badge variant={integrityBadgeVariant(integrity.state)} className="gap-1" title={integrity.summary}>
      <ShieldAlert className="size-3" />
      {compact ? integrityLabel(integrity.state) : `DNS ${integrityLabel(integrity.state)}`}
    </Badge>
  );
}
