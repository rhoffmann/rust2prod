/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    relative: true,
    files: [
      "./templates/**/*.{html,htmx,js,ts}",
      "./src/routes/**/*.{html,htmx,js,ts}",
    ],
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
