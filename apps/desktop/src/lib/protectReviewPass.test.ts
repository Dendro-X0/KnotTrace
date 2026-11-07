import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  alertFingerprint,
  buildProtectReviewNote,
  filterActiveProtectAlerts,
  planProtectReviewPass,
} from "./protectReviewPass.ts";
import type { ProtectAlert } from "../types.ts";

function alert(
  title: string,
  actions: ProtectAlert["actions"] = [],
  level: ProtectAlert["level"] = "warning",
): ProtectAlert {
  return { level, title, message: `${title} detail`, actions };
}

describe("planProtectReviewPass", () => {
  it("collects unique assists and acknowledges indicator rows", () => {
    const alerts = [
      alert("Sites unreachable", [{ kind: "connect_assist", label: "Try Connect" }]),
      alert("Open network service found"),
      alert("Open network service found"),
      alert("Untrusted network", [
        { kind: "dns_assist", label: "DNS" },
        { kind: "connect_assist", label: "Connect" },
      ]),
    ];

    const plan = planProtectReviewPass(alerts);
    assert.deepEqual(plan.assistKinds, ["connect_assist", "dns_assist"]);
    assert.equal(plan.actionableCount, 2);
    assert.equal(plan.indicatorCount, 2);
    assert.equal(plan.acknowledgeTitles.length, 4);
  });

  it("skips already acknowledged fingerprints", () => {
    const row = alert("Proxy path looks forced");
    const fp = alertFingerprint(row);
    const plan = planProtectReviewPass([row], new Set([fp]));
    assert.equal(plan.acknowledgeTitles.length, 0);
    assert.equal(plan.indicatorCount, 0);
  });
});

describe("filterActiveProtectAlerts", () => {
  it("hides acknowledged rows from the active queue", () => {
    const a = alert("A");
    const b = alert("B");
    const active = filterActiveProtectAlerts([a, b], new Set([alertFingerprint(a)]));
    assert.equal(active.length, 1);
    assert.equal(active[0]?.title, "B");
  });
});

describe("buildProtectReviewNote", () => {
  it("summarizes applies and reviews", () => {
    const note = buildProtectReviewNote({
      appliedDns: true,
      appliedConnect: false,
      dnsMessage: "Switched to 1.1.1.1",
      acknowledgedCount: 3,
      skipped: ["No Connect API"],
    });
    assert.match(note, /1\.1\.1\.1/);
    assert.match(note, /3 items marked reviewed/);
    assert.match(note, /No Connect API/);
  });
});
