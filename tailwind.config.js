/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    relative: true,
    files: ["./templates/**/*.{html,js}", "./src/routes/**/*.{html,js}"],
  },
  theme: {
    extend: {},
  },
  plugins: [
    "@tailwindcss/typography",
    "@tailwindcss/forms",
    "@tailwindcss/aspect-ratio",
  ],
};
