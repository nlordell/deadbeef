export class SearchWorker {
  #promise;
  #terminate;

  constructor(safe, prefix) {
    const worker = new Worker(new URL("./worker.js", import.meta.url));

    this.#promise = new Promise((resolve, reject) => {
      this.#terminate = (err) => {
        worker.terminate();
        reject(err);
      };

      worker.addEventListener("message", (message) => {
        this.#terminate = undefined;

        const { creation, err } = message.data ?? {};
        if (typeof creation === "object" && creation !== null) {
          resolve(creation);
        } else {
          reject(err ?? new Error("unknown error"));
        }
      });
    });

    worker.postMessage({ safe, prefix });
  }

  wait() {
    return this.#promise;
  }

  cancel(err) {
    if (this.#terminate !== undefined) {
      this.#terminate(err ?? new Error("cancelled"));
    }
  }
}
