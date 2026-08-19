(function () {
  function fileName(path) {
    var parts = path.split("/");
    return parts[parts.length - 1] || "Start.html";
  }

  function markSelectedNavigation() {
    var toc = document.getElementById("toc") || document.querySelector(".toc");
    if (!toc) return;

    var current = fileName(window.location.pathname);
    var hash = window.location.hash;
    var links = toc.getElementsByTagName("a");

    for (var index = 0; index < links.length; index += 1) {
      var link = links[index];
      var selected = link.hash
        ? hash && link.hash === hash
        : fileName(link.pathname) === current;
      if (!selected) continue;

      link.className = link.className ? link.className + " selected" : "selected";
      if (link.parentNode && link.parentNode.tagName.toLowerCase() === "li") {
        link.parentNode.className = link.parentNode.className
          ? link.parentNode.className + " selected"
          : "selected";
      }
    }
  }

  function darkSourceFor(src) {
    return src.replace(/(\.[a-z0-9]+)(\?.*)?$/i, "-dark$1$2");
  }

  function useDarkTheme() {
    var explicitTheme = document.documentElement.getAttribute("data-theme");
    if (explicitTheme) return explicitTheme === "dark";

    return window.matchMedia &&
      window.matchMedia("(prefers-color-scheme: dark)").matches;
  }

  function swapHelpBookImages() {
    var images = document.querySelectorAll(".shot img");
    var dark = useDarkTheme();

    for (var index = 0; index < images.length; index += 1) {
      var image = images[index];
      if (image.className.indexOf("light-only") !== -1 || image.className.indexOf("dark-only") !== -1) {
        continue;
      }

      var lightSrc = image.getAttribute("data-light-src") || image.getAttribute("src");
      var darkSrc = image.getAttribute("data-dark-src") || darkSourceFor(lightSrc);
      image.setAttribute("data-light-src", lightSrc);
      image.setAttribute("src", dark ? darkSrc : lightSrc);
    }
  }

  function installLightbox() {
    var images = document.querySelectorAll(".show-img img, .shot img");
    if (!images.length) return;

    var lightbox = document.createElement("div");
    lightbox.className = "lightbox";
    lightbox.setAttribute("aria-hidden", "true");
    lightbox.setAttribute("role", "button");
    lightbox.setAttribute("tabindex", "-1");
    lightbox.setAttribute("title", "Close enlarged screenshot");

    var lightboxImage = document.createElement("img");
    lightboxImage.alt = "Enlarged screenshot";
    lightbox.appendChild(lightboxImage);
    document.body.appendChild(lightbox);

    function close() {
      lightbox.className = "lightbox";
      lightbox.setAttribute("aria-hidden", "true");
      lightboxImage.removeAttribute("src");
    }

    for (var index = 0; index < images.length; index += 1) {
      images[index].addEventListener("click", function () {
        var src = this.currentSrc || this.getAttribute("src") || this.src;
        if (!src) return;
        lightboxImage.src = src;
        lightbox.className = "lightbox open";
        lightbox.setAttribute("aria-hidden", "false");
        lightbox.focus();
      });
    }

    lightbox.addEventListener("click", close);
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape") close();
    });
  }

  markSelectedNavigation();
  swapHelpBookImages();
  installLightbox();

  if (window.matchMedia) {
    var query = window.matchMedia("(prefers-color-scheme: dark)");
    if (query.addEventListener) {
      query.addEventListener("change", swapHelpBookImages);
    } else if (query.addListener) {
      query.addListener(swapHelpBookImages);
    }
  }
})();
