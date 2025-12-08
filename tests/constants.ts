import { PublicKey } from "@solana/web3.js";
import { BN } from "bn.js";
import idl from "../target/idl/solvbtc.json";

export const SURFPOOL_RPC_URL = "http://127.0.0.1:8899";

export const SOLVBTC_PROGRAM_ID = new PublicKey(idl.address);
export const CCIP_TOKENPOOL_PROGRAM_ID = new PublicKey("ECvqYduigrFHeAU1kFCkehiiQz9eaeddUz6gH7BfD7AL");

export const ONE_BITCOIN = new BN(100_000_000); // 10^8