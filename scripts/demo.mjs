import {
  PrivatePredictionMarket,
  encryptForSimulation
} from "../src/market.mjs";

const market = new PrivatePredictionMarket({
  question: "Will private prediction markets reduce copy-trading?",
  closesAt: new Date("2026-06-01T00:00:00Z")
});

market.submitEncryptedPrediction(
  encryptForSimulation({ yes: true, stake: 500n }),
  new Date("2026-05-01T12:00:00Z")
);
market.submitEncryptedPrediction(
  encryptForSimulation({ yes: false, stake: 150n }),
  new Date("2026-05-01T12:05:00Z")
);

market.close(new Date("2026-06-02T00:00:00Z"));
console.log(market.settle());

