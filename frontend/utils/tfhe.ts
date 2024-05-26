import init, {
  // initThreadPool, // only available with parallelism
  init_panic_hook,
  TfheCompactPublicKey,
  CompactPkePublicParams,
  ProvenCompactFheUint64,
  ZkComputeLoad,
  CompactFheUint64,
} from "tfhe";

export async function loadPublicKey(path: string): Promise<TfheCompactPublicKey> {
  console.log("Fetching public key...");
  const compressedPublicKeyData = await fetchBinaryData(path);
  console.log("Done!");
  console.log("Deserializing public key...");
  const publicKey = TfheCompactPublicKey.deserialize(compressedPublicKeyData);
  console.log("Done! Public key: ", publicKey);
  return publicKey;
}

export async function loadCrsParams(path: string): Promise<CompactPkePublicParams> {
  const publicZkParamsData = await fetchBinaryData(path);
  console.log(publicZkParamsData);
  const publicZkParams = CompactPkePublicParams.deserialize(publicZkParamsData);
  console.log(publicZkParams);
  return publicZkParams;
}

export async function fetchBinaryData(path: string): Promise<Uint8Array> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${path}: ${response.statusText}`);
  }
  const arrayBuffer = await response.arrayBuffer();
  return new Uint8Array(arrayBuffer);
}

export function encrypt(value: bigint, publicKey: TfheCompactPublicKey): CompactFheUint64 {
  return CompactFheUint64.encrypt_with_compact_public_key(value, publicKey);
}

export function encryptAndProve(
  value: bigint,
  publicParams: CompactPkePublicParams,
  publicKey: TfheCompactPublicKey,
): ProvenCompactFheUint64 {
  return ProvenCompactFheUint64.encrypt_with_compact_public_key(value, publicParams, publicKey, ZkComputeLoad.Proof);
}

export async function initializeTfhe() {
  await init();
  // await initThreadPool(navigator.hardwareConcurrency);
  await init_panic_hook();
}
