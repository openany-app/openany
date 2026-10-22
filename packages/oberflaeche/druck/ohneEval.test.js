import { describe, it, expect } from 'vitest';
import { funktionOhneEval } from './ohneEval';

describe('funktionOhneEval', () => {
  it('bildet die Konstanten nach, die der Satz-Kern verlangt', () => {
    expect(funktionOhneEval('return 0')()).toBe(0);
    expect(funktionOhneEval('return true')()).toBe(true);
    expect(funktionOhneEval('return null')()).toBe(null);
    expect(funktionOhneEval('return -1.5')()).toBe(-1.5);
    expect(funktionOhneEval('return this')()).toBe(globalThis);
  });

  it('bildet die Identität nach (Function("path", "return path"))', () => {
    expect(funktionOhneEval('path', 'return path')('/main.typ')).toBe('/main.typ');
  });

  it('bildet Platzhalter nach, die einen festen Fehler werfen', () => {
    const f = funktionOhneEval("throw new Error('Dummy AccessModel, please initialize compiler with withAccessModel()')");
    expect(f).toThrow('Dummy AccessModel');
  });

  it('lehnt alles andere ab, wie die CSP es täte', () => {
    expect(() => funktionOhneEval('alert(1)')).toThrow(EvalError);
    expect(() => funktionOhneEval('m', 'return import(m)')).toThrow(EvalError);
    expect(() => funktionOhneEval('return document')).toThrow(EvalError);
    expect(() => funktionOhneEval('a', 'return b')).toThrow(EvalError);
    expect(() => funktionOhneEval("throw new Error('x' + y)")).toThrow(EvalError);
  });
});
