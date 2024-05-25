const isETHBerlinPublicKey = (signer) => {
  return (
    signer[0] ===
      "1ebfb986fbac5113f8e2c72286fe9362f8e7d211dbc68227a468d7b919e75003" &&
    signer[1] ===
      "10ec38f11baacad5535525bbe8e343074a483c051aa1616266f3b1df3fb7d204"
  );
};

module.exports = {
  isETHBerlinPublicKey,
};
