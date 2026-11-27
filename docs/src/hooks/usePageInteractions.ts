import { useEffect } from 'react';

type Cleanup = () => void;

export function usePageInteractions(dependencyKey: string) {
  useEffect(() => {
    const cleanups: Cleanup[] = [];
    const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    const tocLinks = [...document.querySelectorAll<HTMLAnchorElement>(".toc a[href^='#']")];
    if (tocLinks.length && 'IntersectionObserver' in window) {
      const map = new Map(tocLinks.map((link) => [link.getAttribute('href')?.slice(1) ?? '', link]));
      const spy = new IntersectionObserver((entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            tocLinks.forEach((link) => link.classList.remove('active'));
            map.get(entry.target.id)?.classList.add('active');
          }
        });
      }, { rootMargin: '-20% 0px -70% 0px' });
      map.forEach((_, id) => {
        const element = document.getElementById(id);
        if (element) spy.observe(element);
      });
      cleanups.push(() => spy.disconnect());
    }

    const reveals = [...document.querySelectorAll('.reveal')];
    if ('IntersectionObserver' in window && !reduceMotion) {
      const revealObserver = new IntersectionObserver((entries) => entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('in');
          revealObserver.unobserve(entry.target);
        }
      }), { threshold: 0.12 });
      reveals.forEach((element) => revealObserver.observe(element));
      cleanups.push(() => revealObserver.disconnect());
    } else {
      reveals.forEach((element) => element.classList.add('in'));
    }

    document.querySelectorAll<HTMLElement>('.card').forEach((card) => {
      const move = (event: PointerEvent) => {
        if (event.pointerType !== 'mouse') return;
        const rect = card.getBoundingClientRect();
        card.style.setProperty('--mx', `${event.clientX - rect.left}px`);
        card.style.setProperty('--my', `${event.clientY - rect.top}px`);
      };
      const leave = (event: PointerEvent) => {
        if (event.pointerType !== 'mouse') return;
        card.style.removeProperty('--mx');
        card.style.removeProperty('--my');
      };
      card.addEventListener('pointermove', move);
      card.addEventListener('pointerleave', leave);
      cleanups.push(() => {
        card.removeEventListener('pointermove', move);
        card.removeEventListener('pointerleave', leave);
      });
    });

    return () => cleanups.forEach((cleanup) => cleanup());
  }, [dependencyKey]);
}
