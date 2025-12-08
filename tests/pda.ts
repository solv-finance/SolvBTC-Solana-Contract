import { PublicKey } from "@solana/web3.js"
import { CCIP_TOKENPOOL_PROGRAM_ID, SOLVBTC_PROGRAM_ID } from "./constants";

const textEncoder = new TextEncoder();

export function derivePoolSignerAddress(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      textEncoder.encode("ccip_tokenpool_signer"),
      mint.toBytes()
    ],
    CCIP_TOKENPOOL_PROGRAM_ID
  )[0]
}

export function deriveVaultAddress(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      textEncoder.encode("vault"),
      mint.toBytes()
    ],
    SOLVBTC_PROGRAM_ID
  )[0]
}

export function deriveMinterManagerAddress(vault: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      textEncoder.encode("minter_manager"),
      vault.toBytes()
    ],
    SOLVBTC_PROGRAM_ID
  )[0]
}

export function deriveWithdrawRequestAddress(vault: PublicKey, withdrawMint: PublicKey, user: PublicKey, hash: Uint8Array): PublicKey {
  if (hash.length != 32) {
    throw new Error("Invalid hash length, expected 32")
  }

  return PublicKey.findProgramAddressSync(
    [
      textEncoder.encode("withdraw_request"),
      vault.toBytes(),
      withdrawMint.toBytes(),
      user.toBytes(),
      hash
    ],
    SOLVBTC_PROGRAM_ID
  )[0]
}