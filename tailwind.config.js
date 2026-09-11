/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        accent: {
          DEFAULT: "#818cf8",
          hover: "#a78bfa",
        },
        brand: {
          400: "#a78bfa",
          500: "#818cf8",
          600: "#6366f1",
          700: "#4f46e5",
        },
        surface: {
          DEFAULT: "rgba(15, 23, 42, 0.72)",
          raised: "rgba(30, 41, 59, 0.55)",
          inset: "rgba(2, 6, 23, 0.55)",
          border: "rgba(148, 163, 184, 0.16)",
          highlight: "rgba(255, 255, 255, 0.08)",
        },
      },
      fontFamily: {
        sans: [
          "ui-sans-serif",
          "Segoe UI Variable",
          "Segoe UI",
          "system-ui",
          "sans-serif",
        ],
      },
      boxShadow: {
        glass: "0 18px 50px -20px rgba(2, 6, 23, 0.75)",
        glow: "0 0 0 1px rgba(129, 140, 248, 0.18), 0 10px 28px -12px rgba(99, 102, 241, 0.55)",
        "glow-lg": "0 0 0 1px rgba(167, 139, 250, 0.28), 0 16px 40px -12px rgba(99, 102, 241, 0.55)",
      },
      keyframes: {
        "ambient-drift": {
          "0%, 100%": { transform: "translate3d(0, 0, 0) scale(1)" },
          "50%": { transform: "translate3d(4%, -3%, 0) scale(1.08)" },
        },
        shimmer: {
          "0%": { transform: "translateX(-120%)" },
          "100%": { transform: "translateX(220%)" },
        },
        "status-in": {
          "0%": { opacity: "0", transform: "translateY(6px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
      animation: {
        ambient: "ambient-drift 18s ease-in-out infinite",
        shimmer: "shimmer 1.8s ease-in-out infinite",
        "status-in": "status-in 280ms ease-out",
      },
    },
  },
  plugins: [],
};
