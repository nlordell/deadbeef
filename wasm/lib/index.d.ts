export type Address = string;
export type Bytes = string;

/**
 * Safe configuration for searching for vanity addresses.
 */
export interface Configuration {
  proxyFactory: Address;
  proxyInitCode: Bytes;
  singleton: Address;
  owners: Address[];
  threshold: number;
  safeToL2Setup?: Address;
  l2Singleton?: Address;
  fallbackHandler?: Address;
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
export declare class DeadbeefWorker {
  constructor(config: Configuration, prefix: Bytes);
  wait(): Promise<Creation>;
  cancel(err?: Error): void;
}
