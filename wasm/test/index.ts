// @deno-types="../lib/index.d.ts"
import { DeadbeefWorker } from "../dist/index.js";

const safe = {
  proxyFactory: "0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67",
  proxyInitCode:
    "0x608060405234801561001057600080fd5b506040516101e63803806101e68339818101604052602081101561003357600080fd5b8101908080519060200190929190505050600073ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff1614156100ca576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260228152602001806101c46022913960400191505060405180910390fd5b806000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055505060ab806101196000396000f3fe608060405273ffffffffffffffffffffffffffffffffffffffff600054167fa619486e0000000000000000000000000000000000000000000000000000000060003514156050578060005260206000f35b3660008037600080366000845af43d6000803e60008114156070573d6000fd5b3d6000f3fea264697066735822122003d1488ee65e08fa41e58e888a9865554c535f2c77126a82cb4c0f917f31441364736f6c63430007060033496e76616c69642073696e676c65746f6e20616464726573732070726f7669646564",
  singleton: "0x41675C099F32341bf84BFc5382aF534df5C7461a",
  fallbackHandler: "0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99",
  owners: [
    "0x1111111111111111111111111111111111111111",
    "0x2222222222222222222222222222222222222222",
    "0x3333333333333333333333333333333333333333",
  ],
  threshold: 2,
};

function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

function delay(duration: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, duration));
}

Deno.test("computes Safe creation", async () => {
  const prefix = "0xabcd";

  const worker = new DeadbeefWorker(safe, prefix);
  const { creationAddress } = await worker.wait();

  assert(
    creationAddress.toLowerCase().startsWith(prefix),
    "Safe creation address does not start with prefix",
  );
});

Deno.test("cancel resolves to error", async () => {
  const longPrefix = "0x00112233445566778899aabbccddeeff";
  const worker = new DeadbeefWorker(safe, longPrefix);

  const creation = worker.wait();
  await Promise.race([creation, delay(100)]);

  worker.cancel();

  let errored;
  try {
    await creation;
    errored = false;
  } catch {
    errored = true;
  }

  assert(errored, "Safe worker promise did not reject when cancelled");
});
