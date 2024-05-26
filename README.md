## Overview

Often, hackathons have a Hackers Choice Award for participants to select their favourite project. These are usually held in public which can lead to quid pro quo arrangements and other privacy issues. Other than the lack of privacy, public voting has other drawbacks such as the current tally of votes influencing the future decision of voters.

Zukofi (or ZuQuoFHE from Zuzalu, quadratic voting and FHE) allows participants to prove that they are an ETHBerlin hacker and allows them to vote on projects. Each participant has a fixed budget for voting and the cost of each extra vote to a project scales quadratically. The participants encrypt their votes on the client side and send the encrypted votes to a server along with a ZK proof of inclusion showing that they are a ETHBerlin hacker.

The server checks that the voter is eligible by verifying the ZK proof of inclusion and checking that the voter hasn’t already voted. It then checks that the vote is valid and doesn’t exceed the budget. If both checks are passed, the server adds the vote to the current tally using homomorphic encryption.

Once the voting has finished, the server decrypts and reveals the final tally.

### Getting Started

Run `bash run-servers.sh` to start the backend servers. Run `cd frontend && pnpm dev` to start the frontend.
