import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { needsEnglishProbeLabel } from "./probeLanguage.ts";

describe("needsEnglishProbeLabel", () => {
  it("labels zh-Hans", () => {
    assert.equal(needsEnglishProbeLabel("zh-Hans"), true);
  });

  it("does not label en", () => {
    assert.equal(needsEnglishProbeLabel("en"), false);
    assert.equal(needsEnglishProbeLabel(null), false);
  });
});
