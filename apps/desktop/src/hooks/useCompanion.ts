import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";

import { invokeErrorMessage } from "@/lib/utils";
import type {
  AutoProtectResult,
  BenchmarkSnapshot,
  ConnectConfig,
  ConnectRecommendation,
  DnsAssistRecommendation,
  DnsAssistState,
  DnsIntegritySettings,
  HealthReport,
  HistoryTrendPoint,
  MonitorStatus,
  PageId,
  ProxyPathComparison,
  ProtectAction,
  ProtectSettings,
  ProtectStatus,
  ThroughputProbeResult,
  ThroughputSettings,
} from "@/types";
import { isPageId, PAGE_STORAGE_KEY } from "@/types";

export function useCompanion() {
  const [page, setPage] = useState<PageId>("overview");
  const [checking, setChecking] = useState(false);
  const [report, setReport] = useState<HealthReport | null>(null);
  const [checkError, setCheckError] = useState<string | null>(null);
  const [history, setHistory] = useState<HealthReport[]>([]);
  const [historyError, setHistoryError] = useState<string | null>(null);
  const [trends, setTrends] = useState<HistoryTrendPoint[]>([]);
  const [trendsError, setTrendsError] = useState<string | null>(null);
  const [monitorStatus, setMonitorStatus] = useState<MonitorStatus | null>(null);
  const [monitorError, setMonitorError] = useState<string | null>(null);

  const [dnsRecommendation, setDnsRecommendation] = useState<DnsAssistRecommendation | null>(null);
  const [dnsState, setDnsState] = useState<DnsAssistState | null>(null);
  const [dnsSummary, setDnsSummary] = useState("Analyzing whether a faster DNS resolver could help.");
  const [dnsError, setDnsError] = useState<string | null>(null);
  const [dnsApplying, setDnsApplying] = useState(false);
  const [dnsRestoring, setDnsRestoring] = useState(false);
  const [integritySettings, setIntegritySettings] = useState<DnsIntegritySettings | null>(null);
  const [integritySettingsError, setIntegritySettingsError] = useState<string | null>(null);
  const [integritySettingsSaving, setIntegritySettingsSaving] = useState(false);
  const [benchmarkSnapshots, setBenchmarkSnapshots] = useState<BenchmarkSnapshot[]>([]);
  const [benchmarkError, setBenchmarkError] = useState<string | null>(null);
  const [benchmarkSaving, setBenchmarkSaving] = useState(false);

  const [throughputSettings, setThroughputSettings] = useState<ThroughputSettings | null>(null);
  const [throughputResult, setThroughputResult] = useState<ThroughputProbeResult | null>(null);
  const [throughputError, setThroughputError] = useState<string | null>(null);
  const [throughputRunning, setThroughputRunning] = useState(false);
  const [throughputSaving, setThroughputSaving] = useState(false);
  const [proxyComparison, setProxyComparison] = useState<ProxyPathComparison | null>(null);
  const [proxyComparisonError, setProxyComparisonError] = useState<string | null>(null);
  const [proxyComparing, setProxyComparing] = useState(false);

  const [connectRecommendation, setConnectRecommendation] =
    useState<ConnectRecommendation | null>(null);
  const [connectSummary, setConnectSummary] = useState(
    "Looking for Mihomo or sing-box on your machine.",
  );
  const [connectError, setConnectError] = useState<string | null>(null);
  const [connectApiBase, setConnectApiBase] = useState("");
  const [connectSecret, setConnectSecret] = useState("");
  const [connectSaving, setConnectSaving] = useState(false);
  const [connectApplying, setConnectApplying] = useState(false);

  const [protectStatus, setProtectStatus] = useState<ProtectStatus | null>(null);
  const [protectError, setProtectError] = useState<string | null>(null);
  const [autoProtectNote, setAutoProtectNote] = useState("");
  const [bootstrapping, setBootstrapping] = useState(true);

  const navigate = useCallback((next: PageId) => {
    setPage(next);
    try {
      sessionStorage.setItem(PAGE_STORAGE_KEY, next);
    } catch {
      // Storage may be unavailable.
    }
  }, []);

  const refreshHistory = useCallback(async () => {
    try {
      const items = await invoke<HealthReport[]>("get_history", { limit: 6 });
      setHistory(items);
      setHistoryError(null);
    } catch {
      setHistory([]);
      setHistoryError("History unavailable.");
    }
  }, []);

  const refreshTrends = useCallback(async () => {
    try {
      const points = await invoke<HistoryTrendPoint[]>("get_history_trends", { limit: 48 });
      setTrends(points);
      setTrendsError(null);
    } catch {
      setTrends([]);
      setTrendsError("Trend data is unavailable right now.");
    }
  }, []);

  const refreshMonitorStatus = useCallback(async () => {
    try {
      const status = await invoke<MonitorStatus>("get_monitor_status");
      setMonitorStatus(status);
      setMonitorError(null);
    } catch {
      setMonitorStatus(null);
      setMonitorError("Background monitor: unavailable");
    }
  }, []);

  const refreshAssist = useCallback(async () => {
    try {
      const [recommendation, state] = await Promise.all([
        invoke<DnsAssistRecommendation>("recommend_dns"),
        invoke<DnsAssistState>("get_dns_assist_state"),
      ]);
      setDnsRecommendation(recommendation);
      setDnsState(state);
      setDnsError(null);

      if (state.active && state.backup) {
        setDnsSummary(
          `Using ${state.backup.applied_resolver} on ${state.backup.interface_alias}. You can restore your previous DNS at any time.`,
        );
      } else {
        setDnsSummary(recommendation.reason);
      }
    } catch (error) {
      setDnsRecommendation(null);
      setDnsState(null);
      setDnsSummary(
        error instanceof Error ? error.message : "DNS assist is unavailable right now.",
      );
      setDnsError(error instanceof Error ? error.message : "DNS assist is unavailable right now.");
    }
  }, []);

  const loadConnectConfig = useCallback(async () => {
    try {
      const config = await invoke<ConnectConfig | null>("get_connect_config");
      if (config?.api_base) setConnectApiBase(config.api_base);
      if (config?.secret) setConnectSecret(config.secret);
    } catch {
      // Optional config.
    }
  }, []);

  const refreshConnect = useCallback(async () => {
    try {
      await invoke<ConnectConfig | null>("discover_connect");
      const recommendation = await invoke<ConnectRecommendation>("recommend_connect");
      setConnectRecommendation(recommendation);
      setConnectSummary(recommendation.reason);
      setConnectError(null);
      await loadConnectConfig();
    } catch (error) {
      setConnectRecommendation(null);
      setConnectSummary(
        error instanceof Error ? error.message : "Connect assist is unavailable right now.",
      );
      setConnectError(
        error instanceof Error ? error.message : "Connect assist is unavailable right now.",
      );
    }
  }, [loadConnectConfig]);

  const refreshProtect = useCallback(async () => {
    try {
      const status = await invoke<ProtectStatus>("get_protect_status");
      setProtectStatus(status);
      setProtectError(null);
    } catch (error) {
      setProtectStatus(null);
      setProtectError(
        error instanceof Error ? error.message : "Protect status unavailable.",
      );
    }
  }, []);

  const refreshIntegritySettings = useCallback(async () => {
    try {
      const settings = await invoke<DnsIntegritySettings>("get_dns_integrity_settings");
      setIntegritySettings(settings);
      setIntegritySettingsError(null);
    } catch (error) {
      setIntegritySettings(null);
      setIntegritySettingsError(
        error instanceof Error ? error.message : "Could not load integrity settings.",
      );
    }
  }, []);

  const refreshBenchmarks = useCallback(async () => {
    try {
      const snapshots = await invoke<BenchmarkSnapshot[]>("list_benchmarks");
      setBenchmarkSnapshots(snapshots);
      setBenchmarkError(null);
    } catch (error) {
      setBenchmarkSnapshots([]);
      setBenchmarkError(
        error instanceof Error ? error.message : "Could not load benchmark snapshots.",
      );
    }
  }, []);

  const refreshThroughputSettings = useCallback(async () => {
    try {
      const settings = await invoke<ThroughputSettings>("get_throughput_settings");
      setThroughputSettings(settings);
      setThroughputError(null);
    } catch (error) {
      setThroughputSettings(null);
      setThroughputError(
        error instanceof Error ? error.message : "Could not load throughput settings.",
      );
    }
  }, []);

  const runCheck = useCallback(async () => {
    setChecking(true);
    setCheckError(null);
    try {
      const next = await invoke<HealthReport>("run_check");
      setReport(next);
      await Promise.all([
        refreshHistory(),
        refreshMonitorStatus(),
        refreshAssist(),
        refreshConnect(),
        refreshProtect(),
        refreshTrends(),
        refreshIntegritySettings(),
        refreshBenchmarks(),
      ]);
    } catch (error) {
      setCheckError(invokeErrorMessage(error, "Health check failed."));
    } finally {
      setChecking(false);
    }
  }, [
    refreshAssist,
    refreshBenchmarks,
    refreshConnect,
    refreshHistory,
    refreshIntegritySettings,
    refreshMonitorStatus,
    refreshProtect,
    refreshTrends,
  ]);

  const saveIntegritySettings = useCallback(
    async (verificationDomains: string[]) => {
      setIntegritySettingsSaving(true);
      try {
        const settings = await invoke<DnsIntegritySettings>("set_dns_integrity_settings", {
          verificationDomains,
        });
        setIntegritySettings(settings);
        setIntegritySettingsError(null);
        await runCheck();
      } catch (error) {
        setIntegritySettingsError(
          error instanceof Error ? error.message : "Could not save integrity settings.",
        );
        throw error;
      } finally {
        setIntegritySettingsSaving(false);
      }
    },
    [runCheck],
  );

  const saveBenchmark = useCallback(
    async (input: {
      label: string;
      downloadMbps?: number;
      uploadMbps?: number;
      pingMs?: number;
      notes?: string;
    }) => {
      setBenchmarkSaving(true);
      try {
        await invoke<BenchmarkSnapshot>("save_benchmark", {
          label: input.label,
          downloadMbps: input.downloadMbps ?? null,
          uploadMbps: input.uploadMbps ?? null,
          pingMs: input.pingMs ?? null,
          notes: input.notes ?? null,
        });
        await refreshBenchmarks();
        setBenchmarkError(null);
      } catch (error) {
        setBenchmarkError(
          error instanceof Error ? error.message : "Could not save benchmark snapshot.",
        );
        throw error;
      } finally {
        setBenchmarkSaving(false);
      }
    },
    [refreshBenchmarks],
  );

  const deleteBenchmark = useCallback(
    async (id: string) => {
      try {
        await invoke("delete_benchmark", { id });
        await refreshBenchmarks();
        setBenchmarkError(null);
      } catch (error) {
        setBenchmarkError(
          error instanceof Error ? error.message : "Could not delete benchmark snapshot.",
        );
      }
    },
    [refreshBenchmarks],
  );

  const saveThroughputSettings = useCallback(
    async (downloadBytes: number, uploadBytes: number) => {
      setThroughputSaving(true);
      try {
        const settings = await invoke<ThroughputSettings>("set_throughput_settings", {
          downloadBytes,
          uploadBytes,
        });
        setThroughputSettings(settings);
        setThroughputError(null);
      } catch (error) {
        setThroughputError(
          error instanceof Error ? error.message : "Could not save throughput settings.",
        );
        throw error;
      } finally {
        setThroughputSaving(false);
      }
    },
    [],
  );

  const runThroughputTest = useCallback(async () => {
    setThroughputRunning(true);
    try {
      const result = await invoke<ThroughputProbeResult>("run_throughput_test");
      setThroughputResult(result);
      setThroughputError(null);
    } catch (error) {
      setThroughputResult(null);
      setThroughputError(
        error instanceof Error ? error.message : "Throughput test failed.",
      );
      throw error;
    } finally {
      setThroughputRunning(false);
    }
  }, []);

  const compareProxyPaths = useCallback(async (groupName: string) => {
    setProxyComparing(true);
    try {
      const result = await invoke<ProxyPathComparison>("compare_proxy_paths", { groupName });
      setProxyComparison(result);
      setProxyComparisonError(null);
    } catch (error) {
      setProxyComparison(null);
      setProxyComparisonError(
        error instanceof Error ? error.message : "Proxy path comparison failed.",
      );
      throw error;
    } finally {
      setProxyComparing(false);
    }
  }, []);

  const setMonitorEnabled = useCallback(
    async (enabled: boolean) => {
      try {
        await invoke("set_monitor_enabled", { enabled });
        await refreshMonitorStatus();
      } catch (error) {
        setMonitorError(
          error instanceof Error ? error.message : "Could not update monitor setting.",
        );
        throw error;
      }
    },
    [refreshMonitorStatus],
  );

  const applyRecommendedDns = useCallback(async () => {
    const resolver = dnsRecommendation?.recommended?.resolver;
    if (!resolver) return;

    setDnsApplying(true);
    try {
      const result = await invoke<{ message: string; kept: boolean }>("apply_dns", { resolver });
      setDnsSummary(result.message);
      await runCheck();
      await refreshAssist();
    } catch (error) {
      setDnsSummary(error instanceof Error ? error.message : "Could not apply DNS.");
      await refreshAssist();
    } finally {
      setDnsApplying(false);
    }
  }, [dnsRecommendation, refreshAssist, runCheck]);

  const restoreDns = useCallback(async () => {
    setDnsRestoring(true);
    try {
      const message = await invoke<string>("restore_dns");
      setDnsSummary(message);
      await runCheck();
      await refreshAssist();
    } catch (error) {
      setDnsSummary(error instanceof Error ? error.message : "Could not restore DNS.");
      await refreshAssist();
    } finally {
      setDnsRestoring(false);
    }
  }, [refreshAssist, runCheck]);

  const saveConnectConfig = useCallback(async () => {
    setConnectSaving(true);
    try {
      await invoke("set_connect_config", {
        apiBase: connectApiBase.trim(),
        secret: connectSecret.trim() || null,
      });
      await refreshConnect();
    } catch (error) {
      setConnectSummary(
        error instanceof Error ? error.message : "Could not save API settings.",
      );
    } finally {
      setConnectSaving(false);
    }
  }, [connectApiBase, connectSecret, refreshConnect]);

  const applyRecommendedConnect = useCallback(async () => {
    setConnectApplying(true);
    try {
      const result = await invoke<{ message: string }>("apply_recommended_connect");
      setConnectSummary(result.message);
      await runCheck();
      await refreshConnect();
    } catch (error) {
      setConnectSummary(
        error instanceof Error ? error.message : "Could not switch proxy node.",
      );
      await refreshConnect();
    } finally {
      setConnectApplying(false);
    }
  }, [refreshConnect, runCheck]);

  const saveProtectSettings = useCallback(
    async (partial: Partial<ProtectSettings>) => {
      const current = await invoke<ProtectSettings>("get_protect_settings");
      const next: ProtectSettings = { ...current, ...partial };
      await invoke("set_protect_settings", { settings: next });
      await refreshProtect();
    },
    [refreshProtect],
  );

  const handleProtectAction = useCallback(
    (kind: ProtectAction["kind"]) => {
      if (kind === "dns_assist") {
        navigate("dns");
        void refreshAssist();
        return;
      }
      if (kind === "connect_assist") {
        navigate("connect");
        void refreshConnect();
        return;
      }
      void runCheck();
    },
    [navigate, refreshAssist, refreshConnect, runCheck],
  );

  const applyAutoProtectResult = useCallback((result: AutoProtectResult) => {
    if (result.skipped_reason && result.applied.length === 0) {
      setAutoProtectNote(result.skipped_reason);
      return;
    }
    const lines = result.applied.map((action) => {
      const prefix = action.success ? "Applied" : "Skipped";
      return `${prefix} ${action.kind}: ${action.message}`;
    });
    setAutoProtectNote(lines.join(" · "));
  }, []);

  useEffect(() => {
    try {
      const saved = sessionStorage.getItem(PAGE_STORAGE_KEY);
      if (isPageId(saved)) setPage(saved);
    } catch {
      // Keep default page.
    }
  }, []);

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      try {
        const cached = await invoke<HealthReport | null>("get_last_report");
        if (!cancelled && cached) setReport(cached);
      } catch {
        // First launch.
      }

      if (cancelled) return;

      await Promise.all([
        refreshHistory(),
        refreshMonitorStatus(),
        refreshAssist(),
        loadConnectConfig(),
        refreshConnect(),
        refreshProtect(),
        refreshTrends(),
        refreshIntegritySettings(),
        refreshBenchmarks(),
        refreshThroughputSettings(),
      ]);

      if (!cancelled) {
        setBootstrapping(false);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [
    loadConnectConfig,
    refreshAssist,
    refreshBenchmarks,
    refreshConnect,
    refreshHistory,
    refreshIntegritySettings,
    refreshMonitorStatus,
    refreshProtect,
    refreshThroughputSettings,
    refreshTrends,
  ]);

  useEffect(() => {
    const unsubs: Array<() => void> = [];

    void (async () => {
      unsubs.push(
        await listen<AutoProtectResult>("auto-protect-result", (event) => {
          applyAutoProtectResult(event.payload);
          void refreshAssist();
          void refreshConnect();
          void refreshTrends();
        }),
      );
      unsubs.push(
        await listen<ProtectStatus>("protect-status-updated", (event) => {
          setProtectStatus(event.payload);
          setProtectError(null);
        }),
      );
      unsubs.push(
        await listen<string>("health-check-failed", (event) => {
          setCheckError(event.payload);
        }),
      );
      unsubs.push(
        await listen<HealthReport>("health-report-updated", (event) => {
          setReport(event.payload);
          setCheckError(null);
          void refreshHistory();
          void refreshMonitorStatus();
          void refreshAssist();
          void refreshConnect();
          void refreshProtect();
          void refreshTrends();
        }),
      );
    })();

    return () => {
      unsubs.forEach((unsub) => unsub());
    };
  }, [
    applyAutoProtectResult,
    refreshAssist,
    refreshBenchmarks,
    refreshConnect,
    refreshHistory,
    refreshMonitorStatus,
    refreshProtect,
    refreshTrends,
  ]);

  return {
    page,
    navigate,
    checking,
    report,
    checkError,
    history,
    historyError,
    trends,
    trendsError,
    monitorStatus,
    monitorError,
    dnsRecommendation,
    dnsState,
    dnsSummary,
    dnsError,
    dnsApplying,
    dnsRestoring,
    integritySettings,
    integritySettingsError,
    integritySettingsSaving,
    benchmarkSnapshots,
    benchmarkError,
    benchmarkSaving,
    throughputSettings,
    throughputResult,
    throughputError,
    throughputRunning,
    throughputSaving,
    proxyComparison,
    proxyComparisonError,
    proxyComparing,
    connectRecommendation,
    connectSummary,
    connectError,
    connectApiBase,
    setConnectApiBase,
    connectSecret,
    setConnectSecret,
    connectSaving,
    connectApplying,
    protectStatus,
    protectError,
    autoProtectNote,
    bootstrapping,
    runCheck,
    setMonitorEnabled,
    applyRecommendedDns,
    restoreDns,
    saveIntegritySettings,
    saveBenchmark,
    deleteBenchmark,
    saveThroughputSettings,
    runThroughputTest,
    compareProxyPaths,
    saveConnectConfig,
    applyRecommendedConnect,
    saveProtectSettings,
    handleProtectAction,
  };
}

export type CompanionState = ReturnType<typeof useCompanion>;
