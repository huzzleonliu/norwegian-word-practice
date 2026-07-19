/**
 * OAOA Candy Machine mint helper for browser (Phantom / window.solana).
 * Loaded as an ES module; exposes window.__oaoaMintNft().
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
  ] = await Promise.all([
    import("https://esm.sh/@metaplex-foundation/umi-bundle-defaults@0.9.2"),
    import("https://esm.sh/@metaplex-foundation/mpl-candy-machine@6.0.1"),
    import("https://esm.sh/@metaplex-foundation/umi@0.9.2"),
    import("https://esm.sh/@metaplex-foundation/mpl-toolbox@0.9.4"),
    import("https://esm.sh/@metaplex-foundation/umi-signer-wallet-adapters@0.9.2"),
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
  };
}

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
