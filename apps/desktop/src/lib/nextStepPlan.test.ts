import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  buildNextStepPlanFromInput,
  buildQuickHeroSummary,
  shouldSurfaceQuickNextCtas,
  type NextStepInput,
  type NextStepPlan,
} from "./nextStepPlan.ts";

function base(overrides: Partial<NextStepInput> = {}): NextStepInput {
  return {
    shape: "general_degradation",
    confidence: "low",
    thrashForbidden: false,
    dnsAssistEligible: false,
    connectAssistEligible: false,
    integrityCount: 0,
    recurrenceCount: 0,
    captiveState: "none",
    bufferbloatGrade: "other",
    activePath: null,
    localOk: false,
    torSocksDown: false,
    ...overrides,
  };
}

describe("buildNextStepPlanFromInput", () => {
  it("limits low confidence to one Network action for general degradation", () => {
    const plan = buildNextStepPlanFromInput(base());
    assert.equal(plan.actions.length, 1);
    assert.equal(plan.actions[0]?.page, "network");
    assert.equal(plan.titleKey, "nextSteps.copy.softTitle");
  });

  it("keeps DNS Assist for page_start even when not apply-eligible", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "page_start",
        confidence: "high",
        dnsAssistEligible: false,
      }),
    );
    assert.equal(plan.actions[0]?.page, "dns");
    assert.ok(plan.actions.every((action) => action.page !== "connect"));
  });

  it("omits Connect Assist when not eligible", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "partial_site_failure",
        confidence: "high",
        connectAssistEligible: false,
      }),
    );
    assert.ok(plan.actions.every((action) => action.page !== "connect"));
    assert.equal(plan.actions[0]?.page, "network");
  });

  it("keeps Connect Assist when eligible and thrash is not forbidden", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "partial_site_failure",
        confidence: "high",
        connectAssistEligible: true,
      }),
    );
    assert.deepEqual(
      plan.actions.map((action) => action.page),
      ["network", "connect"],
    );
  });

  it("never offers Connect when thrash is forbidden", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "tunnel_overhead",
        confidence: "medium",
        thrashForbidden: true,
        connectAssistEligible: false,
      }),
    );
    assert.ok(plan.actions.every((action) => action.page !== "connect"));
    assert.ok(plan.actions.some((action) => action.page === "overview"));
  });

  it("caps medium confidence at two actions", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "page_start",
        confidence: "medium",
        integrityCount: 2,
      }),
    );
    assert.equal(plan.actions.length, 2);
    assert.deepEqual(
      plan.actions.map((action) => action.page),
      ["dns", "network"],
    );
  });

  it("prioritizes preferred page when recurrence count is at least 2", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "restricted_network",
        confidence: "high",
        recurrenceCount: 2,
      }),
    );
    assert.equal(plan.actions[0]?.page, "protect");
    assert.equal(plan.prioritized, false);
  });

  it("marks prioritized when recurrence moves a later action first", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "partial_site_failure",
        confidence: "high",
        connectAssistEligible: true,
        recurrenceCount: 3,
      }),
    );
    assert.equal(plan.actions[0]?.page, "connect");
    assert.equal(plan.prioritized, true);
  });

  it("uses soft copy for low-confidence shaped diagnoses", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "tunnel_overhead",
        confidence: "low",
      }),
    );
    assert.equal(plan.actions.length, 1);
    assert.equal(plan.summaryKey, "nextSteps.copy.softSummary");
  });

  it("leads captive confirmed with Network only and no DNS", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "restricted_network",
        confidence: "high",
        captiveState: "confirmed",
        dnsAssistEligible: true,
      }),
    );
    assert.deepEqual(
      plan.actions.map((action) => action.page),
      ["network"],
    );
    assert.equal(plan.titleKey, "nextSteps.copy.captiveConfirmedTitle");
    assert.equal(plan.summaryKey, "nextSteps.copy.captiveConfirmedSummary");
  });

  it("names bufferbloat for under_load_lag without ISP blame", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "under_load_lag",
        confidence: "high",
        bufferbloatGrade: "severe",
      }),
    );
    assert.equal(plan.titleKey, "nextSteps.copy.bufferbloatTitle");
    assert.equal(plan.summaryKey, "nextSteps.copy.bufferbloatSummary");
    assert.equal(plan.actions[0]?.page, "network");
  });

  it("uses tunnel honesty with Network-only and no Connect when local OK", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "tunnel_overhead",
        confidence: "medium",
        activePath: "tor_socks",
        localOk: true,
        connectAssistEligible: true,
        dnsAssistEligible: true,
      }),
    );
    assert.equal(plan.titleKey, "nextSteps.copy.tunnelHonestyTitle");
    assert.deepEqual(
      plan.actions.map((action) => action.page),
      ["network"],
    );
  });

  it("does not offer Connect for tunnel thrash either", () => {
    const plan = buildNextStepPlanFromInput(
      base({
        shape: "tunnel_overhead",
        confidence: "medium",
        thrashForbidden: true,
        activePath: "system_proxy",
        localOk: true,
        connectAssistEligible: false,
      }),
    );
    assert.ok(plan.actions.every((action) => action.page !== "connect"));
    assert.equal(plan.titleKey, "nextSteps.copy.tunnelThrashTitle");
  });
});

describe("buildQuickHeroSummary", () => {
  it("prefers captive confirmed copy over score summary", () => {
    const summary = buildQuickHeroSummary({
      trends: [],
      report: {
        score: { grade: "poor", summary: "Connection looks slow overall." },
        network_context: { captive_portal: { state: "confirmed" } },
        diagnosis: {
          slowdown_shape: "restricted_network",
          confidence: "high",
          summary: "Restricted network.",
        },
      },
    });
    assert.deepEqual(summary, { kind: "key", key: "nextSteps.hero.captiveConfirmed" });
  });

  it("prefers bufferbloat copy over score summary", () => {
    const summary = buildQuickHeroSummary({
      trends: [],
      report: {
        score: { grade: "fair", summary: "Your ISP path may be limited." },
        stability: { bufferbloat: { grade: "moderate" } },
        diagnosis: {
          slowdown_shape: "under_load_lag",
          confidence: "high",
          summary: "Under load lag.",
        },
      },
    });
    assert.deepEqual(summary, { kind: "key", key: "nextSteps.hero.bufferbloat" });
  });

  it("names tunnel expected when VPN path is active and local OK", () => {
    const summary = buildQuickHeroSummary({
      trends: [],
      report: {
        score: { grade: "fair", summary: "Connection is a bit slow." },
        path_break: {
          active_path: "vpn_tunnel",
          local_ok: true,
          break_hop: "none",
        },
        diagnosis: {
          slowdown_shape: "tunnel_overhead",
          confidence: "medium",
          summary: "Tunnel overhead.",
        },
      },
    });
    assert.deepEqual(summary, { kind: "key", key: "nextSteps.hero.tunnelExpected" });
  });
});

describe("shouldSurfaceQuickNextCtas", () => {
  const plan: NextStepPlan = {
    titleKey: "t",
    summaryKey: "s",
    actions: [{ labelKey: "nextSteps.action.network", page: "network" }],
    prioritized: false,
  };

  it("surfaces CTAs on good grade when bufferbloat is moderate", () => {
    assert.equal(
      shouldSurfaceQuickNextCtas(
        {
          trends: [],
          report: {
            score: { grade: "good", summary: "Looks fine." },
            stability: { bufferbloat: { grade: "moderate" } },
            diagnosis: { slowdown_shape: "under_load_lag", confidence: "high" },
          },
        },
        plan,
      ),
      true,
    );
  });

  it("keeps calm good grade without path/integrity/captive/bufferbloat", () => {
    assert.equal(
      shouldSurfaceQuickNextCtas(
        {
          trends: [],
          report: {
            score: { grade: "good", summary: "Looks fine." },
            diagnosis: { slowdown_shape: "general_degradation", confidence: "low" },
          },
        },
        plan,
      ),
      false,
    );
  });
});
