type Handlers = Record<string, (...args: any[]) => void>;

const handlers: Handlers = {};

export function on(event: string, fn: (...args: any[]) => void) {
  handlers[event] = fn;
}

export function off(event: string) {
  delete handlers[event];
}

export function emit(event: string, ...args: any[]) {
  handlers[event]?.(...args);
}
