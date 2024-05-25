"use client";

import { useCallback, useState, useEffect } from "react";
import { ZKEdDSAEventTicketPCDPackage } from "@pcd/zk-eddsa-event-ticket-pcd";
import { zuAuthPopup } from "@pcd/zuauth";
import type { NextPage } from "next";
import { hexToBigInt } from "viem";
import { useAccount } from "wagmi";
import { useScaffoldReadContract, useScaffoldWriteContract } from "~~/hooks/scaffold-eth";
import { notification } from "~~/utils/scaffold-eth";
import { generateWitness, isETHBerlinPublicKey } from "~~/utils/scaffold-eth/pcd";
import { ETHBERLIN_ZUAUTH_CONFIG } from "~~/utils/zupassConstants";
import init, {
  // initThreadPool, // only available with parallelism
  init_panic_hook,
  TfheCompactPublicKey,
  CompactPkePublicParams,
  ProvenCompactFheUint64,
  ZkComputeLoad,
  CompactFheUint64,
} from "tfhe";

const BASE_URL = "http://localhost:8000";
const PUBLIC_KEY_PATH = `${BASE_URL}/public-key`;

const CRS_PARAMS_PATH = "config/crs/params_1.bin";

// Get a valid event id from { supportedEvents } from "zuauth" or https://api.zupass.org/issue/known-ticket-types
const fieldsToReveal = {
  revealAttendeeEmail: true,
  revealEventId: true,
  revealProductId: true,
};

function encryptAndProve(
  value: bigint,
  publicParams: CompactPkePublicParams,
  publicKey: TfheCompactPublicKey,
): ProvenCompactFheUint64 {
  return ProvenCompactFheUint64.encrypt_with_compact_public_key(value, publicParams, publicKey, ZkComputeLoad.Proof);
}

function encrypt(value: bigint, publicKey: TfheCompactPublicKey): CompactFheUint64 {
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
  console.log("Fetching public key...");
  const compressedPublicKeyData = await fetchBinaryData(PUBLIC_KEY_PATH);
  console.log("Done!");
  console.log("Deserializing public key...");
  const publicKey = TfheCompactPublicKey.deserialize(compressedPublicKeyData);
  console.log("Done! Public key: ", publicKey);
  return publicKey;
}

const Home: NextPage = () => {
  const [verifiedFrontend, setVerifiedFrontend] = useState(false);
  const [verifiedBackend, setVerifiedBackend] = useState(false);
  const [verifiedOnChain, setVerifiedOnChain] = useState(false);
  const { address: connectedAddress } = useAccount();
  const [pcd, setPcd] = useState<string>();
  const [publicKey, setPublicKey] = useState<TfheCompactPublicKey | null>(null);

  useEffect(() => {
    (async () => {
      console.log("Initializing TFHE...");
      await initializeTfhe();
      console.log("Done!");

      const key = await loadPublicKey();
      setPublicKey(key);
    })();
  }, []);

  const handleClick = async (vote: bigint) => {
    if (publicKey) {
      await sendPCDToServer();
      console.log("Encrypting...");
      console.log("Vote: ", vote);
      const cipher = encrypt(vote, publicKey);
      console.log("Done!");
      const serialized = cipher.serialize();
      console.log("Serialized: ", serialized);
    }
  };

  const getProof = async () => {
    const result = await zuAuthPopup({ fieldsToReveal, watermark: connectedAddress, config: ETHBERLIN_ZUAUTH_CONFIG });
    if (result.type === "pcd") {
      setPcd(JSON.parse(result.pcdStr).pcd);
    } else {
      notification.error("Failed to parse PCD");
    }
  };

  const verifyProofFrontend = async () => {
    if (!pcd) {
      notification.error("No PCD found!");
      return;
    }

    const deserializedPCD = await ZKEdDSAEventTicketPCDPackage.deserialize(pcd);

    if (!(await ZKEdDSAEventTicketPCDPackage.verify(deserializedPCD))) {
      notification.error(`[ERROR Frontend] ZK ticket PCD is not valid`);
      return;
    }

    if (!isETHBerlinPublicKey(deserializedPCD.claim.signer)) {
      notification.error(`[ERROR Frontend] PCD is not signed by ETHBerlin`);
      return;
    }

    if (deserializedPCD.claim.watermark.toString() !== hexToBigInt(connectedAddress as `0x${string}`).toString()) {
      notification.error(`[ERROR Frontend] PCD watermark doesn't match`);
      return;
    }

    setVerifiedFrontend(true);
    notification.success(
      <>
        <p className="font-bold m-0">Frontend Verified!</p>
        <p className="m-0">
          The proof has been verified
          <br /> by the frontend.
        </p>
      </>,
    );
  };

  const sendPCDToServer = async () => {
    let response;
    try {
      response = await fetch("/api/verify", {
        method: "POST",
        body: JSON.stringify({
          pcd: pcd,
          address: connectedAddress,
        }),
        headers: {
          "Content-Type": "application/json",
        },
      });
    } catch (e) {
      notification.error(`Error: ${e}`);
      return;
    }

    const data = await response.json();
    setVerifiedBackend(true);
    notification.success(
      <>
        <p className="font-bold m-0">Backend Verified!</p>
        <p className="m-0">{data?.message}</p>
      </>,
    );
  };

  // mintItem verifies the proof on-chain and mints an NFT
  const { writeContractAsync: mintNFT, isPending: isMintingNFT } = useScaffoldWriteContract("YourCollectible");

  const { data: yourBalance } = useScaffoldReadContract({
    contractName: "YourCollectible",
    functionName: "balanceOf",
    args: [connectedAddress],
  });

  return (
    <>
      <div className="flex flex-col items-center mt-24">
        <div className="card max-w-[90%] sm:max-w-lg bg-base-100 shadow-xl">
          <div className="card-body">
            <h2 className="card-title">Zupass: Private Voting</h2>
            <div className="flex flex-col gap-4 mt-6">
              <div className="tooltip" data-tip="Loads the Zupass UI in a modal, where you can prove your PCD.">
                <button className="btn btn-secondary w-full tooltip" onClick={getProof} disabled={!!pcd}>
                  {!pcd ? "1. Prove Membership" : "1. Proof Received!"}
                </button>
              </div>
              <div className="tooltip" data-tip="When you get back the PCD, verify it on the frontend.">
                <button
                  className="btn btn-primary w-full"
                  disabled={!pcd || verifiedFrontend}
                  onClick={verifyProofFrontend}
                >
                  2. Verify (frontend)
                </button>
              </div>
              <div className="tooltip" data-tip="Vote for project">
                <button
                  className="btn btn-primary w-full"
                  disabled={!verifiedFrontend || verifiedBackend}
                  onClick={() => handleClick(0n)}
                >
                  YAY
                </button>
              </div>
              <div className="tooltip" data-tip="Vote against project">
                <button
                  className="btn btn-primary w-full"
                  disabled={!verifiedFrontend || verifiedBackend}
                  onClick={() => handleClick(1n)}
                >
                  NAY
                </button>
              </div>
              <div className="flex justify-center">
                <button
                  className="btn btn-ghost text-error underline normal-case"
                  onClick={() => {
                    setVerifiedFrontend(false);
                  }}
                >
                  Reset
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

export default Home;
