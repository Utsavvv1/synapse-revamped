/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        card: "hsl(var(--card))",
        'card-foreground': "hsl(var(--card-foreground))",
        popover: "hsl(var(--popover))",
        'popover-foreground': "hsl(var(--popover-foreground))",
        primary: "hsl(var(--primary))",
        'primary-foreground': "hsl(var(--primary-foreground))",
        secondary: "hsl(var(--secondary))",
        'secondary-foreground': "hsl(var(--secondary-foreground))",
        muted: "hsl(var(--muted))",
        'muted-foreground': "hsl(var(--muted-foreground))",
        accent: "hsl(var(--accent))",
        'accent-foreground': "hsl(var(--accent-foreground))",
        destructive: "hsl(var(--destructive))",
        'destructive-foreground': "hsl(var(--destructive-foreground))",
        // Synapse custom colors
        'synapse-dark': "hsl(var(--synapse-dark))",
        'synapse-dark-alt': "hsl(var(--synapse-dark-alt))",
        'synapse-accent': "hsl(var(--synapse-accent))",
        'synapse-glass': "hsl(var(--synapse-glass))",
        'synapse-text-light': "hsl(var(--synapse-text-light))",
      },
      fontFamily: {
        instrument: ["'Instrument Serif'", 'serif'],
        dmserif: ["'DM Serif Text'", 'serif'],
        wix: ["'Wix Madefor Text'", 'sans-serif'],
      },
      letterSpacing: {
        tightest: '-0.07em', // -7%
        tighter: '-0.03em',  // -3%
        box: '-0.01em',      // -1%
        normal: '0em',       // 0%
      },
    },
  },
  plugins: [
    require('tailwindcss-animate'),
  ],
}; 