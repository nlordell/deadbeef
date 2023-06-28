export type Address = string;
export type Bytes = string;

/**
 * Safe parameters for searching for vanity addresses.
 */
export interface Safe {
    proxyFactory: Address;
    proxyInitCode: Bytes;
    singleton: Address;
    fallbackHandler: Address;
    owners: Address[];
    threshold: number;
}

/**
 * Vanity Safe creation data.
 */
export interface Creation {
    creationAddress: Address;
    saltNonce: Bytes;
    transaction: Transaction;
}

/**
 * Vanity Safe transaction data.
 */
export interface Transaction {
    to: Address;
    calldata: Bytes;
}

/**
 * A worker for searching for a vanity Safe address for the specified
 * parameters and prefix.
 */
export declare class SearchWorker {
    constructor(safe: Safe, prefix: Bytes);
    wait(): Promise<Creation>;
    cancel(err?: Error): void;
}
