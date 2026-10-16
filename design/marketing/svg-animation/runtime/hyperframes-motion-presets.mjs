const clamp = value => Math.max(0, Math.min(1, value));

export function sampleHyperFrameProgress(time, start, end) {
  if (!Number.isFinite(time) || !Number.isFinite(start) || !Number.isFinite(end) || end <= start) {
    throw new RangeError('HyperFrames progress requires finite time and an increasing interval');
  }
  const value = clamp((time - start) / (end - start));
  return value * value * (3 - 2 * value);
}

export function sampleHyperFrameStagger(time, start, index, spacing, duration) {
  return sampleHyperFrameProgress(time, start + index * spacing, start + index * spacing + duration);
}

export function sampleHyperFrameCarry(time, enterStart, enterEnd, exitStart, exitEnd) {
  return sampleHyperFrameProgress(time, enterStart, enterEnd)
    * (1 - sampleHyperFrameProgress(time, exitStart, exitEnd));
}

export function sampleHyperFramePulse(time, start, rise, fall) {
  return sampleHyperFrameCarry(time, start, start + rise, start + rise, start + rise + fall);
}

export function sampleHyperFrameKeyframes(time, keys) {
  if (keys.length < 2 || keys.some((key, index) => !Number.isFinite(key[0]) || (index && key[0] <= keys[index - 1][0]))) {
    throw new RangeError('HyperFrames keyframes require at least two strictly increasing times');
  }
  if (time <= keys[0][0]) return keys[0].slice(1);
  if (time >= keys.at(-1)[0]) return keys.at(-1).slice(1);
  const right = keys.findIndex(key => key[0] > time);
  const previous = keys[right - 1];
  const next = keys[right];
  const amount = sampleHyperFrameProgress(time, previous[0], next[0]);
  return previous.slice(1).map((value, index) => value + (next[index + 1] - value) * amount);
}

// Damped-spring step response (textbook second-order system: natural frequency omega in rad/s,
// damping ratio zeta), 0 before `start` and settling at 1. Pattern credit: the spring presets of
// github.com/Barty-Bart/motion-graphics (MIT); this is a clean-room reimplementation, no code copied.
// zeta < 1 wobbles past 1 once and settles; zeta >= 1 uses the critically damped curve (never above 1).
export function sampleHyperFrameSpring(time, start, omega, zeta) {
  if (!Number.isFinite(time) || !Number.isFinite(start) || !Number.isFinite(omega) || omega <= 0
    || !Number.isFinite(zeta) || zeta < 0) {
    throw new RangeError('HyperFrames spring requires finite time and start, omega > 0 and zeta >= 0');
  }
  const t = time - start;
  if (t <= 0) return 0;
  if (zeta >= 1) return 1 - Math.exp(-omega * t) * (1 + omega * t);
  const damped = omega * Math.sqrt(1 - zeta * zeta);
  return 1 - Math.exp(-zeta * omega * t) * (Math.cos(damped * t) + (zeta * omega / damped) * Math.sin(damped * t));
}

// [omega, zeta] pairs; damping stays in the 0.8-0.95 band (a slight wobble, then a fast settle).
export const HYPERFRAMES_SPRING = Object.freeze({ morph: [15, 0.84], fast: [27, 0.86], slow: [12.5, 0.9], soft: [10, 0.95] });

// A GSAP-compatible ease that plays the spring over a tween of `durationS` seconds. The ends are exact
// (0 and 1) so a tween always lands on its authored value; the in-between samples the spring.
export function springEase(durationS, [omega, zeta]) {
  if (!Number.isFinite(durationS) || durationS <= 0) throw new RangeError('HyperFrames spring ease requires a positive duration');
  sampleHyperFrameSpring(0, 0, omega, zeta);
  return progress => (progress <= 0 ? 0 : progress >= 1 ? 1 : sampleHyperFrameSpring(progress * durationS, 0, omega, zeta));
}

const SPRING_PROPS = new Set(['x', 'y', 'scale', 'scaleX', 'scaleY', 'rotation', 'rotationX', 'rotationY', 'rotationZ']);
const TIMING_KEYS = ['duration', 'stagger'];
const pick = (vars, keep) => Object.fromEntries(Object.entries(vars).filter(([key]) => keep(key)));
const unitOpacity = { opacity: value => Math.min(1, Math.max(0, value)) };

// Entrance tween for a beat kind. Kit motion "tween" (or none) hands the authored vars to GSAP as they
// are. Kit motion "spring" moves the transform props on a spring (the fast preset for staggers) and
// keeps every other prop on the authored ease: a spring passes 1, and opacity must never exceed 1
// (an authored back.out ease also passes 1, so the spring path clamps opacity to 0..1 as well).
export function enterWithMotion(timeline, targets, vars, position, options, fromVars) {
  if (options.motion !== 'spring') {
    return fromVars ? timeline.fromTo(targets, fromVars, vars, position) : timeline.from(targets, vars, position);
  }
  const duration = vars.duration ?? 0.5;
  const ease = springEase(duration, vars.stagger === undefined ? HYPERFRAMES_SPRING.morph : HYPERFRAMES_SPRING.fast);
  const isSpring = key => SPRING_PROPS.has(key);
  const isRest = key => !isSpring(key) && !TIMING_KEYS.includes(key) && key !== 'ease';
  const timing = pick(vars, key => TIMING_KEYS.includes(key));
  for (const [keep, tweenEase] of [[isSpring, ease], [isRest, vars.ease]]) {
    const to = pick(vars, keep);
    if (!Object.keys(to).length) continue;
    const tween = { ...to, ...timing, ...(tweenEase === undefined ? {} : { ease: tweenEase }),
      ...('opacity' in to ? { modifiers: unitOpacity } : {}) };
    if (fromVars) timeline.fromTo(targets, pick(fromVars, keep), tween, position);
    else timeline.from(targets, tween, position);
  }
  return timeline;
}
