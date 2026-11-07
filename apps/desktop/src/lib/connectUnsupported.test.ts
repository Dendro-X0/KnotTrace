import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  isConnectUnsupported,
  shouldShowConnectSwitch,
} from "./connectUnsupported.ts";

describe("connectUnsupported", () => {
  it("treats missing kernel as unsupported", () => {
    assert.equal(isConnectUnsupported(null), true);
    assert.equal(isConnectUnsupported({ kernel: null }), true);
    assert.equal(isConnectUnsupported({ kernel: { kind: "mihomo" } }), false);
  });

  it("hides Switch when unsupported", () => {
    assert.equal(shouldShowConnectSwitch({ kernel: null, should_apply: true }), false);
  });

  it("hides Switch when kernel present but no apply target", () => {
    assert.equal(
      shouldShowConnectSwitch({
        kernel: { kind: "mihomo" },
        should_apply: false,
        recommended_switch: { target_proxy: "HK-1" },
      }),
      false,
    );
    assert.equal(
      shouldShowConnectSwitch({
        kernel: { kind: "mihomo" },
        should_apply: true,
        recommended_switch: null,
      }),
      false,
    );
  });

  it("shows Switch only when apply + target exist", () => {
    assert.equal(
      shouldShowConnectSwitch({
        kernel: { kind: "sing-box" },
        should_apply: true,
        recommended_switch: { target_proxy: "US-2" },
      }),
      true,
    );
  });
});
