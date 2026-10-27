// Offers the app's two themes, see marco-sparko.css: Light, and Dark, which is mdBook's Navy theme restyled.
(function () {
    for (const name of ["rust", "coal", "ayu"]) {
        const button = document.getElementById("mdbook-theme-" + name);
        if (button && button.parentElement) {
            button.parentElement.style.display = "none";
        }
    }
    const dark = document.getElementById("mdbook-theme-navy");
    if (dark) {
        dark.textContent = "Dark";
    }
})();
