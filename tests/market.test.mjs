import assert from "node:assert/strict";
import {
  PrivatePredictionMarket,
  encryptForSimulation
} from "../src/market.mjs";

function test(name, fn) {
  fn();
  console.log(`ok - ${name}`);
}

test("settles to yes when encrypted yes stake is larger", () => {
  const market = new PrivatePredictionMarket({
    question: "Will testnet volume exceed 1M transactions this month?",
    closesAt: new Date("2026-06-01T00:00:00Z")
  });

  market.submitEncryptedPrediction(
    encryptForSimulation({ yes: true, stake: 700n }),
    new Date("2026-05-10T00:00:00Z")
  );
  market.submitEncryptedPrediction(
    encryptForSimulation({ yes: false, stake: 250n }),
    new Date("2026-05-10T00:01:00Z")
  );
  market.submitEncryptedPrediction(
    encryptForSimulation({ yes: true, stake: 100n }),
    new Date("2026-05-10T00:02:00Z")
  );

  market.close(new Date("2026-06-02T00:00:00Z"));
  const result = market.settle();

  assert.equal(result.submissions, 3);
  assert.equal(result.yesStake, 800n);
  assert.equal(result.noStake, 250n);
  assert.equal(result.winningSide, "yes");
});

test("rejects late submissions", () => {
  const market = new PrivatePredictionMarket({
    question: "Will a private prediction market demo ship?",
    closesAt: new Date("2026-06-01T00:00:00Z")
  });

  assert.throws(
    () =>
      market.submitEncryptedPrediction(
        encryptForSimulation({ yes: true, stake: 1n }),
        new Date("2026-06-01T00:00:01Z")
      ),
    /market is closed/
  );
});

