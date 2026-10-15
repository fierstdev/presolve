(() => {
  document.documentElement.classList.add("js");

  const SITE_LINKS = [
    ["Home", "/"],
    ["How it works", "/#how"],
    ["Documentation", "/docs/"],
    ["Proof", "/proof/"],
    ["Status", "/status/"],
    ["GitHub ↗", "https://github.com/fierstdev/presolve"],
  ];

  const DOC_GROUPS = [
    {
      label: "Start",
      links: [
        ["Documentation home", "/docs/"],
        ["Getting started", "/docs/getting-started/"],
      ],
    },
    {
      label: "Build & run",
      links: [
        ["CLI & workflows", "/docs/cli/"],
        ["Define your application", "/docs/application/"],
        ["Define an environment", "/docs/environment/"],
      ],
    },
    {
      label: "Understand",
      links: [
        ["Built releases", "/docs/releases/"],
        ["Capabilities", "/docs/capabilities/"],
        ["Architecture", "/docs/model/"],
      ],
    },
    {
      label: "Verify",
      links: [
        ["Portability proofs", "/docs/proofs/"],
        ["Current status", "/status/"],
      ],
    },
  ];

  const currentPath = normalizePath(window.location.pathname);

  function normalizePath(path) {
    if (!path || path === "/") return "/";
    return path.endsWith("/") ? path : `${path}/`;
  }

  function isCurrent(href) {
    if (href.startsWith("http") || href.includes("#")) return false;
    const normalized = normalizePath(href);
    if (normalized === "/docs/") return currentPath.startsWith("/docs/");
    return normalized === currentPath;
  }

  function makeLink(label, href) {
    const link = document.createElement("a");
    link.href = href;
    link.textContent = label;
    if (isCurrent(href)) link.setAttribute("aria-current", "page");
    return link;
  }

  function markDesktopNavigation() {
    document.querySelectorAll(".nav-links a").forEach((link) => {
      const href = link.getAttribute("href");
      if (href && isCurrent(href)) link.setAttribute("aria-current", "page");
    });
  }

  function buildDocsGroups(container, compact = false) {
    DOC_GROUPS.forEach((group) => {
      const wrapper = document.createElement("div");
      wrapper.className = compact ? "mobile-menu-group" : "docs-nav-group";

      const label = document.createElement("span");
      label.className = compact ? "" : "docs-nav-label";
      label.textContent = group.label;
      wrapper.append(label);

      group.links.forEach(([text, href]) => wrapper.append(makeLink(text, href)));
      container.append(wrapper);
    });
  }

  function buildMobileMenu() {
    const shell = document.querySelector(".site-header .nav-shell");
    if (!shell || shell.querySelector(".mobile-menu")) return;

    const details = document.createElement("details");
    details.className = "mobile-menu";

    const summary = document.createElement("summary");
    summary.textContent = "Menu";
    summary.setAttribute("aria-label", "Open site navigation");
    details.append(summary);

    const panel = document.createElement("nav");
    panel.className = "mobile-menu-panel";
    panel.setAttribute("aria-label", "Mobile navigation");

    const primary = document.createElement("div");
    primary.className = "mobile-menu-primary";
    SITE_LINKS.forEach(([label, href]) => primary.append(makeLink(label, href)));
    panel.append(primary);

    if (currentPath.startsWith("/docs/")) buildDocsGroups(panel, true);

    details.append(panel);
    shell.append(details);

    panel.addEventListener("click", (event) => {
      if (event.target.closest("a")) details.removeAttribute("open");
    });

    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape") details.removeAttribute("open");
    });

    document.addEventListener("click", (event) => {
      if (details.open && !details.contains(event.target)) details.removeAttribute("open");
    });
  }

  function enhanceDocsTree() {
    const nav = document.querySelector(".docs-nav");
    if (!nav) return;

    nav.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = "Documentation";
    nav.append(title);
    buildDocsGroups(nav);

    const layout = document.querySelector(".docs-layout");
    if (!layout) return;

    const mobileTree = document.createElement("details");
    mobileTree.className = "docs-mobile-tree";
    const summary = document.createElement("summary");
    summary.textContent = "Browse documentation";
    mobileTree.append(summary);

    const tree = document.createElement("nav");
    tree.setAttribute("aria-label", "Documentation navigation");
    buildDocsGroups(tree);
    mobileTree.append(tree);

    layout.insertBefore(mobileTree, layout.querySelector(".docs-content"));
  }

  function slugify(value) {
    return value
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "section";
  }

  function buildPageOutline() {
    const article = document.querySelector(".docs-content");
    const layout = document.querySelector(".docs-layout");
    if (!article || !layout) return;

    const sections = [...article.querySelectorAll(":scope > section[id]")];
    if (sections.length < 2) return;

    const entries = sections.map((section) => {
      const heading = section.querySelector(":scope > h2");
      if (!heading) return null;

      const children = [...section.querySelectorAll(":scope > h3")].map((h3, index) => {
        if (!h3.id) h3.id = `${section.id}-${slugify(h3.textContent)}-${index + 1}`;
        return { label: h3.textContent.trim(), href: `#${h3.id}` };
      });

      return {
        element: section,
        label: heading.textContent.trim(),
        href: `#${section.id}`,
        children,
      };
    }).filter(Boolean);

    const makeOutline = (className, label) => {
      const nav = document.createElement("nav");
      nav.className = className;
      nav.setAttribute("aria-label", label);

      const title = document.createElement("strong");
      title.textContent = "On this page";
      nav.append(title);

      const list = document.createElement("ol");
      entries.forEach((entry) => {
        const item = document.createElement("li");
        item.append(makeLink(entry.label, entry.href));

        if (entry.children.length) {
          const nested = document.createElement("ol");
          entry.children.forEach((child) => {
            const nestedItem = document.createElement("li");
            nestedItem.append(makeLink(child.label, child.href));
            nested.append(nestedItem);
          });
          item.append(nested);
        }
        list.append(item);
      });
      nav.append(list);
      return nav;
    };

    layout.append(makeOutline("docs-toc", "On this page"));

    const mobileOutline = document.createElement("details");
    mobileOutline.className = "docs-page-menu";
    const summary = document.createElement("summary");
    summary.textContent = "On this page";
    mobileOutline.append(summary);
    const mobileNav = makeOutline("docs-page-menu-inner", "On this page");
    mobileNav.querySelector("strong")?.remove();
    mobileOutline.append(mobileNav);
    article.insertBefore(mobileOutline, article.firstChild);

    mobileOutline.addEventListener("click", (event) => {
      if (event.target.closest("a")) mobileOutline.removeAttribute("open");
    });

    let ticking = false;
    const updateCurrentSection = () => {
      ticking = false;
      const threshold = 160;
      let active = entries[0];
      for (const entry of entries) {
        if (entry.element.getBoundingClientRect().top <= threshold) active = entry;
        else break;
      }

      document.querySelectorAll(".docs-toc a[aria-current='location'], .docs-page-menu-inner a[aria-current='location']")
        .forEach((link) => link.removeAttribute("aria-current"));

      const escapedId = CSS.escape(active.element.id);
      document.querySelectorAll(`.docs-toc a[href='#${escapedId}'], .docs-page-menu-inner a[href='#${escapedId}']`)
        .forEach((link) => link.setAttribute("aria-current", "location"));
    };

    const requestUpdate = () => {
      if (ticking) return;
      ticking = true;
      window.requestAnimationFrame(updateCurrentSection);
    };

    window.addEventListener("scroll", requestUpdate, { passive: true });
    window.addEventListener("resize", requestUpdate);
    window.addEventListener("hashchange", requestUpdate);
    updateCurrentSection();
  }

  markDesktopNavigation();
  buildMobileMenu();
  enhanceDocsTree();
  buildPageOutline();
})();
