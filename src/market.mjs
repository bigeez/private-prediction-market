export class PrivatePredictionMarket {
  constructor({ question, closesAt }) {
    if (!question || typeof question !== "string") {
      throw new Error("question is required");
    }
    if (!(closesAt instanceof Date)) {
      throw new Error("closesAt must be a Date");
    }

    this.question = question;
    this.closesAt = closesAt;
    this.closed = false;
    this.settled = false;
    this.submissionCount = 0;
    this.#yesStake = 0n;
    this.#noStake = 0n;
  }

  #yesStake;
  #noStake;

  submitEncryptedPrediction(encryptedPayload, now = new Date()) {
    if (this.closed || now >= this.closesAt) {
      throw new Error("market is closed");
    }

    const prediction = decryptForSimulation(encryptedPayload);
    if (typeof prediction.yes !== "boolean") {
      throw new Error("prediction side must be boolean");
    }
    if (prediction.stake <= 0n) {
      throw new Error("stake must be positive");
    }

    if (prediction.yes) {
      this.#yesStake += prediction.stake;
    } else {
      this.#noStake += prediction.stake;
    }

    this.submissionCount += 1;
  }

  close(now = new Date()) {
    if (now < this.closesAt) {
      throw new Error("market cannot close before deadline");
    }
    this.closed = true;
  }

  settle() {
    if (!this.closed) {
      throw new Error("market must be closed before settlement");
    }
    if (this.settled) {
      throw new Error("market already settled");
    }

    this.settled = true;
    const yesStake = this.#yesStake;
    const noStake = this.#noStake;
    const totalStake = yesStake + noStake;
    const winningSide = yesStake >= noStake ? "yes" : "no";

    return {
      question: this.question,
      submissions: this.submissionCount,
      yesStake,
      noStake,
      totalStake,
      winningSide
    };
  }
}

export function encryptForSimulation({ yes, stake }) {
  const stakeBigInt = BigInt(stake);
  const encoded = JSON.stringify({ yes, stake: stakeBigInt.toString() });
  return Buffer.from(encoded, "utf8").toString("base64");
}

function decryptForSimulation(payload) {
  const decoded = Buffer.from(payload, "base64").toString("utf8");
  const parsed = JSON.parse(decoded);
  return {
    yes: parsed.yes,
    stake: BigInt(parsed.stake)
  };
}

