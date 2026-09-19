/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.rs"],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "Iosevka",
          "Iosevka Web",
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "Consolas",
          "monospace"
        ],
        mono: [
          "Iosevka",
          "Iosevka Web",
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "Consolas",
          "monospace"
        ]
      }
    }
  },
  plugins: []
};
