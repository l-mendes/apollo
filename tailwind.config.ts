import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{vue,ts}"],
  theme: {
    extend: {
      colors: {
        apollo: {
          app: {
            shell: "#05070b",
            panel: "#0b0f14",
            sidebar: "#090d12",
            header: "#0e131a",
            card: "#10161f",
            border: "rgb(255 255 255 / 0.08)",
            hover: "rgb(255 255 255 / 0.04)",
            selected: "rgb(37 99 235 / 0.18)",
            selectedBorder: "rgb(96 165 250 / 0.34)",
            accent: "#60a5fa",
            muted: "#94a3b8",
            subtle: "#cbd5e1"
          },
          tray: {
            shell: "rgb(9 13 20 / 0.94)",
            panel: "rgb(255 255 255 / 0.04)",
            hover: "rgb(255 255 255 / 0.08)",
            divider: "rgb(255 255 255 / 0.10)",
            active: "rgb(251 146 60 / 0.18)",
            danger: "rgb(239 68 68 / 0.12)",
            tooltip: "#1e293b",
            idle: "#64748b",
            online: "#6ee7b7"
          }
        },
        ember: {
          50: "#fff7ed",
          100: "#ffedd5",
          400: "#fb923c",
          500: "#f97316"
        },
        slateNight: "#0f172a"
      },
      boxShadow: {
        trayStatus: "0 0 14px rgba(110, 231, 183, 0.55)",
        tray: "0 24px 64px rgba(15, 23, 42, 0.28)"
      },
      fontFamily: {
        sans: ["IBM Plex Sans", "Segoe UI", "sans-serif"]
      }
    }
  },
  plugins: []
} satisfies Config;
