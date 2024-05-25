import { ZKEdDSAEventTicketPCDPackage } from "@pcd/zk-eddsa-event-ticket-pcd";
import { NextApiRequest, NextApiResponse } from "next";
import { isETHBerlinPublicKey } from "~~/utils/scaffold-eth/pcd";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  console.log("Breakpoint");
  const pcd = await ZKEdDSAEventTicketPCDPackage.deserialize(req.body.pcd);

  if (!(await ZKEdDSAEventTicketPCDPackage.verify(pcd))) {
    console.error(`[ERROR] ZK ticket PCD is not valid`);

    return res.status(401).send("ZK ticket PCD is not valid");
  }

  if (!isETHBerlinPublicKey(pcd.claim.signer)) {
    console.error(`[ERROR] PCD is not signed by Zupass`);

    return res.status(401).send("PCD is not signed by ETHBerlin");
  }

  // TODO: Check that the event id is the one we expect

  return res.status(200).json({ message: "PCD verified!" });
}
