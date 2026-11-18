(() => {
  const canvas = document.getElementById("demoCanvas");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  const trailSel = document.getElementById("trailSelect");
  const haloSel = document.getElementById("haloSelect");
  const colorSel = document.getElementById("colorSelect");
  const findBtn = document.getElementById("findBtn");

  let pts = [];
  let sparks = [];
  let bubbles = [];
  let hover = false;
  let mx = canvas.width / 2;
  let my = canvas.height / 2;
  let speed = 0;
  let findStart = -10;
  let lastT = performance.now();

  canvas.addEventListener("mouseenter", () => { hover = true; });
  canvas.addEventListener("mouseleave", () => { hover = false; });
  canvas.addEventListener("mousemove", (e) => {
    const r = canvas.getBoundingClientRect();
    const nx = ((e.clientX - r.left) / r.width) * canvas.width;
    const ny = ((e.clientY - r.top) / r.height) * canvas.height;
    const dist = Math.hypot(nx - mx, ny - my);
    speed = Math.min(1, speed * 0.7 + (dist / 45) * 0.3);
    mx = nx;
    my = ny;
  });
  findBtn.addEventListener("click", () => { findStart = performance.now() / 1000; });

  function getColor(t, alpha, nowS) {
    const mode = colorSel.value;
    if (mode === "rainbow") {
      const h = (nowS * 60 + t * 280) % 360;
      return `hsla(${h}, 85%, 62%, ${alpha})`;
    }
    if (mode === "gradient") {
      const r = Math.round(0 + 168 * t);
      const g = Math.round(245 + (85 - 245) * t);
      const b = Math.round(212 + (247 - 212) * t);
      return `rgba(${r},${g},${b},${alpha})`;
    }
    if (mode === "speed") {
      const h = 220 - 220 * Math.min(1, speed);
      return `hsla(${h}, 85%, 60%, ${alpha})`;
    }
    return `rgba(0, 245, 212, ${alpha})`;
  }

  function frame(nowMs) {
    const dt = Math.min(0.05, (nowMs - lastT) / 1000);
    lastT = nowMs;
    const nowS = nowMs / 1000;

    if (!hover) {
      const nx = canvas.width * 0.5 + Math.sin(nowS * 1.6) * canvas.width * 0.34;
      const ny = canvas.height * 0.5 + Math.sin(nowS * 3.2) * canvas.height * 0.30;
      speed = Math.min(1, Math.hypot(nx - mx, ny - my) / 28);
      mx = nx;
      my = ny;
    }

    pts.unshift({ x: mx, y: my, ms: nowMs });
    pts = pts.filter((p) => nowMs - p.ms < 520).slice(0, 120);

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const trail = trailSel.value;
    if (trail === "comet" || trail === "ribbon") {
      for (let i = pts.length - 1; i > 0; i--) {
        const a = pts[i], b = pts[i - 1];
        const t = (nowMs - b.ms) / 520;
        const w = trail === "comet" ? 12 * (1 - t * 0.85) : 9;
        ctx.strokeStyle = getColor(t, (1 - t) * 0.9, nowS);
        ctx.lineWidth = Math.max(1, w);
        ctx.lineCap = "round";
        ctx.beginPath();
        ctx.moveTo(a.x, a.y);
        ctx.lineTo(b.x, b.y);
        ctx.stroke();
      }
    } else if (trail === "beads") {
      for (let i = 0; i < pts.length; i += 3) {
        const p = pts[i], t = (nowMs - p.ms) / 520;
        ctx.fillStyle = getColor(t, 1 - t, nowS);
        ctx.beginPath();
        ctx.arc(p.x, p.y, Math.max(1, 6 * (1 - t * 0.7)), 0, Math.PI * 2);
        ctx.fill();
      }
    } else if (trail === "echoes") {
      for (let i = 0; i < pts.length; i += 5) {
        const p = pts[i], t = (nowMs - p.ms) / 520;
        ctx.strokeStyle = getColor(t, 1 - t, nowS);
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        ctx.arc(p.x, p.y, 4 + t * 18, 0, Math.PI * 2);
        ctx.stroke();
      }
    } else if (trail === "sparks") {
      if (sparks.length < 140) {
        for (let k = 0; k < 2; k++) {
          const ang = Math.random() * Math.PI * 2;
          const sp = 30 + Math.random() * 90;
          sparks.push({ x: mx, y: my, vx: Math.cos(ang) * sp, vy: Math.sin(ang) * sp - 25, age: 0, life: 0.45 + Math.random() * 0.35 });
        }
      }
      sparks = sparks.filter((s) => {
        s.age += dt;
        if (s.age >= s.life) return false;
        s.vy += 170 * dt;
        s.x += s.vx * dt;
        s.y += s.vy * dt;
        const t = s.age / s.life;
        ctx.strokeStyle = getColor(t, 1 - t, nowS);
        ctx.lineWidth = 2.2 * (1 - t * 0.5);
        ctx.beginPath();
        ctx.moveTo(s.x, s.y);
        ctx.lineTo(s.x - s.vx * 0.04, s.y - s.vy * 0.04);
        ctx.stroke();
        return true;
      });
    } else if (trail === "bubbles") {
      if (bubbles.length < 70 && Math.random() < 0.45) {
        bubbles.push({ x: mx + (Math.random() - 0.5) * 14, y: my, age: 0, life: 1.0, r: 4 + Math.random() * 5, ph: Math.random() * 6.28 });
      }
      bubbles = bubbles.filter((b) => {
        b.age += dt;
        if (b.age >= b.life) return false;
        b.y -= 32 * dt;
        const t = b.age / b.life;
        const bx = b.x + Math.sin(b.ph + b.age * 4) * 5;
        ctx.strokeStyle = getColor(t, 1 - t, nowS);
        ctx.lineWidth = 1.4;
        ctx.beginPath();
        ctx.arc(bx, b.y, b.r * (1 + t * 0.7), 0, Math.PI * 2);
        ctx.stroke();
        return true;
      });
    }

    const halo = haloSel.value;
    if (halo === "glow") {
      const g = ctx.createRadialGradient(mx, my, 0, mx, my, 52);
      g.addColorStop(0, getColor(0, 0.48, nowS));
      g.addColorStop(1, getColor(0.5, 0, nowS));
      ctx.fillStyle = g;
      ctx.beginPath();
      ctx.arc(mx, my, 52, 0, Math.PI * 2);
      ctx.fill();
    } else if (halo === "ring") {
      ctx.strokeStyle = getColor(0, 0.9, nowS);
      ctx.lineWidth = 2.2;
      ctx.beginPath();
      ctx.arc(mx, my, 24, 0, Math.PI * 2);
      ctx.stroke();
    } else if (halo === "pulse") {
      const w = (nowS % 1.4) / 1.4;
      ctx.strokeStyle = getColor(0, 0.85, nowS);
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(mx, my, 20 * (1 + 0.08 * Math.sin(nowS * 3.2)), 0, Math.PI * 2);
      ctx.stroke();
      ctx.strokeStyle = getColor(w, (1 - w) * 0.65, nowS);
      ctx.beginPath();
      ctx.arc(mx, my, 20 * (1 + w * 1.3), 0, Math.PI * 2);
      ctx.stroke();
    } else if (halo === "crosshair") {
      ctx.strokeStyle = getColor(0, 0.55, nowS);
      ctx.lineWidth = 1.2;
      ctx.beginPath();
      ctx.moveTo(0, my); ctx.lineTo(mx - 14, my);
      ctx.moveTo(mx + 14, my); ctx.lineTo(canvas.width, my);
      ctx.moveTo(mx, 0); ctx.lineTo(mx, my - 14);
      ctx.moveTo(mx, my + 14); ctx.lineTo(mx, canvas.height);
      ctx.stroke();
    } else if (halo === "target") {
      ctx.save();
      ctx.translate(mx, my);
      ctx.rotate(nowS * 0.6);
      ctx.strokeStyle = getColor(0, 0.9, nowS);
      ctx.lineWidth = 2.4;
      for (let i = 0; i < 4; i++) {
        ctx.rotate(Math.PI / 2);
        ctx.beginPath();
        ctx.moveTo(12, -24); ctx.lineTo(24, -24); ctx.lineTo(24, -12);
        ctx.stroke();
      }
      ctx.restore();
    }

    const findAge = nowS - findStart;
    if (findAge >= 0 && findAge < 1.4) {
      for (let i = 0; i < 3; i++) {
        const t = (findAge - i * 0.16) / 1.08;
        if (t <= 0 || t >= 1) continue;
        ctx.strokeStyle = getColor(t, (1 - t) * 0.95, nowS);
        ctx.lineWidth = Math.max(2, 6 * (1 - t));
        ctx.beginPath();
        ctx.arc(mx, my, 24 + t * 140, 0, Math.PI * 2);
        ctx.stroke();
      }
    }

    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
})();
