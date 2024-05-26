const fs = require("fs");
const { init_panic_hook, CompactPkePublicParams } = require("node-tfhe");

const CRS_PARAMS_PATH = "../config/crs/params_1.bin";

async function initializeTfhe() {
  await init_panic_hook();
}

function loadCrsParams(path: string) {
  const publicZkParamsData = fs.readFileSync(path);
  const publicZkParams = CompactPkePublicParams.deserialize(publicZkParamsData);
  return publicZkParams;
}

(async () => {
  await initializeTfhe();
  const crs = loadCrsParams(CRS_PARAMS_PATH);
  fs.writeFileSync("crs.json", JSON.stringify(crs));
})();
