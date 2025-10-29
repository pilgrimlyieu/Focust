/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,ts,tsx}"],
  daisyui: {
    base: true,
    logs: false,
    styled: true,
    themes: [
      {
        focust: {
          accent: "#f59e0b",
          "base-100": "#0f172a",
          "base-200": "#1e293b",
          "base-300": "#334155",
          "base-content": "#e2e8f0",
          error: "#ef4444",
          info: "#0ea5e9",
          neutral: "#3d4451",
          primary: "#2563eb",
          "primary-content": "#f8fafc",
          secondary: "#7c3aed",
          success: "#22c55e",
          warning: "#facc15",
        },
      },
      "light",
      "dark",
    ],
    utils: true,
  },
  plugins: [require("daisyui")],
  theme: {
    extend: {
      fontFamily: {
        display: [
          "Inter",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "sans-serif",
        ],
      },
    },
  },
};
