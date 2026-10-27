import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const testDirectory = dirname(fileURLToPath(import.meta.url));
const uiDirectory = dirname(testDirectory);
const stylesheet = readFileSync(join(uiDirectory, "src", "style.css"), "utf8");

describe("responsive and motion-safe presentation", () => {
  it.each(["light", "dark"])("keeps %s text and controls legible", (theme) => {
    const selector = theme === "dark" ? ':root[data-theme="dark"]' : ":root";
    const rules = stylesheet
      .slice(stylesheet.indexOf(`${selector} {`))
      .split("}")[0]!;
    const colors: Record<string, string> = Object.fromEntries(
      [...rules.matchAll(/--budna-([\w-]+): (#\w{6});/gu)].map(
        ([, name, color]) => [name, color!],
      ),
    );
    const luminance = (name: string) => {
      const channels = colors[name]!.slice(1)
        .match(/.{2}/gu)!
        .map((hex) => {
          const value = parseInt(hex, 16) / 255;
          return value <= 0.04045
            ? value / 12.92
            : ((value + 0.055) / 1.055) ** 2.4;
        });
      return (
        channels[0]! * 0.2126 + channels[1]! * 0.7152 + channels[2]! * 0.0722
      );
    };
    for (const [foreground, background, minimum] of [
      ["text", "background", 4.5],
      ["muted", "background", 4.5],
      ["muted", "surface", 4.5],
      ["badge-ink", "fill", 4.5],
      ["on-contrast", "contrast", 4.5],
      ["control-border", "background", 3],
    ] as const) {
      const values = [luminance(foreground), luminance(background)].sort(
        (a, b) => a - b,
      );
      expect(
        (values[1]! + 0.05) / (values[0]! + 0.05),
        `${foreground} on ${background}`,
      ).toBeGreaterThanOrEqual(minimum);
    }
  });

  it("defines compact mobile layouts and a reduced-motion mode", () => {
    expect(stylesheet).toContain("@media (max-width: 620px)");
    expect(stylesheet).toContain("grid-template-columns: 1fr");
    expect(stylesheet).toContain("@media (prefers-reduced-motion: reduce)");
  });
});
