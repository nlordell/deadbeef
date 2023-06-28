/**
 * This script bundles the `deadbeef` SearchWorker into a single file. This is
 * done in a two step process:
 * 1. The `worker.js` and imports are bundled into a file with the `deadbeef`
 *    WASM blob embedded.
 * 2. The `index.js` is bundled with the `worker.js` JavaScript, so we can
 *    do `deadbeef` vanity Safe address searching with a single file.
 */

// TODO(nlordell): Figure out why this doesn't work...
// import { build } from "https://deno.land/x/esbuild@v0.18.10/mod.js";

import { build } from "npm:esbuild@0.18.10";

await build({
  entryPoints: ["lib/worker.js"],
  bundle: true,
  // TODO(nlordell): Figure out why minifying doesn't work.
  // minify: true,
  outdir: "dist",
  format: "esm",
  loader: {
    [".wasm"]: "binary",
  },
});

const workerSrc = await Deno.readTextFile("dist/worker.js");
const indexSrc = await Deno.readTextFile("lib/index.js");

const bundle = `
const workerSource = new Blob([${
  JSON.stringify(
    workerSrc
      .replace(
        /[a-zA-Z]* *= *new URL\(['"]deadbeef_bg\.wasm['"], *import\.meta\.url\)/,
        'throw new Error("missing WASM source")',
      ),
  )
}]);
const workerUrl = URL.createObjectURL(workerSource);

${
  indexSrc
    .replace('new URL("./worker.js", import.meta.url)', "workerUrl")
}
`;

await Deno.writeTextFile("dist/index.js", `${bundle.trim()}\n`);
