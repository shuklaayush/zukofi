## Overview

Often, hackathons have a Hackers Choice Award for participants to select their favourite project. Other than the lack of privacy, public voting has multiple drawbacks such as the current tally of the votes might influcence the decision of voters.

We use Zupass as the source of proof for participants to prove that they are an EthBerlin hacker, which allows them to vote on projects. The participant will encrypt their vote on client side and send the encrypted vote to a server. The server will then be able to tally the votes using homomorphic encrption and reveal the final tally at the end.

### Getting Started

Run `bash run-servers.sh` to start the backend servers. Run `cd frontend && pnpm dev` to start the frontend.