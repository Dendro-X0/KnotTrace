import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  PATH_EXIT_JOURNAL_QUICK_LIMIT,
  buildPathExitJournal,
} from "./pathExitJournal.ts";
import type { HistoryTrendPoint } from "../types.ts";

function point(
  overrides: Partial<HistoryTrendPoint> & { timestamp: string },
): HistoryTrendPoint {
  return {
    score: 80,
    grade: "good",
    dns_latency_ms: null,
    internet_latency_ms: null,
    gateway_latency_ms: null,
    ...overrides,
  };
}

describe("buildPathExitJournal", () => {
  it("returns empty when no trends", () => {
    const journal = buildPathExitJournal([]);
    assert.equal(journal.empty, true);
    assert.equal(journal.rows.length, 0);
  });

  it("orders newest first and respects limit", () => {
    const journal = buildPathExitJournal(
      [
        point({
          timestamp: "2026-11-06T10:00:00.000Z",
          grade: "good",
          active_path: "direct",
          egress_ip: "1.1.1.1",
          slowdown_shape: "general_degradation",
        }),
        point({
          timestamp: "2026-11-06T11:00:00.000Z",
          grade: "fair",
          active_path: "system_proxy",
          egress_ip: "9.9.9.9",
          slowdown_shape: "partial_site_failure",
        }),
        point({
          timestamp: "2026-11-06T12:00:00.000Z",
          grade: "poor",
          active_path: "vpn_tunnel",
          egress_ip: "8.8.8.8",
          slowdown_shape: "tunnel_overhead",
        }),
      ],
      PATH_EXIT_JOURNAL_QUICK_LIMIT,
    );
    assert.equal(journal.empty, false);
    assert.equal(journal.rows.length, 1);
    assert.equal(journal.rows[0]?.activePath, "vpn_tunnel");
    assert.equal(journal.rows[0]?.egressIp, "8.8.8.8");
  });

  it("does not invent active_path when missing", () => {
    const journal = buildPathExitJournal([
      point({
        timestamp: "2026-11-06T10:00:00.000Z",
        grade: "fair",
        proxy_enabled: true,
        egress_ip: "203.0.113.1",
        slowdown_shape: "partial_site_failure",
      }),
    ]);
    assert.equal(journal.rows[0]?.activePath, null);
    assert.match(journal.rows[0]?.summaryLine ?? "", /—/);
    assert.ok(!journal.rows[0]?.summaryLine.includes("vpn"));
  });

  it("includes shape and grade in summary line", () => {
    const journal = buildPathExitJournal([
      point({
        timestamp: "2026-11-06T10:00:00.000Z",
        grade: "poor",
        active_path: "tor_socks",
        egress_ip: "198.51.100.2",
        slowdown_shape: "under_load_lag",
      }),
    ]);
    const line = journal.rows[0]?.summaryLine ?? "";
    assert.match(line, /poor/);
    assert.match(line, /tor socks/);
    assert.match(line, /198\.51\.100\.2/);
    assert.match(line, /under load lag/);
  });
});
