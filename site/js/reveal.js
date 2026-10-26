// Fallback for browsers without CSS scroll-driven animations: reveal
// elements once as they enter the viewport.

export function initReveal() {
  if (CSS.supports("animation-timeline: view()")) return;
  document.documentElement.classList.add("js");
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (e.isIntersecting) {
          e.target.classList.add("is-visible");
          io.unobserve(e.target);
        }
      }
    },
    { rootMargin: "0px 0px -10% 0px" },
  );
  for (const el of document.querySelectorAll(".reveal")) io.observe(el);
}
