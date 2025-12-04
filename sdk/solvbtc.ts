import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { keccak256 } from 'ethereum-cryptography/keccak';

import { secp256k1 } from "@noble/curves/secp256k1";
import { sha256 } from "@noble/hashes/sha2";
import { Keypair, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { LENGTH_SIZE } from "@solana/spl-token";

export const SOLVBTC_PROGRAM_ID = new PublicKey("CAm2xucNj5S5xRtGikxBbZcCyXLaz3Y3VyBd6CewVBxn");
export const CCIP_TOKENPOOL_PROGRAM_ID = new PublicKey("ECvqYduigrFHeAU1kFCkehiiQz9eaeddUz6gH7BfD7AL");

export const SOLV_BTC_MINT = new PublicKey("SoLvHDFVstC74Jr9eNLTDoG4goSUsn1RENmjNtFKZvW");
export const X_SOLV_BTC = new PublicKey("SoLvAiHLF7LGEaiTN5KGZt1bNnraoWTi5mjcvRoDAX4");
export const SOLV_BTC_JUP = new PublicKey("SoLvzL3ZVjofmNB5LYFrf94QtNhMUSea4DawFhnAau8");

export const VAULT_SIGNER_SEED = new TextEncoder().encode("vault");
export const MINTER_MANAGER_SEED = new TextEncoder().encode("minter_manager");
export const POOL_SIGNER_SEED = new TextEncoder().encode("ccip_tokenpool_signer");
export const WITHDRAW_REQUEST_SEED = new TextEncoder().encode("withdraw_request");

export const ONE_BITCOIN = new BN(100_000_000)

export function ecdsaPubkeyFromPrivkey(privkey: Uint8Array): Uint8Array {
    return secp256k1.getPublicKey(privkey, false)
}

export function derivePoolSignerAddress(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      POOL_SIGNER_SEED,
      mint.toBytes()
    ],
    CCIP_TOKENPOOL_PROGRAM_ID
  )[0]
}

export function deriveVaultAddress(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      VAULT_SIGNER_SEED,
      mint.toBytes()
    ],
    SOLVBTC_PROGRAM_ID    
  )[0]
}

export function deriveMinterManagerAddress(vault: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      MINTER_MANAGER_SEED,
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
      WITHDRAW_REQUEST_SEED,
      vault.toBytes(),
      withdrawMint.toBytes(),
      user.toBytes(),
      hash
    ],
    SOLVBTC_PROGRAM_ID    
  )[0]
}

/// Create a random 32-byte scalar for the request
export function createWithdrawRequestHash(): Uint8Array {
  return Keypair.generate().secretKey.subarray(0, 32)
}

export function deriveWithdrawRequestSigningHash(user: PublicKey, mint: PublicKey, hash: Uint8Array, shares: BN, nav: BN): Uint8Array {
  return sha256(new Uint8Array([
    ...user.toBytes(),
    ...mint.toBytes(),
    ...hash,
    ...shares.toArrayLike(Buffer, 'le', 8),
    ...nav.toArrayLike(Buffer, 'le', 8),
  ]))
}

export function deriveWithdrawRequestEip191(user: PublicKey, mint: PublicKey, hash: Uint8Array, share:BN, nav: BN): Uint8Array{
  let prefix = "\x19Ethereum Signed Message:\n";
  let msg = user.toBase58() +"\n" + mint.toBase58() +"\n" + bs58.encode(hash) + "\n" 
  + Number(share).toString() + "\n" + Number(nav).toString();
  let eip191Msg = prefix + msg.length.toString() + msg;
  console.log(eip191Msg);
  return Buffer.from(eip191Msg);
}

export function createEip191WithdrawSig(privkey: Uint8Array, hash: Uint8Array): {
  isOdd: boolean;
  signature: number[];
}{
  const signature = secp256k1.sign(keccak256(hash), privkey, {lowS:true})
  return {
    isOdd: signature.recovery != 0,
    signature: new Array(...signature.toBytes("compact"))
  }
}

export function createWithdrawSignature(privkey: Uint8Array, hash: Uint8Array): {
  isOdd: boolean;
  signature: number[];
} {
  const signature = secp256k1.sign(sha256(hash), privkey, {
    lowS: true,
  });

  return {
    isOdd: signature.recovery != 0,
    signature: new Array(...signature.toBytes("compact")),
  };
}