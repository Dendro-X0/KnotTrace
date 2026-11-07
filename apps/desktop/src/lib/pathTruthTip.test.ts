import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { shouldShowPathTruthTip } from "./pathTruthTip.ts";

describe("shouldShowPathTruthTip", () => {
  it("shows when unset", () => {
    assert.equal(shouldShowPathTruthTip(null), true);
    assert.equal(shouldShowPathTruthTip(undefined), true);
    assert.equal(shouldShowPathTruthTip(""), true);
  });

  it("hides when dismissed", () => {
    assert.equal(shouldShowPathTruthTip("1"), false);
  });
});
