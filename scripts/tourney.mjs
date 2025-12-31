#!/usr/bin/env node

import readline from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";
import fs from "node:fs/promises";

const API_KEY = process.env.RIOT_API_KEY;

if (!API_KEY) {
  console.error("RIOT_API_KEY is not set");
  process.exit(1);
}

const rl = readline.createInterface({ input, output });

async function riotFetch(url, params = {}) {
  const u = new URL(url);
  Object.entries(params).forEach(([k, v]) => u.searchParams.set(k, v));
  u.searchParams.set("api_key", API_KEY);

  const res = await fetch(u);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status} ${res.statusText}: ${text}`);
  }
  return res.json();
}

function formatScoreboard(match) {
  const lines = [];
  const participants = match.info.participants;

  lines.push(`Game ID: ${match.metadata.matchId}`);
  lines.push(
    "Player".padEnd(20) +
      "Champion".padEnd(15) +
      "K".padStart(4) +
      "D".padStart(4) +
      "A".padStart(4),
  );
  lines.push("-".repeat(47));

  for (const p of participants) {
    lines.push(
      p.riotIdGameName.padEnd(20) +
        p.championName.padEnd(15) +
        String(p.kills).padStart(4) +
        String(p.deaths).padStart(4) +
        String(p.assists).padStart(4),
    );
  }

  lines.push(""); // blank line between games
  return lines.join("\n");
}

async function main() {
  const inputId = await rl.question("Enter Riot ID (RiotName#TagLine): ");
  rl.close();

  const [riotName, tagLine] = inputId.split("#");
  if (!riotName || !tagLine) {
    console.error("Invalid format. Use RiotName#TagLine");
    process.exit(1);
  }

  // 1. Account → PUUID
  const account = await riotFetch(
    `https://asia.api.riotgames.com/riot/account/v1/accounts/by-riot-id/${encodeURIComponent(
      riotName,
    )}/${encodeURIComponent(tagLine)}`,
  );

  // 2. Last 15 tourney matches
  const matchIds = await riotFetch(
    `https://sea.api.riotgames.com/lol/match/v5/matches/by-puuid/${account.puuid}/ids`,
    {
      type: "tourney",
      count: 15,
    },
  );

  if (matchIds.length === 0) {
    await fs.writeFile("results.txt", "No tournament matches found.\n");
    return;
  }

  const outputLines = [];

  for (const matchId of matchIds) {
    const match = await riotFetch(
      `https://sea.api.riotgames.com/lol/match/v5/matches/${matchId}`,
    );
    outputLines.push(formatScoreboard(match));
  }

  await fs.writeFile("results.txt", outputLines.join("\n"));
  console.log("Results written to results.txt");
}

main().catch((err) => {
  console.error("Error:", err.message);
  process.exit(1);
});
