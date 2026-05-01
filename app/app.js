const closesAt = new Date("2026-06-01T00:00:00Z");
let market;
let privateInputs;

class PrivatePredictionMarket {
  constructor({ question, closesAt }) {
    this.question = question;
    this.closesAt = closesAt;
    this.closed = false;
    this.settled = false;
    this.submissionCount = 0;
    this.yesStake = 0n;
    this.noStake = 0n;
  }

  submitEncryptedPrediction(encryptedPayload, now = new Date()) {
    if (this.closed || now >= this.closesAt) {
      throw new Error("market is closed");
    }

    const prediction = decryptForDemo(encryptedPayload);
    if (prediction.yes) {
      this.yesStake += prediction.stake;
    } else {
      this.noStake += prediction.stake;
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
    const winningSide = this.yesStake >= this.noStake ? "yes" : "no";

    return {
      submissions: this.submissionCount,
      winningSide
    };
  }
}

function encryptForSimulation({ yes, stake }) {
  const encoded = JSON.stringify({ yes, stake: BigInt(stake).toString() });
  return btoa(encoded);
}

function decryptForDemo(payload) {
  const parsed = JSON.parse(atob(payload));
  return {
    yes: parsed.yes,
    stake: BigInt(parsed.stake)
  };
}

const canvas = document.querySelector("#market-canvas");
const ctx = canvas.getContext("2d");
const form = document.querySelector("#prediction-form");
const sideInput = document.querySelector("#side-input");
const stakeInput = document.querySelector("#stake-input");
const closeButton = document.querySelector("#close-button");
const settleButton = document.querySelector("#settle-button");
const resetButton = document.querySelector("#reset-button");
const eventLog = document.querySelector("#event-log");

const submissionCount = document.querySelector("#submission-count");
const visibleSignal = document.querySelector("#visible-signal");
const settlementState = document.querySelector("#settlement-state");
const marketStatus = document.querySelector("#market-status");

function resetMarket() {
  market = new PrivatePredictionMarket({
    question: "Will private prediction markets reduce copy-trading?",
    closesAt
  });
  privateInputs = [];
  eventLog.replaceChildren();
  log("Market opened. Individual predictions are hidden.");
  render();
}

function log(message) {
  const item = document.createElement("li");
  item.textContent = message;
  eventLog.prepend(item);
}

function submitPrediction(event) {
  event.preventDefault();
  const side = sideInput.value === "yes";
  const stake = BigInt(Math.max(1, Number(stakeInput.value || 1)));

  const payload = encryptForSimulation({ yes: side, stake });
  market.submitEncryptedPrediction(payload, new Date("2026-05-10T12:00:00Z"));
  privateInputs.push({ yes: side, stake });

  log(`Encrypted ${side ? "YES" : "NO"} prediction queued with private stake.`);
  render();
}

function closeMarket() {
  market.close(new Date("2026-06-02T00:00:00Z"));
  log("Market closed. No new encrypted predictions can be submitted.");
  render();
}

function settleMarket() {
  if (!market.closed) {
    closeMarket();
  }
  const result = market.settle();
  log(`Settlement revealed: ${result.winningSide.toUpperCase()} wins.`);
  render(result);
}

function render(result = null) {
  submissionCount.textContent = String(market.submissionCount);
  visibleSignal.textContent = market.settled ? "Revealed" : "Hidden";
  settlementState.textContent = market.settled
    ? `${result?.winningSide?.toUpperCase() ?? "Final"}`
    : "Pending";

  marketStatus.textContent = market.settled ? "Settled" : market.closed ? "Closed" : "Open";
  marketStatus.classList.toggle("closed", market.closed && !market.settled);
  marketStatus.classList.toggle("settled", market.settled);

  draw(result);
}

function draw(result) {
  const width = canvas.width;
  const height = canvas.height;
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = "#080b12";
  ctx.fillRect(0, 0, width, height);

  for (let x = 0; x < width; x += 28) {
    for (let y = 0; y < height; y += 28) {
      ctx.fillStyle = "rgba(73, 210, 255, 0.12)";
      ctx.fillRect(x, y, 2, 2);
    }
  }

  ctx.font = "700 16px Inter, sans-serif";
  ctx.fillStyle = "#a7b0c0";
  ctx.fillText("Encrypted order flow", 34, 42);

  privateInputs.forEach((entry, index) => {
    const x = 70 + index * 72;
    const y = 128 + Math.sin(index * 1.2) * 38;
    ctx.beginPath();
    ctx.arc(x, y, 18 + Number(entry.stake % 12n), 0, Math.PI * 2);
    ctx.fillStyle = market.settled
      ? entry.yes
        ? "rgba(112, 225, 132, 0.84)"
        : "rgba(255, 110, 122, 0.84)"
      : "rgba(73, 210, 255, 0.5)";
    ctx.fill();
    ctx.strokeStyle = "rgba(244, 247, 251, 0.7)";
    ctx.stroke();
  });

  ctx.fillStyle = "#f4f7fb";
  ctx.font = "900 28px Inter, sans-serif";
  ctx.fillText(market.settled ? "Signal revealed" : "Signal hidden", 34, 288);

  ctx.font = "600 14px Inter, sans-serif";
  ctx.fillStyle = "#a7b0c0";
  const caption = market.settled
    ? `Winning side: ${result?.winningSide?.toUpperCase() ?? "FINAL"}`
    : "Side and stake remain encrypted until settlement.";
  ctx.fillText(caption, 34, 315);
}

form.addEventListener("submit", submitPrediction);
closeButton.addEventListener("click", closeMarket);
settleButton.addEventListener("click", settleMarket);
resetButton.addEventListener("click", resetMarket);

resetMarket();
