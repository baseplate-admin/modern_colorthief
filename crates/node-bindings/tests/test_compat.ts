// Cross-runtime test compatibility layer
// Maps Vitest-style APIs to native test runners for Deno/Bun/Node

const isDeno = typeof (globalThis as any).Deno !== 'undefined';
const isBun = typeof (globalThis as any).Bun !== 'undefined';
const isVitest = typeof (globalThis as any).__vitest_worker__ !== 'undefined';
const runtime: 'deno' | 'bun' | 'vitest' | 'node' = isDeno ? 'deno' : isBun ? 'bun' : isVitest ? 'vitest' : 'node';

// Load vitest module at module init time (top-level await)
// This only runs when NOT in Deno (Deno doesn't have vitest)
let vitestDescribe: any, vitestIt: any, vitestExpect: any;
let vitestBeforeAll: any, vitestBeforeEach: any, vitestAfterAll: any, vitestAfterEach: any;

if (!isDeno) {
    try {
        const vitest = await import('vitest');
        vitestDescribe = vitest.describe;
        vitestIt = vitest.it;
        vitestExpect = vitest.expect;
        vitestBeforeAll = vitest.beforeAll;
        vitestBeforeEach = vitest.beforeEach;
        vitestAfterAll = vitest.afterAll;
        vitestAfterEach = vitest.afterEach;
    } catch {
        // Fall back to globals (Bun)
        vitestDescribe = (globalThis as any).describe;
        vitestIt = (globalThis as any).it;
        vitestExpect = (globalThis as any).expect;
        vitestBeforeAll = (globalThis as any).beforeAll;
        vitestBeforeEach = (globalThis as any).beforeEach;
        vitestAfterAll = (globalThis as any).afterAll;
        vitestAfterEach = (globalThis as any).afterEach;
    }
}

// ---------------------------------------------------------------------------
// Assertion helpers
// ---------------------------------------------------------------------------

class AssertionError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'AssertionError';
    }
}

// ---------------------------------------------------------------------------
// expect() matcher - Vitest-compatible subset
// ---------------------------------------------------------------------------

class ChainedAssertion<T> {
    private actual: T;
    private isNot: boolean;
    private promise: boolean;
    private resolvedValue?: T;

    constructor(actual: T | Promise<T>, isNot: boolean) {
        this.isNot = isNot;
        this.promise = actual instanceof Promise;
        this.actual = actual as T;
    }

    private async unwrap(): Promise<T> {
        if (this.promise && this.resolvedValue === undefined) {
            this.resolvedValue = await this.actual;
        }
        return this.resolvedValue !== undefined ? this.resolvedValue : this.actual;
    }

    private check(passed: boolean, expected: string, actual?: string): void {
        const realPassed = this.isNot ? !passed : passed;
        if (!realPassed) {
            const msg = this.isNot
                ? `Expected NOT ${expected}`
                : `Expected ${expected}${actual ? ` but got ${actual}` : ''}`;
            throw new AssertionError(msg);
        }
    }

    private async asyncCheck(passed: boolean, expected: string, actual?: string): Promise<void> {
        const realPassed = this.isNot ? !passed : passed;
        if (!realPassed) {
            const msg = this.isNot
                ? `Expected NOT ${expected}`
                : `Expected ${expected}${actual ? ` but got ${actual}` : ''}`;
            throw new AssertionError(msg);
        }
    }

    // Sync matchers (for non-promise values)
    toBe(expected: unknown): void {
        const val = this.actual;
        this.check(Object.is(val, expected), `toBe ${JSON.stringify(expected)}`, JSON.stringify(val));
    }

    toEqual(expected: unknown): void {
        const val = this.actual;
        this.check(JSON.stringify(val) === JSON.stringify(expected),
            `toEqual ${JSON.stringify(expected)}`, JSON.stringify(val));
    }

    toStrictEqual(expected: unknown): void {
        this.toEqual(expected);
    }

    toBeTruthy(): void {
        this.check(!!this.actual, 'toBe truthy');
    }

    toBeFalsy(): void {
        this.check(!this.actual, 'toBe falsy');
    }

    toBeDefined(): void {
        this.check(this.actual !== undefined && this.actual !== null, 'toBe defined');
    }

    toBeUndefined(): void {
        this.check(this.actual === undefined, 'toBe undefined');
    }

    toBeNull(): void {
        this.check(this.actual === null, 'toBe null');
    }

    toBeNaN(): void {
        this.check(Number.isNaN(this.actual), 'toBe NaN');
    }

    toBeGreaterThan(expected: number): void {
        const val = this.actual as unknown as number;
        this.check((val as number) > expected, `toBe greater than ${expected}`);
    }

    toBeLessThan(expected: number): void {
        const val = this.actual as unknown as number;
        this.check((val as number) < expected, `toBe less than ${expected}`);
    }

    toBeGreaterThanOrEqual(expected: number): void {
        const val = this.actual as unknown as number;
        this.check((val as number) >= expected, `toBe >= ${expected}`);
    }

    toBeLessThanOrEqual(expected: number): void {
        const val = this.actual as unknown as number;
        this.check((val as number) <= expected, `toBe <= ${expected}`);
    }

    toHaveLength(expected: number): void {
        const val = this.actual as unknown as { length: number };
        this.check((val as { length: number }).length === expected, `to have length ${expected}`);
    }

    toHaveSize(expected: number): void {
        this.toHaveLength(expected);
    }

    toContain(expected: unknown): void {
        const val = this.actual as unknown as unknown[];
        this.check((val as unknown[]).includes(expected as any), `to contain ${JSON.stringify(expected)}`);
    }

    toBeInstanceOf(expected: any): void {
        this.check(this.actual instanceof expected, `toBe instance of ${expected.name}`);
    }

    toThrow(expected?: any): void {
        const fn = this.actual as () => unknown;
        let thrown: unknown;
        try {
            fn();
        } catch (e) {
            thrown = e;
        }
        if (expected) {
            this.check(thrown instanceof expected, `toThrow ${expected.name}`);
        } else {
            this.check(thrown !== undefined, 'toThrow');
        }
    }

    toThrowError(expected?: any): void {
        this.toThrow(expected);
    }

    // Async versions
    async toBeAsync(expected: unknown): Promise<void> {
        const val = await this.unwrap();
        await this.asyncCheck(Object.is(val, expected), `toBe ${JSON.stringify(expected)}`, JSON.stringify(val));
    }

    async toEqualAsync(expected: unknown): Promise<void> {
        const val = await this.unwrap();
        await this.asyncCheck(JSON.stringify(val) === JSON.stringify(expected),
            `toEqual ${JSON.stringify(expected)}`, JSON.stringify(val));
    }

    async toBeTruthyAsync(): Promise<void> {
        const val = await this.unwrap();
        await this.asyncCheck(!!val, 'toBe truthy');
    }

    async toBeFalsyAsync(): Promise<void> {
        const val = await this.unwrap();
        await this.asyncCheck(!val, 'toBe falsy');
    }

    async toBeDefinedAsync(): Promise<void> {
        const val = await this.unwrap();
        await this.asyncCheck(val !== undefined && val !== null, 'toBe defined');
    }

    async toBeUndefinedAsync(): Promise<void> {
        const val = await this.unwrap();
        await this.asyncCheck(val === undefined, 'toBe undefined');
    }

    async toBeNullAsync(): Promise<void> {
        const val = await this.unwrap();
        await this.asyncCheck(val === null, 'toBe null');
    }

    async toBeGreaterThanAsync(expected: number): Promise<void> {
        const val = await this.unwrap() as unknown as number;
        await this.asyncCheck((val as number) > expected, `toBe greater than ${expected}`);
    }

    async toBeLessThanAsync(expected: number): Promise<void> {
        const val = await this.unwrap() as unknown as number;
        await this.asyncCheck((val as number) < expected, `toBe less than ${expected}`);
    }

    async toBeGreaterThanOrEqualAsync(expected: number): Promise<void> {
        const val = await this.unwrap() as unknown as number;
        await this.asyncCheck((val as number) >= expected, `toBe >= ${expected}`);
    }

    async toBeLessThanOrEqualAsync(expected: number): Promise<void> {
        const val = await this.unwrap() as unknown as number;
        await this.asyncCheck((val as number) <= expected, `toBe <= ${expected}`);
    }

    async toHaveLengthAsync(expected: number): Promise<void> {
        const val = await this.unwrap() as unknown as { length: number };
        await this.asyncCheck((val as { length: number }).length === expected, `to have length ${expected}`);
    }

    async toHaveSizeAsync(expected: number): Promise<void> {
        await this.toHaveLengthAsync(expected);
    }

    async toContainAsync(expected: unknown): Promise<void> {
        const val = await this.unwrap() as unknown as unknown[];
        await this.asyncCheck((val as unknown[]).includes(expected as any), `to contain ${JSON.stringify(expected)}`);
    }

    async toThrowAsync(expected?: any): Promise<void> {
        const fn = this.actual as () => Promise<unknown>;
        let thrown: unknown;
        try {
            await fn();
        } catch (e) {
            thrown = e;
        }
        if (expected) {
            await this.asyncCheck(thrown instanceof expected, `toThrow ${expected.name}`);
        } else {
            await this.asyncCheck(thrown !== undefined, 'toThrow');
        }
    }

    async toThrowErrorAsync(expected?: any): Promise<void> {
        await this.toThrowAsync(expected);
    }

    get not(): ChainedAssertion<T> {
        return new ChainedAssertion(this.actual, !this.isNot);
    }
}

// Smart expect that auto-detects async and routes to correct matchers
class SmartAssertion<T> {
    private chained: ChainedAssertion<T>;

    constructor(actual: T | Promise<T>) {
        this.chained = new ChainedAssertion(actual, false);
    }

    // Sync matchers
    toBe(expected: unknown): void { this.chained.toBe(expected); }
    toEqual(expected: unknown): void { this.chained.toEqual(expected); }
    toStrictEqual(expected: unknown): void { this.chained.toStrictEqual(expected); }
    toBeTruthy(): void { this.chained.toBeTruthy(); }
    toBeFalsy(): void { this.chained.toBeFalsy(); }
    toBeDefined(): void { this.chained.toBeDefined(); }
    toBeUndefined(): void { this.chained.toBeUndefined(); }
    toBeNull(): void { this.chained.toBeNull(); }
    toBeNaN(): void { this.chained.toBeNaN(); }
    toBeGreaterThan(expected: number): void { this.chained.toBeGreaterThan(expected); }
    toBeLessThan(expected: number): void { this.chained.toBeLessThan(expected); }
    toBeGreaterThanOrEqual(expected: number): void { this.chained.toBeGreaterThanOrEqual(expected); }
    toBeLessThanOrEqual(expected: number): void { this.chained.toBeLessThanOrEqual(expected); }
    toHaveLength(expected: number): void { this.chained.toHaveLength(expected); }
    toHaveSize(expected: number): void { this.chained.toHaveSize(expected); }
    toContain(expected: unknown): void { this.chained.toContain(expected); }
    toBeInstanceOf(expected: any): void { this.chained.toBeInstanceOf(expected); }
    toThrow(expected?: any): void { this.chained.toThrow(expected); }
    toThrowError(expected?: any): void { this.chained.toThrowError(expected); }

    get not(): SmartAssertion<T> {
        return new NotAssertion(this.chained);
    }
    get rejects(): RejectsAssertion {
        return new RejectsAssertion(this.chained);
    }
}

class NotAssertion extends SmartAssertion<any> {
    constructor(chained: ChainedAssertion<any>) {
        const negated = new ChainedAssertion(chained.actual, true);
        super(negated.actual);
        (this as any).chained = negated;
    }
}

class RejectsAssertion {
    private promise: Promise<any>;
    private isNot: boolean;

    constructor(chained: ChainedAssertion<any>) {
        this.promise = chained.actual as Promise<any>;
        this.isNot = chained.isNot;
    }

    private check(passed: boolean, expected: string, actual?: string): void {
        const realPassed = this.isNot ? !passed : passed;
        if (!realPassed) {
            const msg = this.isNot
                ? `Expected NOT ${expected}`
                : `Expected ${expected}${actual ? ` but got ${actual}` : ''}`;
            throw new AssertionError(msg);
        }
    }

    async toThrow(expected?: any): Promise<void> {
        let error: unknown;
        try {
            await this.promise;
        } catch (e) {
            error = e;
        }
        if (expected === undefined) {
            this.check(error !== undefined, 'toThrow');
        } else if (typeof expected === 'string') {
            this.check(
                error instanceof Error && error.message.includes(expected),
                `toThrow with message containing "${expected}"`
            );
        } else {
            this.check(error instanceof expected, `toThrow ${expected.name}`);
        }
    }

    async toThrowError(expected?: any): Promise<void> {
        await this.toThrow(expected);
    }

    get not(): RejectsAssertion {
        const negated = new RejectsAssertion({ actual: this.promise, isNot: !this.isNot } as any);
        return negated;
    }
}

// For Deno/Bun: use our custom expect. For Vitest: use vitest expect (passthrough).
let _expectFn: (actual: any) => SmartAssertion<any>;

if (runtime === 'vitest') {
    _expectFn = (actual: any) => vitestExpect(actual);
} else if (runtime === 'bun') {
    _expectFn = (actual: any) => vitestExpect(actual);
} else {
    _expectFn = (actual: any) => new SmartAssertion(actual);
}

export function expect<T>(actual: T | Promise<T>): SmartAssertion<T> {
    return _expectFn(actual);
}

// ---------------------------------------------------------------------------
// describe/it - mapped to Deno.test for Deno, passthrough for Vitest/Bun
// ---------------------------------------------------------------------------

type TestFn = () => void | Promise<void>;

let currentSuiteName = '';

export function describe(name: string, fn: TestFn): void {
    if (runtime === 'deno') {
        const prev = currentSuiteName;
        currentSuiteName = name;
        fn();
        currentSuiteName = prev;
    } else {
        vitestDescribe(name, fn);
    }
}

export const it: any = ((name: string, fnOrTimeout: TestFn | number, maybeFn?: TestFn) => {
    if (runtime === 'deno') {
        const fn = typeof fnOrTimeout === 'function' ? fnOrTimeout : maybeFn!;
        const testName = currentSuiteName ? `[${currentSuiteName}] ${name}` : name;
        Deno.test(testName, fn as () => Promise<void>);
    } else {
        if (typeof fnOrTimeout === 'function') {
            vitestIt(name, fnOrTimeout);
        } else {
            vitestIt(name, fnOrTimeout, maybeFn!);
        }
    }
}) as any;

it.skip = (name: string, fn: TestFn) => {
    if (runtime === 'deno') {
        const testName = currentSuiteName ? `[${currentSuiteName}] ${name}` : name;
        Deno.test({ name: testName, ignore: true, fn: fn as () => Promise<void> });
    } else {
        vitestIt.skip(name, fn);
    }
};

it.only = (name: string, fn: TestFn) => {
    if (runtime === 'deno') {
        const testName = currentSuiteName ? `[${currentSuiteName}] ${name}` : name;
        Deno.test({ name: testName, only: true, fn: fn as () => Promise<void> });
    } else {
        vitestIt.only(name, fn);
    }
};

// ---------------------------------------------------------------------------
// beforeAll / beforeEach / afterAll / afterEach
// ---------------------------------------------------------------------------

export function beforeAll(fn: () => void | Promise<void>, timeout?: number): void {
    if (runtime !== 'deno') {
        vitestBeforeAll(fn, timeout);
    }
}

export function beforeEach(fn: () => void | Promise<void>, timeout?: number): void {
    if (runtime !== 'deno') {
        vitestBeforeEach(fn, timeout);
    }
}

export function afterAll(fn: () => void | Promise<void>, timeout?: number): void {
    if (runtime !== 'deno') {
        vitestAfterAll(fn, timeout);
    }
}

export function afterEach(fn: () => void | Promise<void>, timeout?: number): void {
    if (runtime !== 'deno') {
        vitestAfterEach(fn, timeout);
    }
}

export function assertIfRanAtLeastOneTest(): void {
    // No-op - only relevant for Vitest
}
