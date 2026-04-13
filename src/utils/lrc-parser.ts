import type { LyricLine } from "@/types";

const LRC_PATTERN = /\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)/;

export function parseLrc(lrcText: string): LyricLine[] {
  const lines: LyricLine[] = [];

  for (const raw of lrcText.split("\n")) {
    const match = raw.trim().match(LRC_PATTERN);
    if (!match) continue;

    const minutes = parseInt(match[1], 10);
    const seconds = parseInt(match[2], 10);
    const millisStr = match[3].padEnd(3, "0").slice(0, 3);
    const millis = parseInt(millisStr, 10);
    const timeSec = minutes * 60 + seconds + millis / 1000;
    const text = match[4].trim();

    if (text) {
      lines.push({
        time: Math.round(timeSec * 1000) / 1000,
        text,
      });
    }
  }

  lines.sort((a, b) => a.time - b.time);
  return lines;
}

export function findCurrentLine(lines: LyricLine[], time: number): number {
  if (lines.length === 0) return -1;

  let lo = 0;
  let hi = lines.length - 1;

  while (lo <= hi) {
    const mid = Math.floor((lo + hi) / 2);
    if (lines[mid].time <= time) {
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }

  return hi;
}
