import wasm from "./pkg/deadbeef_bg.wasm";
import init, { search } from "./pkg/deadbeef.js";

self.onmessage = async (message) => {
  const { safe, prefix } = message.data;
  try {
    await init(wasm);
    const creation = search(safe, prefix);
    self.postMessage({ creation });
  } catch (message) {
    self.postMessage({ creation: null, err: new Error(message) });
  } finally {
    self.close();
  }
};
