import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import "./App.css";

import init, {
  // initThreadPool, // only available with parallelism
  init_panic_hook,
  TfheCompactPublicKey,
  TfheCompressedCompactPublicKey,
  CompactPkePublicParams,
  ProvenCompactFheUint64,
  ZkComputeLoad,
  CompactFheUint64,
} from "tfhe";

function encryptAndProve(
  value: bigint,
  publicParams: CompactPkePublicParams,
  publicKey: TfheCompactPublicKey,
): ProvenCompactFheUint64 {
  return ProvenCompactFheUint64.encrypt_with_compact_public_key(
    value,
    publicParams,
    publicKey,
    ZkComputeLoad.Proof,
  );
}

function encrypt(
  value: bigint,
  publicKey: TfheCompactPublicKey,
): CompactFheUint64 {
  return CompactFheUint64.encrypt_with_compact_public_key(value, publicKey);
}

async function fetchBinaryData(path: string): Promise<Uint8Array> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${path}: ${response.statusText}`);
  }
  const arrayBuffer = await response.arrayBuffer();
  return new Uint8Array(arrayBuffer);
}

const PUBLIC_KEY_PATH = "config/public_key.bin";
const CRS_PARAMS_PATH = "config/crs/params_1.bin";

async function initializeTfhe() {
  await init();
  // await initThreadPool(navigator.hardwareConcurrency);
  await init_panic_hook();
}

async function loadCrsParams(): Promise<CompactPkePublicParams> {
  const publicZkParamsData = await fetchBinaryData(CRS_PARAMS_PATH);
  console.log(publicZkParamsData);
  const publicZkParams = CompactPkePublicParams.deserialize(publicZkParamsData);
  console.log(publicZkParams);
  return publicZkParams;
}

async function loadPublicKey(): Promise<TfheCompactPublicKey> {
  const compressedPublicKeyData = await fetchBinaryData(PUBLIC_KEY_PATH);
  console.log(compressedPublicKeyData);
  const compressedPublicKey = TfheCompressedCompactPublicKey.deserialize(
    compressedPublicKeyData,
  );
  const publicKey = compressedPublicKey.decompress();
  console.log(publicKey);
  return publicKey;
}

function App() {
  const [publicKey, setPublicKey] = useState<TfheCompactPublicKey | null>(null);
  console.log("here0");

  useEffect(() => {
    console.log("Initializing TFHE...");
    initializeTfhe();
    console.log("Done!");
  }, []);

  const handleClick = async () => {
    console.log("Loading public key...");
    const key = await loadPublicKey();
    console.log("Done!");
    setPublicKey(key);
    console.log("Encrypting...");
    const cipher = encrypt(1n, key);
    console.log("Done!");
    const serialized = cipher.serialize();
  };

  return (
    <>
      <div>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vote</h1>
      <div className="card">
        <button onClick={handleClick}>Load</button>
      </div>
    </>
  );
}

export default App;
