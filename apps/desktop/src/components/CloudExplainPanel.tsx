import { useCallback, useEffect, useState } from "react";
import { Cloud } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { CompanionState } from "@/hooks/useCompanion";
import type { AiExplainSettings, CloudExplainResult, PageId } from "@/types";

const DATA_SHEET =
  "KnotTrace will send a redacted diagnostic JSON to the AI provider you configured. Nothing is sent until you confirm. KnotTrace does not operate a KnotTrace-hosted model. Hostname stays out; SSID only if you enable it. The API key goes only to your provider. Replies are kept in this session memory only.";

const PROVIDER_PRESETS: Array<{ id: string; label: string; baseUrl: string; model: string }> = [
  {
    id: "openai",
    label: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
  },
  {
    id: "openrouter",
    label: "OpenRouter",
    baseUrl: "https://openrouter.ai/api/v1",
    model: "openai/gpt-4o-mini",
  },
  {
    id: "custom",
    label: "Custom OpenAI-compatible",
    baseUrl: "",
    model: "",
  },
];

interface CloudExplainPanelProps {
  state: CompanionState;
}

function pageForAction(actionId: string): PageId | null {
  switch (actionId) {
    case "open_dns_assist":
      return "dns";
    case "open_connect":
      return "connect";
    case "open_protect":
      return "protect";
    case "open_network":
    case "review_mtu":
    case "review_local_caps":
      return "network";
    default:
      return null;
  }
}

export function CloudExplainPanel({ state }: CloudExplainPanelProps) {
  const [settings, setSettings] = useState<AiExplainSettings | null>(null);
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [includeSsid, setIncludeSsid] = useState(false);
  const [consentAck, setConsentAck] = useState(false);
  const [result, setResult] = useState<CloudExplainResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [presetId, setPresetId] = useState("openai");
  const [copyNote, setCopyNote] = useState<string | null>(null);

  const refreshSettings = useCallback(async () => {
    try {
      const next = await state.getAiExplainSettings();
      setSettings(next);
      setBaseUrl(next.base_url || "https://api.openai.com/v1");
      setModel(next.model || "gpt-4o-mini");
      setIncludeSsid(next.include_ssid);
      setConsentAck(next.consent_version.length > 0);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not load AI settings.");
    }
  }, [state]);

  useEffect(() => {
    void refreshSettings();
  }, [refreshSettings]);

  async function handleEnable(enabled: boolean) {
    setBusy(true);
    setError(null);
    try {
      const next = await state.setAiExplainSettings({
        cloudEnabled: enabled,
        provider: enabled ? "openai_compat" : "none",
        baseUrl,
        model,
        includeSsid,
        consentAck: enabled ? consentAck : false,
        apiKey: apiKey.trim() ? apiKey.trim() : null,
      });
      setSettings(next);
      setApiKey("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not save AI settings.");
    } finally {
      setBusy(false);
    }
  }

  async function handleExplain() {
    setBusy(true);
    setError(null);
    try {
      const cloud = await state.explainWithCloud();
      setResult(cloud);
      if (!cloud.ok) {
        setError(cloud.error ?? "Cloud explain failed.");
      }
    } catch (err) {
      setResult(null);
      setError(err instanceof Error ? err.message : "Cloud explain failed.");
    } finally {
      setBusy(false);
    }
  }

  async function handleClearKey() {
    setBusy(true);
    setError(null);
    try {
      const next = await state.clearAiExplainKey();
      setSettings(next);
      setApiKey("");
      setResult(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not clear API key.");
    } finally {
      setBusy(false);
    }
  }

  function applyPreset(id: string) {
    setPresetId(id);
    const preset = PROVIDER_PRESETS.find((item) => item.id === id);
    if (!preset || id === "custom") {
      return;
    }
    setBaseUrl(preset.baseUrl);
    setModel(preset.model);
  }

  async function handleCopyReply() {
    if (!result?.ok || !result.plain_markdown) {
      return;
    }
    try {
      await navigator.clipboard.writeText(result.plain_markdown);
      setCopyNote("Copied reply to clipboard.");
    } catch {
      setCopyNote("Could not copy reply.");
    }
  }

  const enabled = settings?.cloud_enabled === true;

  return (
    <Card className="border-border/70 bg-muted/10">
      <CardHeader className="pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <div className="bg-primary/10 text-primary flex size-8 items-center justify-center rounded-lg">
              <Cloud className="size-4" />
            </div>
            <CardTitle className="text-base">Explain with AI (opt-in)</CardTitle>
          </div>
          <Button
            size="sm"
            variant="secondary"
            disabled={busy || !state.report || !enabled}
            onClick={() => void handleExplain()}
          >
            {busy ? "Working…" : "Send redacted bundle"}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-muted-foreground text-xs leading-relaxed">{DATA_SHEET}</p>

        <label className="flex items-start gap-2 text-xs">
          <input
            type="checkbox"
            className="mt-0.5"
            checked={consentAck}
            onChange={(event) => setConsentAck(event.target.checked)}
          />
          <span>I understand the data sheet and consent to send a redacted bundle when I ask.</span>
        </label>

        <div className="grid gap-2 sm:grid-cols-2">
          <label className="grid gap-1 text-xs sm:col-span-2">
            Provider preset
            <select
              className="rounded-md border border-border bg-background px-2 py-1"
              value={presetId}
              onChange={(event) => applyPreset(event.target.value)}
            >
              {PROVIDER_PRESETS.map((preset) => (
                <option key={preset.id} value={preset.id}>
                  {preset.label}
                </option>
              ))}
            </select>
          </label>
          <label className="grid gap-1 text-xs">
            Base URL
            <input
              className="rounded-md border border-border bg-background px-2 py-1"
              value={baseUrl}
              onChange={(event) => {
                setPresetId("custom");
                setBaseUrl(event.target.value);
              }}
              placeholder="https://api.openai.com/v1"
            />
          </label>
          <label className="grid gap-1 text-xs">
            Model
            <input
              className="rounded-md border border-border bg-background px-2 py-1"
              value={model}
              onChange={(event) => {
                setPresetId("custom");
                setModel(event.target.value);
              }}
              placeholder="gpt-4o-mini"
            />
          </label>
          <label className="grid gap-1 text-xs sm:col-span-2">
            API key {settings?.api_key_present ? "(saved on device)" : ""}
            <input
              type="password"
              className="rounded-md border border-border bg-background px-2 py-1"
              value={apiKey}
              onChange={(event) => setApiKey(event.target.value)}
              placeholder={settings?.api_key_present ? "Leave blank to keep saved key" : "sk-…"}
            />
          </label>
        </div>

        <label className="flex items-center gap-2 text-xs">
          <input
            type="checkbox"
            checked={includeSsid}
            onChange={(event) => setIncludeSsid(event.target.checked)}
          />
          Include Wi‑Fi SSID in the redacted bundle
        </label>

        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => void handleEnable(true)}>
            Enable cloud
          </Button>
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => void handleEnable(false)}>
            Disable cloud
          </Button>
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => void handleClearKey()}>
            Clear API key
          </Button>
        </div>

        {enabled && (
          <p className="text-xs text-muted-foreground">
            Cloud explain is on ({settings?.model || "model unset"}). Sends only after you click Send.
          </p>
        )}
        {error && <p className="text-xs text-destructive">{error}</p>}
        {result?.ok && (
          <div className="space-y-2 rounded-lg border border-border/70 bg-background/40 p-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <p className="text-xs font-medium">Provider reply ({result.model})</p>
              <div className="flex flex-wrap gap-2">
                <Button size="sm" variant="secondary" onClick={() => void handleCopyReply()}>
                  Copy reply
                </Button>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => {
                    setResult(null);
                    setCopyNote(null);
                  }}
                >
                  Clear reply
                </Button>
              </div>
            </div>
            {copyNote && <p className="text-xs text-muted-foreground">{copyNote}</p>}
            <pre className="whitespace-pre-wrap text-xs leading-relaxed">{result.plain_markdown}</pre>
            {result.parsed_actions.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {result.parsed_actions.map((actionId) => {
                  const page = pageForAction(actionId);
                  return (
                    <Button
                      key={actionId}
                      size="sm"
                      variant="secondary"
                      disabled={!page}
                      onClick={() => page && state.navigate(page)}
                    >
                      {actionId.replace(/_/g, " ")}
                    </Button>
                  );
                })}
              </div>
            )}
            {result.rejected_actions.length > 0 && (
              <p className="text-xs text-muted-foreground">
                Ignored unknown action ids: {result.rejected_actions.join(", ")}
              </p>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
