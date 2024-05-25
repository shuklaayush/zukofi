const cors = require("cors");
const express = require("express");
const bodyParser = require("body-parser");
const {
  ZKEdDSAEventTicketPCDPackage,
} = require("@pcd/zk-eddsa-event-ticket-pcd");
const { isETHBerlinPublicKey } = require("./utils"); // Adjust the path as needed

const nullifierHashSet = new Set();

const app = express();
const port = 8001;

// CORS configuration
const corsOptions = {
  origin: "*", // You can change this to a specific origin or an array of allowed origins
  methods: ["GET", "POST"],
  allowedHeaders: [
    "Authorization",
    "Accept",
    "Access-Control-Allow-Origin",
    "Content-Type",
  ],
  credentials: true,
};

// Use CORS middleware
app.use(cors(corsOptions));

// Middleware to parse JSON bodies
app.use(bodyParser.json());

app.post("/verify", async (req, res) => {
  try {
    console.error("Received verification request");
    const pcd = await ZKEdDSAEventTicketPCDPackage.deserialize(req.body.pcd);

    if (!isETHBerlinPublicKey(pcd.claim.signer)) {
      console.error(`[ERROR] PCD is not signed by Zupass`);
      return res.status(401).send("PCD is not signed by ETHBerlin");
    }

    const nullifierHash = pcd.claim.nullifierHash;
    if (nullifierHashSet.has(nullifierHash)) {
      console.error(`[ERROR] Already voted`);
      return res.status(401).send("Already voted");
    }

    if (!(await ZKEdDSAEventTicketPCDPackage.verify(pcd))) {
      console.error(`[ERROR] ZK ticket PCD is not valid`);
      return res.status(401).send("ZK ticket PCD is not valid");
    }

    // TODO: Check that the event id is the one we expect

    // Add nullifier to set
    nullifierHashSet.add(nullifierHash);
    return res.status(200).json({ message: "PCD verified!" });
  } catch (error) {
    console.error(`[ERROR] ${error.message}`);
    return res.status(500).send("Internal Server Error");
  }
});

app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});
