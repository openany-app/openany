import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { debounce } from '@oberflaeche/shared/debounce';

describe('debounce', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('ruft erst nach der Ruhephase und nur einmal auf', () => {
    let calls = 0;
    const fn = debounce(() => calls++, 100);

    fn(); fn(); fn();
    expect(calls).toBe(0);

    vi.advanceTimersByTime(99);
    expect(calls).toBe(0);

    vi.advanceTimersByTime(1);
    expect(calls).toBe(1);
  });

  it('übergibt die letzten Argumente', () => {
    const spy = vi.fn();
    const fn = debounce(spy, 50);

    fn('a');
    fn('b');
    vi.advanceTimersByTime(50);

    expect(spy).toHaveBeenCalledTimes(1);
    expect(spy).toHaveBeenCalledWith('b');
  });
});
