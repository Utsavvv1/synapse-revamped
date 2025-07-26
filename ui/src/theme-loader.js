// simple dynamic theme injector
export async function applyTheme(name /* "light" or "dark" */) {
    const res = await fetch(`../../themes/theme-${name}.json`);
    console.log('fetch status:', res.status, res.url);
    if (!res.ok) throw new Error(`cannot load theme-${name}.json`);
    const theme = await res.json();
  console.log('loaded theme data: ', theme);
  console.log('name: ', name);
    const root = document.documentElement;
    Object.entries(theme).forEach(([key, val]) => {
      root.style.setProperty(`--${key}`, val);
    });
  
    if (name === "dark") root.classList.add("dark");
    else root.classList.remove("dark");
  }
  
  // immediately apply stored theme (or default to light)
  applyTheme("black")
    .catch(console.error);
  