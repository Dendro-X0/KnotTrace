import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { buildQuickPathTeaserContent } from "./pathTeaser.ts";

describe("buildQuickPathTeaserContent", () => {
  it("omits expectation on direct path", () => {
    const content = buildQuickPathTeaserContent({
      title: "Direct path OK",
      summary: "No break.",
      active_path: "direct",
      local_ok: true,
      break_hop: "none",
    });
    assert.equal(content.expectation, null);
    assert.equal(content.tunnelActive, false);
  });

  it("adds honesty expectation when tunnel active and local OK", () => {
    const content = buildQuickPathTeaserContent(
      {
        title: "VPN path",
        summary: "No local break.",
        active_path: "vpn_tunnel",
        local_ok: true,
        break_hop: "none",
      },
      {
        expectation: "A VPN/TUN interface is active. Extra latency is normal.",
        vpn_detected: true,
      },
    );
    assert.equal(content.tunnelActive, true);
    assert.deepEqual(content.expectation, {
      kind: "text",
      text: "A VPN/TUN interface is active. Extra latency is normal.",
    });
  });

  it("marks local-first when tunnel active but local broken", () => {
    const content = buildQuickPathTeaserContent({
      title: "Local DNS break",
      summary: "DNS hop failed.",
      active_path: "tor_socks",
      local_ok: false,
      break_hop: "dns",
    });
    assert.deepEqual(content.expectation, {
      kind: "key",
      key: "nextSteps.path.expectationLocalFirst",
    });
  });
});
