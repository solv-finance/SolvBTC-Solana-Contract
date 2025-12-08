import { secp256k1 } from "@noble/curves/secp256k1";
import { Keypair, PublicKey } from "@solana/web3.js";
import { sha256 } from "@noble/hashes/sha2";
import BN from "bn.js";

export function ecdsaPubkeyFromPrivkey(privkey: Uint8Array): Uint8Array {
  return secp256k1.getPublicKey(privkey, false)
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