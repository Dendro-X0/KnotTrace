import { Settings2 } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";

interface DnsIntegritySettingsEditorProps {
  domains: string[];
  saving?: boolean;
  error?: string | null;
  onSave: (domains: string[]) => Promise<void>;
}

export function DnsIntegritySettingsEditor({
  domains,
  saving = false,
  error,
  onSave,
}: DnsIntegritySettingsEditorProps) {
  const [draft, setDraft] = useState(domains.join("\n"));

  useEffect(() => {
    setDraft(domains.join("\n"));
  }, [domains]);

  const handleSave = () => {
    const nextDomains = draft
      .split(/[\n,]/)
      .map((domain) => domain.trim())
      .filter(Boolean);
    void onSave(nextDomains);
  };

  return (
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className="pb-2">
        <div className="flex items-center gap-2">
          <Settings2 className="text-muted-foreground size-4" />
          <CardTitle className="text-base">Verification domains</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-muted-foreground text-xs leading-relaxed">
          One domain per line. Integrity checks compare your resolver answers for these domains
          against trusted public DNS (1.1.1.1, 8.8.8.8, 9.9.9.9).
        </p>
        <div className="space-y-2">
          <Label htmlFor="integrity-domains" className="text-xs">
            Domains to verify
          </Label>
          <textarea
            id="integrity-domains"
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            rows={5}
            className="border-input bg-background focus-visible:ring-ring/50 w-full rounded-md border px-3 py-2 text-sm outline-none focus-visible:ring-2"
            placeholder={"example.com\ncloudflare.com"}
          />
        </div>
        {error && <p className="text-rose-600 dark:text-rose-300 text-xs">{error}</p>}
        <Button size="sm" onClick={handleSave} disabled={saving}>
          {saving ? "Saving..." : "Save domains"}
        </Button>
      </CardContent>
    </Card>
  );
}
