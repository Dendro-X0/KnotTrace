import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { recommendPathEvidenceFocus } from "./pathEvidenceFocus.ts";

describe("recommendPathEvidenceFocus", () => {
  it("prefers tunnel compare when tunnel path is active", () => {
    const focus = recommendPathEvidenceFocus({
      activePath: "vpn_tunnel",
      breakHop: "tcp",
      upstreamClaim: "upstream_pool_poor",
    });
    assert.equal(focus.id, "tunnel_compare");
    assert.equal(focus.reasonKey, "pathFocus.reason.tunnel");
  });

  it("prefers upstream pool when claim is poor and path is Direct", () => {
    const focus = recommendPathEvidenceFocus({
      activePath: "direct",
      breakHop: "none",
      upstreamClaim: "upstream_pool_poor",
      proxyEnabled: true,
    });
    assert.equal(focus.id, "upstream_pool");
  });

  it("prefers proxy path for proxy-only failures on Direct-labeled active path", () => {
    const focus = recommendPathEvidenceFocus({
      activePath: "direct",
      breakHop: "none",
      proxyEnabled: true,
      proxyOnlyFailureCount: 2,
      likelyProviderSide: true,
    });
    assert.equal(focus.id, "proxy_path");
  });

  it("prefers path break when hop is set", () => {
    const focus = recommendPathEvidenceFocus({
      activePath: "direct",
      breakHop: "dns",
    });
    assert.equal(focus.id, "path_break");
    assert.equal(focus.reasonKey, "pathFocus.reason.pathBreak");
  });

  it("prefers egress timeline for churning claim", () => {
    const focus = recommendPathEvidenceFocus({
      activePath: "direct",
      breakHop: "none",
      egressClaim: "churning",
    });
    assert.equal(focus.id, "egress_timeline");
  });

  it("defaults to path break", () => {
    const focus = recommendPathEvidenceFocus({
      activePath: "direct",
      breakHop: "none",
    });
    assert.equal(focus.id, "path_break");
    assert.equal(focus.reasonKey, "pathFocus.reason.default");
  });
});
