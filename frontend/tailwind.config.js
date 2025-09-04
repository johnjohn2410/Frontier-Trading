/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        hl: {
          bg:'#0b0f1a', panel:'#0f1526', text:'#cbd5e1', muted:'#94a3b8',
          buy:'#22d3ee', sell:'#f43f5e', accent:'#06b6d4'
        }
      },
      boxShadow: {
        'neon-cyan': '0 0 .5rem rgba(34,211,238,.45), 0 0 1.25rem rgba(34,211,238,.25)',
        'neon-rose': '0 0 .5rem rgba(244,63,94,.45), 0 0 1.25rem rgba(244,63,94,.25)'
      },
      dropShadow: {
        'text-cyan': '0 0 10px rgba(34,211,238,.65)',
        'text-rose': '0 0 10px rgba(244,63,94,.65)'
      },
      fontFamily: { 
        ui: ['Inter', 'ui-sans-serif', 'system-ui'] 
      },
      borderRadius: { xl2: '1rem' }
    },
  },
  plugins: [],
}
