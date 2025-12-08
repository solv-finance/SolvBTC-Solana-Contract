import { Program } from "@coral-xyz/anchor";
import { beforeAll, describe, expect, it } from "bun:test";
import { Solvbtc } from "../target/types/solvbtc";
import { expectError, expireBlockhash, getSetup } from "./setup";
import { Connection, Keypair, PublicKey, sendAndConfirmTransaction, Transaction } from "@solana/web3.js";
import { Surfpool } from "./surfpool";
import { deriveWithdrawRequestAddress as getWithdrawRequestPda, deriveMinterManagerAddress as getMinterManagerPda, derivePoolSignerAddress as getPoolSignerPda, deriveVaultAddress as getVaultPda } from "./pda";
import { createWithdrawRequestHash, deriveWithdrawRequestSigningHash as getWithdrawRequestSigningHash, ecdsaPubkeyFromPrivkey, createWithdrawSignature } from "./helpers";
import { ONE_BITCOIN } from "./constants";
import { BN } from "bn.js";
import { createAssociatedTokenAccountIdempotentInstruction, createTransferCheckedInstruction, getAccount, getAssociatedTokenAddressSync, getMint, TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("solvbtc", () => {
  let program: Program<Solvbtc>;
  let connection: Connection;

  let authority: Keypair;
  let [
    admin,
    payer,
    feeReceiver,
    treasurer,
    oracleManager,
    user,
    mintA,
    mintB,
    multisigA,
    multisigB,
  ] = Array.from({ length: 10 }, Keypair.generate);

  const poolSignerA = getPoolSignerPda(mintA.publicKey);
  const poolSignerB = getPoolSignerPda(mintB.publicKey);
  const vaultA = getVaultPda(mintA.publicKey);
  const vaultB = getVaultPda(mintB.publicKey);
  const minterManagerA = getMinterManagerPda(vaultA);
  const minterManagerB = getMinterManagerPda(vaultB);

  const hash = createWithdrawRequestHash();
  const withdrawRequest = getWithdrawRequestPda(vaultA, mintB.publicKey, user.publicKey, hash);

  const verifierKeypair = Buffer.from("c2fffbf8e5cec943afb99e8194a5819c64c9df75b4ed03b2a111e8ccdcf55689", "hex")
  const verifier = Array.from(ecdsaPubkeyFromPrivkey(verifierKeypair).subarray(1));

  const userAtaA = getAssociatedTokenAddressSync(mintA.publicKey, user.publicKey, !PublicKey.isOnCurve(user.publicKey));
  const userAtaB = getAssociatedTokenAddressSync(mintB.publicKey, user.publicKey, !PublicKey.isOnCurve(user.publicKey));
  const feeReceiverAtaB = getAssociatedTokenAddressSync(mintB.publicKey, feeReceiver.publicKey, !PublicKey.isOnCurve(feeReceiver.publicKey));
  const vaultAAtaB = getAssociatedTokenAddressSync(mintB.publicKey, vaultA, true);

  let authorityAtaA: PublicKey;
  let authorityAtaB: PublicKey;

  beforeAll(async () => {
    authority = Keypair.fromSecretKey(new Uint8Array(await Bun.file("./authority-keypair-wallet.json").json()));

    authorityAtaA = getAssociatedTokenAddressSync(mintA.publicKey, authority.publicKey, !PublicKey.isOnCurve(authority.publicKey));
    authorityAtaB = getAssociatedTokenAddressSync(mintB.publicKey, authority.publicKey, !PublicKey.isOnCurve(authority.publicKey));

    ({ program } = await getSetup([
      {
        publicKey: authority.publicKey,
      },
      {
        publicKey: admin.publicKey,
      },
      {
        publicKey: user.publicKey,
      },
      {
        publicKey: payer.publicKey,
      },
      {
        publicKey: feeReceiver.publicKey,
      },
      {
        publicKey: treasurer.publicKey,
      },
      {
        publicKey: oracleManager.publicKey,
      },
    ]));

    connection = program.provider.connection;

    await Surfpool.initMultisig({
      multisig: multisigA.publicKey,
      m: 1,
      n: 2,
      signer1: vaultA,
      signer2: poolSignerA,
    })

    await Surfpool.initMultisig({
      multisig: multisigB.publicKey,
      m: 1,
      n: 2,
      signer1: vaultB,
      signer2: poolSignerB,
    })

    await Surfpool.initMint({
      mint: mintA.publicKey,
      decimals: 8,
      mintAuthority: multisigA.publicKey,
      freezeAuthority: multisigA.publicKey,
    })

    await Surfpool.initMint({
      mint: mintB.publicKey,
      decimals: 8,
      mintAuthority: multisigB.publicKey,
      freezeAuthority: multisigB.publicKey,
    })
  });

  it("initialize vaults", async () => {
    const nav = ONE_BITCOIN;
    const withdrawFee = 50; // 0.5%

    await program.methods
      .vaultInitialize(
        admin.publicKey,
        feeReceiver.publicKey,
        treasurer.publicKey,
        Array.from(verifier),
        oracleManager.publicKey,
        nav,
        withdrawFee,
      )
      .accounts({
        authority: authority.publicKey,
        mint: mintA.publicKey,
        payer: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.admin.equals(admin.publicKey)).toBeTrue();
    expect(vaultAAcc.feeReceiver.equals(feeReceiver.publicKey)).toBeTrue();
    expect(vaultAAcc.treasurer.equals(treasurer.publicKey)).toBeTrue();
    expect(vaultAAcc.verifier).toEqual(verifier);
    expect(vaultAAcc.oracleManager.equals(oracleManager.publicKey)).toBeTrue();
    expect(vaultAAcc.nav.eq(nav)).toBeTrue();
    expect(vaultAAcc.mint.equals(mintA.publicKey)).toBeTrue();

    await program.methods
      .vaultInitialize(
        authority.publicKey,
        authority.publicKey,
        authority.publicKey,
        Array.from(verifier),
        authority.publicKey,
        nav,
        withdrawFee,
      )
      .accounts({
        authority: authority.publicKey,
        mint: mintB.publicKey,
        payer: authority.publicKey,
      })
      .signers([authority])
      .rpc();
  })

  it("transfer vault admin", async () => {
    const newAdmin = authority.publicKey;

    await program.methods
      .vaultTransferAdmin(
        newAdmin
      )
      .accountsPartial({
        mint: mintA.publicKey,
        payer: admin.publicKey,
        admin: admin.publicKey,
      })
      .signers([admin])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.admin.equals(newAdmin)).toBeTrue();
  })

  it("set vault treasurer", async () => {
    const newTreasury = authority.publicKey;

    await program.methods
      .vaultSetTreasurer(
        newTreasury
      )
      .accountsPartial({
        mint: mintA.publicKey,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.treasurer.equals(newTreasury)).toBeTrue();
  })

  it("add vault currencies", async () => {
    const newMint = mintB.publicKey;
    const depositFee = 75; // 0.75%

    await program.methods
      .vaultAddCurrency(
        newMint,
        depositFee,
      )
      .accountsPartial({
        mint: mintA.publicKey,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.depositCurrencies).toContainEqual({ mint: newMint, depositFee });
  })

  it("throws if vault currency already exists", async () => {
    try {
      await program.methods
        .vaultAddCurrency(
          mintB.publicKey,
          50,
        )
        .accountsPartial({
          mint: mintA.publicKey,
          payer: authority.publicKey,
          admin: authority.publicKey,
        })
        .signers([authority])
        .rpc();
    } catch (err) {
      expectError(err, "CurrencyAlreadyExists");
    }
  })

  it("remove vault currency", async () => {
    const mintToRemove = mintA.publicKey;

    await program.methods
      .vaultAddCurrency(
        mintToRemove,
        50,
      )
      .accountsPartial({
        mint: mintToRemove,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    await program.methods
      .vaultRemoveCurrency(mintToRemove)
      .accountsPartial({
        mint: mintA.publicKey,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.depositCurrencies).not.toContainEqual({ mint: mintToRemove, depositFee: expect.any(Number) });
  })

  it("set vault withdraw fee", async () => {
    const withdrawFee = 500;

    await program.methods
      .vaultSetWithdrawFee(withdrawFee)
      .accountsPartial({
        mint: mintA.publicKey,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.withdrawFee).toBe(withdrawFee);
  })

  it("set vault deposit fee", async () => {
    const depositFee = 500; // 5%

    await program.methods
      .vaultSetDepositFee(
        mintB.publicKey,
        depositFee,
      )
      .accountsPartial({
        mint: mintA.publicKey,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.depositCurrencies).toContainEqual({ mint: mintB.publicKey, depositFee });
  })

  it("set vault fee receiver", async () => {
    const newFeeReceiver = authority.publicKey;

    await program.methods
      .vaultSetFeeReceiver(
        newFeeReceiver,
      )
      .accountsPartial({
        mint: mintA.publicKey,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.feeReceiver.equals(newFeeReceiver)).toBeTrue();
  })

  it("set vault verifier", async () => {
    await program.methods
      .vaultSetVerifier(
        verifier
      )
      .accountsPartial({
        mint: mintA.publicKey,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.verifier).toEqual(verifier);
  })

  it("initialize minter managers", async () => {
    const minterManagerAdmin = authority.publicKey;

    await program.methods
      .minterManagerInitialize(
        minterManagerAdmin
      )
      .accounts({
        authority: authority.publicKey,
        payer: authority.publicKey,
        mint: mintA.publicKey,
      })
      .signers([authority])
      .rpc();

    const minterManagerAAcc = await program.account.minterManager.fetchNullable(minterManagerA);

    expect(minterManagerAAcc.admin.equals(minterManagerAdmin)).toBeTrue();

    await program.methods
      .minterManagerInitialize(
        minterManagerAdmin
      )
      .accounts({
        authority: authority.publicKey,
        payer: authority.publicKey,
        mint: mintB.publicKey,
      })
      .signers([authority])
      .rpc();
  })

  it("add minter to minter managers", async () => {
    const minter = authority.publicKey;

    await program.methods
      .minterManagerAddMinterByAdmin(
        minter
      )
      .accountsPartial({
        minterManager: minterManagerA,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const minterManagerAAcc = await program.account.minterManager.fetchNullable(minterManagerA);

    expect(minterManagerAAcc.minters).toContainEqual(authority.publicKey);

    await program.methods
      .minterManagerAddMinterByAdmin(
        minter
      )
      .accountsPartial({
        minterManager: minterManagerB,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();
  })

  it("add second minter to minter manager A", async () => {
    const newMinter = payer.publicKey;

    await program.methods
      .minterManagerAddMinterByAdmin(
        newMinter
      )
      .accountsPartial({
        minterManager: minterManagerA,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const minterManagerAAcc = await program.account.minterManager.fetchNullable(minterManagerA);

    expect(minterManagerAAcc.minters).toContainEqual(newMinter);
  })

  it("throws if minter already exists", async () => {
    const currentSlot = await connection.getSlot();
    await expireBlockhash(currentSlot);

    const existingMinter = authority.publicKey;

    try {
      await program.methods
        .minterManagerAddMinterByAdmin(
          existingMinter
        )
        .accountsPartial({
          minterManager: minterManagerA,
          payer: authority.publicKey,
          admin: authority.publicKey,
        })
        .signers([authority])
        .rpc();
    } catch (err) {
      expectError(err, "MinterAlreadyExists");
    }
  })

  it("removes minter from minter manager A", async () => {
    const minterToRemove = payer.publicKey;

    await program.methods
      .minterManagerRemoveMinterByAdmin(
        minterToRemove
      )
      .accountsPartial({
        minterManager: minterManagerA,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const minterManagerAAcc = await program.account.minterManager.fetchNullable(minterManagerA);

    expect(minterManagerAAcc.minters).not.toContainEqual(minterToRemove);
  })

  it("minter managers mint to accounts", async () => {
    const amount = 100_000_000;

    await program.methods
      .minterManagerMintTo(new BN(amount))
      .preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
          authority.publicKey,
          userAtaA,
          user.publicKey,
          mintA.publicKey
        )
      ])
      .accounts({
        authority: authority.publicKey,
        mint: mintA.publicKey,
        multisig: multisigA.publicKey,
        payer: authority.publicKey,
        to: userAtaA,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();

    const userAtaAAcc = await getAccount(connection, userAtaA);

    expect(userAtaAAcc.amount).toBe(BigInt(amount));

    await program.methods
      .minterManagerMintTo(new BN(amount))
      .preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
          authority.publicKey,
          userAtaB,
          user.publicKey,
          mintB.publicKey
        )
      ])
      .accountsPartial({
        authority: authority.publicKey,
        mint: mintB.publicKey,
        multisig: multisigB.publicKey,
        payer: authority.publicKey,
        to: userAtaB,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  })

  it("minter manager transfers admin", async () => {
    const newAdmin = admin.publicKey;

    await program.methods
      .minterManagerTransferAdmin(
        newAdmin
      )
      .accountsPartial({
        minterManager: minterManagerA,
        payer: authority.publicKey,
        admin: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const minterManagerAAcc = await program.account.minterManager.fetchNullable(minterManagerA);

    expect(minterManagerAAcc.admin.equals(newAdmin)).toBeTrue();
  })

  it("throws if slippage is exceeded", async () => {
    const amount = 5_000_000;
    const minimumAmount = amount + 100_000;

    try {
      await program.methods
        .vaultDeposit(
          new BN(amount),
          new BN(minimumAmount),
        )
        .preInstructions([
          createAssociatedTokenAccountIdempotentInstruction(
            user.publicKey,
            userAtaA,
            user.publicKey,
            mintA.publicKey
          ),
          createAssociatedTokenAccountIdempotentInstruction(
            user.publicKey,
            authorityAtaB,
            authority.publicKey,
            mintB.publicKey
          )
        ])
        .accounts({
          mintTarget: mintA.publicKey,
          mintToken: mintB.publicKey,
          user: user.publicKey,
          multisig: multisigA.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          userTargetTa: userAtaA,
          userTokenTa: userAtaB,
        })
        .signers([user])
        .rpc();
    } catch (err) {
      expectError(err, "SlippageExceeded");
    }
  })

  it("set NAV manager", async () => {
    const navManager = authority.publicKey;

    await program.methods
      .vaultSetNavManager(navManager)
      .accounts({
        oracleManager: authority.publicKey,
        vault: vaultA,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.oracleManager.equals(navManager)).toBeTrue();
  })

  it("set NAV", async () => {
    await program.methods
      .vaultSetNav(
        ONE_BITCOIN
      )
      .accounts({
        oracleManager: authority.publicKey,
        vault: vaultA,
      })
      .signers([authority])
      .rpc();

    const vaultAAcc = await program.account.vault.fetchNullable(vaultA);

    expect(vaultAAcc.nav.eq(ONE_BITCOIN)).toBeTrue();
  })

  it("throws if NAV is invalid", async () => {
    try {
      await program.methods
        .vaultSetNav(
          ONE_BITCOIN.subn(1)
        )
        .accounts({
          oracleManager: authority.publicKey,
          vault: vaultA,
        })
        .signers([authority])
        .rpc();
      throw new Error("Expected error was not thrown");
    } catch (err) {
      expectError(err, "InvalidNAVValue");
    }
  })

  it("deposit mint B to vault A", async () => {
    const amount = 5_000_000;
    const minimumAmount = 4_000_000;

    await program.methods
      .vaultDeposit(
        new BN(amount),
        new BN(minimumAmount)
      )
      .preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
          user.publicKey,
          userAtaA,
          user.publicKey,
          mintA.publicKey
        ),
        createAssociatedTokenAccountIdempotentInstruction(
          user.publicKey,
          authorityAtaB,
          authority.publicKey,
          mintB.publicKey
        )
      ])
      .accounts({
        mintTarget: mintA.publicKey,
        mintToken: mintB.publicKey,
        multisig: multisigA.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        user: user.publicKey,
        userTargetTa: userAtaA,
        userTokenTa: userAtaB,
      })
      .signers([user])
      .rpc();

    const userAtaAAcc = await getAccount(connection, userAtaA);

    expect(userAtaAAcc.amount).toBeGreaterThanOrEqual(BigInt(minimumAmount));
  })

  it("create withdraw request", async () => {
    await program.methods
      .vaultWithdrawRequest(
        Array.from(hash),
        new BN(500_000)
      )
      .accountsPartial({
        mintTarget: mintA.publicKey,
        mintWithdraw: mintB.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        user: user.publicKey,
        userTargetTa: userAtaA,
        userWithdrawTa: userAtaB,
        withdrawRequest,
      })
      .signers([user])
      .rpc();

    const withdrawRequestAcc = await program.account.withdrawRequest.fetchNullable(withdrawRequest);

    expect(withdrawRequestAcc.user.equals(user.publicKey)).toBeTrue();
    expect(withdrawRequestAcc.withdrawTokenAccount.equals(userAtaB)).toBeTrue();
    expect(withdrawRequestAcc.withdrawToken.equals(mintB.publicKey)).toBeTrue();
    expect(withdrawRequestAcc.token.equals(mintA.publicKey)).toBeTrue();
    expect(withdrawRequestAcc.requestHash).toEqual(Array.from(hash));
    expect(withdrawRequestAcc.withdrawAmount.gt(new BN(0))).toBeTrue();

    const vaultAcc = await program.account.vault.fetchNullable(vaultA);

    expect(withdrawRequestAcc.nav.eq(vaultAcc.nav)).toBeTrue();
  })

  it("rebalance liquidity from treasurer back to vault", async () => {
    const tx = new Transaction().add(
      createAssociatedTokenAccountIdempotentInstruction(
        authority.publicKey,
        vaultAAtaB,
        vaultA,
        mintB.publicKey
      ),
      createAssociatedTokenAccountIdempotentInstruction(
        authority.publicKey,
        feeReceiverAtaB,
        feeReceiver.publicKey,
        mintB.publicKey
      ),
      createTransferCheckedInstruction(
        authorityAtaB,
        mintB.publicKey,
        vaultAAtaB,
        authority.publicKey,
        600_000,
        8
      )
    )

    await sendAndConfirmTransaction(connection, tx, [authority]);
  })

  it("throws if signature is invalid", async () => {
    let withdrawRequestAcc = await program.account.withdrawRequest.fetchNullable(withdrawRequest);

    const verifierHash = getWithdrawRequestSigningHash(
      user.publicKey,
      mintB.publicKey,
      hash,
      withdrawRequestAcc.shares,
      withdrawRequestAcc.nav,
    )

    const randomBytes = Buffer.from(crypto.getRandomValues(new Uint8Array(32))).toString("hex");
    const invalidSignature = Buffer.from(randomBytes, "hex")

    const signature = createWithdrawSignature(
      invalidSignature,
      verifierHash
    )

    try {
      await program.methods
      .vaultWithdraw(
        Array.from(hash),
        signature.signature
      )
      .accountsPartial({
        mintWithdraw: mintB.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        user: user.publicKey,
        userWithdrawTa: userAtaB,
        withdrawRequest,
        vault: vaultA,
      })
      .signers([user])
      .rpc();
    } catch (err) {
      expectError(err, "MissingRequiredSignature");
    }
  })

  it("process withdraw request", async () => {
    let withdrawRequestAcc = await program.account.withdrawRequest.fetchNullable(withdrawRequest);

    const verifierHash = getWithdrawRequestSigningHash(
      user.publicKey,
      mintB.publicKey,
      hash,
      withdrawRequestAcc.shares,
      withdrawRequestAcc.nav,
    )

    const signature = createWithdrawSignature(
      verifierKeypair,
      verifierHash
    )

    const preAuthorityAtaBBal = (await getAccount(connection, authorityAtaB)).amount;
    const preUserAtaBBal = (await getAccount(connection, userAtaB)).amount;

    await program.methods
      .vaultWithdraw(
        Array.from(hash),
        signature.signature
      )
      .accountsPartial({
        mintWithdraw: mintB.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        user: user.publicKey,
        userWithdrawTa: userAtaB,
        withdrawRequest,
        vault: vaultA,
      })
      .signers([user])
      .rpc();

    const postAuthorityAtaBBal = (await getAccount(connection, authorityAtaB)).amount;

    expect(preAuthorityAtaBBal).toBeLessThan(postAuthorityAtaBBal);

    const postUserAtaBBal = (await getAccount(connection, userAtaB)).amount;

    expect(preUserAtaBBal).toBeLessThan(postUserAtaBBal);

    withdrawRequestAcc = await program.account.withdrawRequest.fetchNullable(withdrawRequest);

    expect(withdrawRequestAcc).toBeNull();
  })
})