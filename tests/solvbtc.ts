import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Solvbtc } from "../target/types/solvbtc";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram, Transaction, ComputeBudgetProgram, sendAndConfirmTransaction } from "@solana/web3.js";
import { createWithdrawRequestHash, createWithdrawSignature, deriveMinterManagerAddress, derivePoolSignerAddress, deriveVaultAddress, deriveWithdrawRequestAddress, 
  deriveWithdrawRequestSigningHash, ecdsaPubkeyFromPrivkey, ONE_BITCOIN,
deriveWithdrawRequestEip191, createEip191WithdrawSig } from "../sdk/solvbtc";
import { BN } from "bn.js";
import { createAssociatedTokenAccountIdempotentInstruction, createInitializeMint2Instruction, createInitializeMultisigInstruction, createTransferCheckedInstruction, getAssociatedTokenAddressSync, getMinimumBalanceForRentExemptMint, getMinimumBalanceForRentExemptMultisig, MINT_SIZE, MULTISIG_SIZE, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("solvbtc", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();

  const connection = provider.connection;

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  const program = anchor.workspace.solvbtc as Program<Solvbtc>;

  /// Signers

  //3GhdJHto7UsUH7sHUtJfqivdsaTc8nAeGi2vPyiQZKFY
  const authorityKeypair = Keypair.fromSecretKey(new Uint8Array([
    50, 113, 208, 51, 176, 21, 27, 129, 26, 20, 53,
    124, 179, 130, 206, 138, 228, 170, 199, 24, 193, 60,
    253, 134, 123, 125, 163, 189, 9, 185, 55, 19, 33,
    189, 5, 146, 32, 116, 24, 195, 49, 105, 110, 174,
    167, 205, 203, 23, 17, 29, 199, 83, 33, 241, 105,
    206, 179, 236, 186, 72, 144, 232, 119, 227
  ]));
  const adminKeypair = Keypair.generate()
  const payerKeypair = Keypair.generate()
  const feeReceiverKeypair = Keypair.generate()
  const treasurerKeypair = Keypair.generate()
  const oracleManagerKeypair = Keypair.generate()
  const userKeypair = Keypair.generate()
  const mintAKeypair = Keypair.generate()
  const mintBKeypair = Keypair.generate()
  const multisigAKeypair = Keypair.generate()
  const multisigBKeypair = Keypair.generate()

  const authority = authorityKeypair.publicKey
  const admin = adminKeypair.publicKey;
  const payer = payerKeypair.publicKey
  const feeReceiver = feeReceiverKeypair.publicKey
  const treasurer = treasurerKeypair.publicKey
  const oracleManager = oracleManagerKeypair.publicKey
  const user = userKeypair.publicKey
  const mintA = mintAKeypair.publicKey
  const mintB = mintBKeypair.publicKey
  const multisigA = multisigAKeypair.publicKey
  const multisigB = multisigBKeypair.publicKey
  const poolSignerA = derivePoolSignerAddress(mintA)
  const poolSignerB = derivePoolSignerAddress(mintB)
  const vaultA = deriveVaultAddress(mintA)
  const vaultB = deriveVaultAddress(mintB)
  const minterManagerA = deriveMinterManagerAddress(vaultA)
  const minterManagerB = deriveMinterManagerAddress(vaultB)

  // Withdraw request
  const hash = createWithdrawRequestHash();
  const withdrawRequest = deriveWithdrawRequestAddress(vaultA, mintB, user, hash);

  // Programs
  const tokenProgram = TOKEN_PROGRAM_ID
  const associatedTokenProgram = ASSOCIATED_PROGRAM_ID
  const systemProgram = SystemProgram.programId

  const userAtaA = getAssociatedTokenAddressSync(mintA, user)
  const userAtaB = getAssociatedTokenAddressSync(mintB, user)
  const feeReceiverAtaB = getAssociatedTokenAddressSync(mintB, feeReceiver)
  const authorityAtaA = getAssociatedTokenAddressSync(mintA, authority)
  const authorityAtaB = getAssociatedTokenAddressSync(mintB, authority)
  const vaultAAtaA = getAssociatedTokenAddressSync(mintA, vaultA, true)
  const vaultAAtaB = getAssociatedTokenAddressSync(mintB, vaultA, true)

  /// ECDSA Keys
  let verifierKeypair = Buffer.from("c2fffbf8e5cec943afb99e8194a5819c64c9df75b4ed03b2a111e8ccdcf55689", "hex")
  let verifier = Array.from(ecdsaPubkeyFromPrivkey(verifierKeypair).subarray(1));
  console.log(Buffer.from(verifier).toString("hex"));
  const accounts = {
    authority,
    admin,
    payer,
    feeReceiver,
    treasurer,
    oracleManager,
    mintA,
    mintB,
    multisigA,
    multisigB,
    user,
    withdrawRequest,
    tokenProgram,
    associatedTokenProgram,
    systemProgram
  }

  it("Airdrop lamports", async () => {
    let tx = new Transaction();
    tx.instructions = [authority, admin, user, payer, feeReceiver, treasurer, oracleManager].map((account) =>
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: account,
        lamports: 10 * LAMPORTS_PER_SOL,
      })
    )
    await provider.sendAndConfirm(tx, []).then(log);
  })

  it("Set up Mints", async () => {
    const lamports = await getMinimumBalanceForRentExemptMint(connection);
    const lamportsMultisig = await getMinimumBalanceForRentExemptMultisig(connection);
    let tx = new Transaction();
    tx.instructions = [
      ...[multisigA, multisigB].map((account) =>
        SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: account,
          space: MULTISIG_SIZE,
          lamports: lamportsMultisig,
          programId: tokenProgram,
        })
      ),
      ...[mintA, mintB].map((mint) =>
        SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: mint,
          lamports,
          space: MINT_SIZE,
          programId: tokenProgram,
        })
      ),
      createInitializeMultisigInstruction(
        multisigA,
        [
          vaultA,
          poolSignerA
        ],
        1,
        tokenProgram
      ),
      createInitializeMultisigInstruction(
        multisigB,
        [
          vaultB,
          poolSignerB
        ],
        1,
        tokenProgram
      ),
      createInitializeMint2Instruction(mintA, 8, multisigA, multisigA),
      createInitializeMint2Instruction(mintB, 8, multisigB, multisigB),
    ];

    await provider.sendAndConfirm(tx, [mintAKeypair, mintBKeypair, multisigAKeypair, multisigBKeypair]).then(log);
  });

  it("Initialize Vault A", async () => {
    // Add your test here.
    const tx = await program.methods.vaultInitialize(
      admin,
      feeReceiver,
      treasurer,
      Array.from(verifier),
      oracleManager,
      ONE_BITCOIN,
      50,
    )
      .accountsStrict({
        ...accounts,
        payer: authority,
        mint: mintA,
        vault: vaultA,
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Initialize Vault B", async () => {
    // Add your test here.
    const tx = await program.methods.vaultInitialize(
      authority,
      authority,
      authority,
      Array.from(verifier),
      authority,
      ONE_BITCOIN,
      50,
    )
      .accountsStrict({
        ...accounts,
        payer: authority,
        mint: mintB,
        vault: vaultB,
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Transfer vault admin", async () => {
    const tx = await program.methods.vaultTransferAdmin(
      authority
    )
      .accountsStrict({
        ...accounts,
        payer: admin,
        mint: mintA,
        vault: vaultA,
      })
      .signers([adminKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Set vault treasurer", async () => {
    const tx = await program.methods.vaultSetTreasurer(
      authority
    )
      .accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        mint: mintA,
        vault: vaultA,
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Add vault currency", async () => {
    const tx = await program.methods.vaultAddCurrency(
      mintB,
      75
    )
      .accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        vault: vaultA,
        mint: mintA
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Fail to add existing vault currency", async () => {
    try {
      await program.methods.vaultAddCurrency(
        mintB,
        50
      )
        .accountsStrict({
          ...accounts,
          admin: authority,
          payer: authority,
          vault: vaultA,
          mint: mintA
        })
        .signers([authorityKeypair])
        .rpc()
        .then(confirm)
        .then(log)
      throw new Error("This test shouldn't have passed!")
    } catch (e) {
      if (e.error.errorMessage != "SolvVault: Currency already exists") {
        throw new Error("Unexpected error message")
      }
    }
  })

  it("Add vault currency for removal", async () => {
    const tx = await program.methods.vaultAddCurrency(
      mintA,
      50
    )
      .accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        vault: vaultA,
        mint: mintA
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Remove vault currency", async () => {
    const tx = await program.methods.vaultRemoveCurrency(
      mintA
    )
      .accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        vault: vaultA,
        mint: mintA
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Set vault withdraw fee", async () => {
    const tx = await program.methods.vaultSetWithdrawFee(
      500
    )
      .accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        vault: vaultA,
        mint: mintA
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

    it("Set vault deposit fee for mint B", async () => {
    const tx = await program.methods.vaultSetDepositFee(
      mintB,
      500
    )
      .accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        vault: vaultA,
        mint: mintA
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Set vault fee receiver", async () => {
    const tx = await program.methods.vaultSetFeeReceiver(
      authority
    )
      .accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        vault: vaultA,
        mint: mintA
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Set vault verifier", async () => {
    const tx = await program.methods.vaultSetVerifier(
      verifier
    )
      .accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        vault: vaultA,
        mint: mintA
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
  });

  it("Initialize minter manager A", async () => {
    const tx = await program.methods.minterManagerInitialize(
      authority
    ).accountsStrict({
      ...accounts,
      payer: authority,
      minterManager: minterManagerA,
      vault: vaultA,
      mint: mintA
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Initialize minter manager B", async () => {
    const tx = await program.methods.minterManagerInitialize(
      authority
    ).accountsStrict({
      ...accounts,
      payer: authority,
      minterManager: minterManagerB,
      vault: vaultB,
      mint: mintB
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Add minter to minter manager A", async () => {
    const tx = await program.methods.minterManagerAddMinterByAdmin(
      authority
    ).accountsStrict({
      ...accounts,
      admin: authority,
      payer: authority,
      minterManager: minterManagerA,
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Add minter to minter manager B", async () => {
    const tx = await program.methods.minterManagerAddMinterByAdmin(
      authority
    ).accountsStrict({
      ...accounts,
      admin: authority,
      payer: authority,
      minterManager: minterManagerB,
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Add second minter to minter manager A", async () => {
    const tx = await program.methods.minterManagerAddMinterByAdmin(
      payer
    ).accountsStrict({
      ...accounts,
      admin: authority,
      payer: authority,
      minterManager: minterManagerA,
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Fail to add existing minter to minter manager", async () => {
    try {
      const tx = await program.methods.minterManagerAddMinterByAdmin(
        authority
      ).accountsStrict({
        ...accounts,
        admin: authority,
        payer: authority,
        minterManager: minterManagerA,
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)
      throw new Error("This shouldn't have succeeded")
    } catch(e) {
      if (e.error.errorMessage != "SolvMinterManager: Minter already exists") {
        throw new Error("Unexpected error message")
      }
    }
  })

  it("Remove minter from minter manager A", async () => {
    const tx = await program.methods.minterManagerRemoveMinterByAdmin(
      payer
    ).accountsStrict({
      ...accounts,
      admin: authority,
      payer: authority,
      minterManager: minterManagerA,
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Minter manager A mint to account", async () => {
    const tx = await program.methods.minterManagerMintTo(
      new BN(100_000_000)
    )
    .preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
        authority,
        userAtaA,
        user,
        mintA
      )
    ])
    .accountsStrict({
      ...accounts,
      payer: authority,
      minterManager: minterManagerA,
      vault: vaultA,
      mint: mintA,
      multisig: multisigA,
      to: userAtaA
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Minter manager B mint to account", async () => {
    const tx = await program.methods.minterManagerMintTo(
      new BN(100_000_000)
    )
    .preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
        authority,
        userAtaB,
        user,
        mintB
      )
    ])
    .accountsStrict({
      ...accounts,
      payer: authority,
      minterManager: minterManagerB,
      vault: vaultB,
      mint: mintB,
      multisig: multisigB,
      to: userAtaB
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Minter manager transfer admin", async () => {
    const tx = await program.methods.minterManagerTransferAdmin(
      admin
    )
    .accountsStrict({
      ...accounts,
      admin: authority,
      payer: authority,
      minterManager: minterManagerB,
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Fail to deposit Token B to vault A due to slippage", async () => {
    try {
        const tx = await program.methods.vaultDeposit(
        new BN(5_000_000), 
        new BN(5_100_000)
      )
      .preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          userAtaA,
          user,
          mintA
        ),
        createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          authorityAtaB,
          authority,
          mintB
        )
      ])

      .accountsStrict({
        ...accounts,
        vault: vaultA,
        multisig: multisigA,
        userTokenTa: userAtaB,
        userTargetTa: userAtaA,
        treasurerTokenTa: authorityAtaB,
        // feeReceiverTokenTa: authorityAtaB,
        mintToken: mintB,
        mintTarget: mintA,
      })
      .signers([userKeypair])
      .rpc()
      .then(confirm)
      .then(log)
      throw new Error("This shouldn't succeed")
    } catch(e) {
      if (e.error.errorMessage != "SolvVault: Slippage exceeded") {
        throw new Error("Unexpected error message")
      }
    }
  })

    it("Set NAV manager", async () => {
    const tx = await program.methods.vaultSetNavManager(
      authority
    )
    .accountsStrict({
      oracleManager: authority,
      vault: vaultA
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  });

  it("Set and Check NAV", async () => {
    const tx = await program.methods.vaultSetNav(
      ONE_BITCOIN
    )
    .accountsStrict({
      ...accounts,
      oracleManager: authority,
      vault: vaultA
    })
    .signers([authorityKeypair])
    .rpc()
    .then(confirm)
    .then(log)

    const vault = await program.account.vault.fetch(
      vaultA
    )

    if (vault.nav.toNumber() != ONE_BITCOIN.toNumber()) {
      throw new Error("Invalid NAV")
    }
  });

  it("Increase NAV", async () => {
      const tx = await program.methods.vaultSetNav(
        ONE_BITCOIN.add(new BN(1))
      )
      .accountsStrict({
        ...accounts,
        oracleManager: authority,
        vault: vaultA
      })
      .signers([authorityKeypair])
      .rpc()
      .then(confirm)
      .then(log)

  });
  
  it("Deposit Token B to vault A", async () => {
    const tx = await program.methods.vaultDeposit(
      new BN(5_000_000), 
      new BN(4_000_000)
    )
    .preInstructions([
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        userAtaA,
        user,
        mintA
      ),
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        authorityAtaB,
        authority,
        mintB
      )
    ])

    .accountsStrict({
      ...accounts,
      vault: vaultA,
      multisig: multisigA,
      userTokenTa: userAtaB,
      userTargetTa: userAtaA,
      treasurerTokenTa: authorityAtaB,
      // feeReceiverTokenTa: authorityAtaB,
      mintToken: mintB,
      mintTarget: mintA,
    })
    .signers([userKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Create withdraw request", async () => {
    const tx = await program.methods.vaultWithdrawRequest(
      Array.from(hash),
      new BN(500_000)
    )
    .accountsStrict({
      ...accounts,
      vault: vaultA,
      userTargetTa: userAtaA,
      mintTarget: mintA,
      userWithdrawTa: userAtaB,
      mintWithdraw: mintB
    })
    .signers([userKeypair])
    .rpc()
    .then(confirm)
    .then(log)
  })

  it("Rebalance liquidity from treasurer back to vault", async () => {
    let tx = new Transaction();
    tx.instructions = [
      createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          vaultAAtaB,
          vaultA,
          mintB
        ),
        createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          feeReceiverAtaB,
          feeReceiver,
          mintB
        ),
        createTransferCheckedInstruction(
          authorityAtaB,
          mintB,
          vaultAAtaB,
          authority,
          600_000,
          8
        )
      ]
      await provider.sendAndConfirm(tx, [authorityKeypair]).then(log);
  })

  it("Process withdraw request", async () => {
    const withdrawRequestData = await program.account.withdrawRequest.fetch(withdrawRequest);

    const verifierHash = deriveWithdrawRequestEip191(
      user,
      mintB,
      hash,
      withdrawRequestData.shares,
      withdrawRequestData.nav,
    )

    const signature = createEip191WithdrawSig(
      verifierKeypair,
      verifierHash
    )
    const ix = await program.methods.vaultWithdraw(
      Array.from(hash),
      signature.signature
    )
    .accountsStrict({
      ...accounts,
      vault: vaultA,
      userWithdrawTa: userAtaB,
      mintWithdraw: mintB,
      vaultWithdrawTa: vaultAAtaB,
      feeReceiverTa: authorityAtaB
    }).instruction();

    const tx = new Transaction();
    tx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }));
    tx.add(ix);
    await sendAndConfirmTransaction(connection, tx, [userKeypair]);




  })
});
