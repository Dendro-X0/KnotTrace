import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  buildScriptRiskGuide,
  flattenScriptRiskHits,
  revealablePathForHit,
} from "./scriptRiskGuide.ts";
import type { SuspiciousActivitySnapshot } from "../types.ts";

function snapshot(
  overrides: Partial<SuspiciousActivitySnapshot> = {},
): SuspiciousActivitySnapshot {
  return {
    scanned: true,
    enabled: true,
    source: "test",
    persistence_hits: [],
    mining_hits: [],
    disk_hits: [],
    summary: "test",
    ...overrides,
  };
}

describe("revealablePathForHit", () => {
  it("accepts Windows file paths", () => {
    assert.equal(
      revealablePathForHit({
        name: "payload.ps1",
        location: "C:\\Users\\a\\Downloads\\payload.ps1",
      }),
      "C:\\Users\\a\\Downloads\\payload.ps1",
    );
  });

  it("rejects registry locations", () => {
    assert.equal(
      revealablePathForHit({
        name: "Updater",
        location: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
      }),
      null,
    );
  });
});

describe("buildScriptRiskGuide", () => {
  it("returns null before a scan", () => {
    assert.equal(buildScriptRiskGuide(null), null);
    assert.equal(buildScriptRiskGuide(snapshot({ scanned: false })), null);
  });

  it("guides empty results without claiming clean", () => {
    const guide = buildScriptRiskGuide(snapshot());
    assert.ok(guide);
    assert.match(guide.headline, /No indicators/i);
    assert.ok(guide.honesty.toLowerCase().includes("does not"));
    assert.equal(guide.steps.length >= 1, true);
  });

  it("prioritizes disk then mining then persistence steps", () => {
    const guide = buildScriptRiskGuide(
      snapshot({
        disk_hits: [
          { name: "x.ps1", location: "C:\\Users\\a\\Downloads\\x.ps1", detail: "IEX" },
        ],
        mining_hits: [{ name: "xmrig.exe", location: "pid 12" }],
        persistence_hits: [
          {
            name: "Updater",
            location: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
          },
        ],
      }),
    );
    assert.ok(guide);
    assert.equal(guide.steps[0]?.id, "disk-review");
    assert.equal(guide.steps[1]?.id, "mining-taskmgr");
    assert.equal(guide.steps[2]?.id, "persistence-startup");
    assert.equal(guide.steps[2]?.openStartupSettings, true);
  });
});

describe("flattenScriptRiskHits", () => {
  it("flattens all hit categories", () => {
    const rows = flattenScriptRiskHits(
      snapshot({
        persistence_hits: [{ name: "a", location: "hk" }],
        mining_hits: [{ name: "b", location: "pid" }],
        disk_hits: [{ name: "c", location: "C:\\c.ps1" }],
      }),
    );
    assert.equal(rows.length, 3);
    assert.deepEqual(
      rows.map((row) => row.kind),
      ["persistence", "mining", "disk"],
    );
  });
});
