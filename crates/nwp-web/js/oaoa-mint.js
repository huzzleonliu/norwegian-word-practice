/**
 * OAOA Candy Machine helpers for browser (Phantom / window.solana).
 * Exposes window.__oaoaMintNft() and window.__oaoaHasCollectionNft(owner).
 */
const RPC = "https://api.devnet.solana.com";
const CANDY_MACHINE = "C2sf5YPcnrKabkXqs1615iF6Y2udrimMHis6rYnUQVN7";
const CANDY_GUARD = "EoDwnkGMgZcoikFS6dLuaDB552m7yx3u35eJvXoSPX6P";
const COLLECTION_MINT = "9qcraUcqpzRuXKSuPBuGvc3xn5wb184MYzWadDUWfgay";
const TREASURY = "YGR2KWyPtqGTcWZ2QiSMfdq5ProDGVvhwFe6KqrYZJx";

async function loadSdk() {
  const [
    { createUmi },
    { mplCandyMachine, fetchCandyMachine, mintV2 },
    { publicKey, generateSigner, some, transactionBuilder },
    { setComputeUnitLimit },
    { walletAdapterIdentity },
    { mplTokenMetadata, fetchAllDigitalAssetWithTokenByOwner },
  ] = await Promise.all([
    import("https://esm.sh/@metaplex-foundation/umi-bundle-defaults@0.9.2"),
    import("https://esm.sh/@metaplex-foundation/mpl-candy-machine@6.0.1"),
    import("https://esm.sh/@metaplex-foundation/umi@0.9.2"),
    import("https://esm.sh/@metaplex-foundation/mpl-toolbox@0.9.4"),
    import("https://esm.sh/@metaplex-foundation/umi-signer-wallet-adapters@0.9.2"),
    import("https://esm.sh/@metaplex-foundation/mpl-token-metadata@3.3.0"),
  ]);
  return {
    createUmi,
    mplCandyMachine,
    fetchCandyMachine,
    mintV2,
    publicKey,
    generateSigner,
    some,
    transactionBuilder,
    setComputeUnitLimit,
    walletAdapterIdentity,
    mplTokenMetadata,
    fetchAllDigitalAssetWithTokenByOwner,
  };
}

/** @param {string} ownerPubkey */
window.__oaoaHasCollectionNft = async function __oaoaHasCollectionNft(ownerPubkey) {
  if (!ownerPubkey || typeof ownerPubkey !== "string") {
    return false;
  }
  const sdk = await loadSdk();
  const umi = sdk.createUmi(RPC).use(sdk.mplTokenMetadata());
  const assets = await sdk.fetchAllDigitalAssetWithTokenByOwner(
    umi,
    sdk.publicKey(ownerPubkey)
  );
  const collection = COLLECTION_MINT;
  return assets.some((asset) => {
    const col = asset?.metadata?.collection;
    if (!col) return false;
    // umi Option: { __option: 'Some', value: { key, verified } } or similar shapes
    const value = col.__option === "Some" ? col.value : col.value ?? col;
    if (!value) return false;
    const key = (value.key ?? value.address ?? value)?.toString?.() ?? String(value.key ?? "");
    const verified = value.verified === true || value.verified === 1;
    return verified && key === collection;
  });
};

window.__oaoaMintNft = async function __oaoaMintNft() {
  const provider = window.solana;
  if (!provider || !provider.publicKey) {
    throw new Error("Wallet not connected. Connect Phantom first.");
  }
  if (provider.isConnected === false) {
    await provider.connect();
  }

  const sdk = await loadSdk();
  const umi = sdk
    .createUmi(RPC)
    .use(sdk.mplCandyMachine())
    .use(sdk.walletAdapterIdentity(provider));

  const candyMachine = await sdk.fetchCandyMachine(
    umi,
    sdk.publicKey(CANDY_MACHINE)
  );
  const nftMint = sdk.generateSigner(umi);

  const builder = sdk
    .transactionBuilder()
    .add(sdk.setComputeUnitLimit(umi, { units: 800_000 }))
    .add(
      sdk.mintV2(umi, {
        candyMachine: candyMachine.publicKey,
        nftMint,
        collectionMint: sdk.publicKey(COLLECTION_MINT),
        collectionUpdateAuthority: candyMachine.authority,
        candyGuard: sdk.publicKey(CANDY_GUARD),
        mintArgs: {
          solPayment: sdk.some({ destination: sdk.publicKey(TREASURY) }),
        },
      })
    );

  const result = await builder.sendAndConfirm(umi);
  const sig =
    typeof result.signature === "string"
      ? result.signature
      : result.signature?.toString?.() ?? String(result.signature);
  return {
    signature: sig,
    mint: nftMint.publicKey.toString(),
  };
};
