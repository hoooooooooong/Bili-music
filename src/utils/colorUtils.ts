export const PRESET_COLORS = [
  { name: "Bilibili粉", color: "#fb7299" },
  { name: "海洋蓝", color: "#23ade5" },
  { name: "皇家紫", color: "#9b59b6" },
  { name: "翡翠绿", color: "#2ecc71" },
  { name: "日落橙", color: "#e67e22" },
  { name: "中国红", color: "#e74c3c" },
  { name: "青色", color: "#1abc9c" },
];

export function hexToHSL(hex: string): { h: number; s: number; l: number } {
  let r = parseInt(hex.slice(1, 3), 16) / 255;
  let g = parseInt(hex.slice(3, 5), 16) / 255;
  let b = parseInt(hex.slice(5, 7), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;

  if (max === min) return { h: 0, s: 0, l: Math.round(l * 100) };

  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

  let h = 0;
  switch (max) {
    case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
    case g: h = ((b - r) / d + 2) / 6; break;
    case b: h = ((r - g) / d + 4) / 6; break;
  }

  return {
    h: Math.round(h * 360),
    s: Math.round(s * 100),
    l: Math.round(l * 100),
  };
}

export function hslToHex(h: number, s: number, l: number): string {
  s /= 100;
  l /= 100;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
    return Math.round(255 * color).toString(16).padStart(2, "0");
  };
  return `#${f(0)}${f(8)}${f(4)}`;
}

export function deriveThemeColors(hex: string): {
  hover: string;
  pressed: string;
  light: string;
} {
  const { h, s, l } = hexToHSL(hex);
  return {
    hover: hslToHex(h, s, Math.min(l + 5, 85)),
    pressed: hslToHex(h, s, Math.max(l - 8, 15)),
    light: `rgba(${parseInt(hex.slice(1, 3), 16)}, ${parseInt(hex.slice(3, 5), 16)}, ${parseInt(hex.slice(5, 7), 16)}, 0.1)`,
  };
}
