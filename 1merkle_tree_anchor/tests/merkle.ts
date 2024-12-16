import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Merkle } from "../target/types/merkle_tree";
import { expect } from 'chai';

describe("merkle", () => {
  const program = anchor.workspace.Merkle as Program<Merkle>;
  const provider = anchor.getProvider();
  const merkleAccount = anchor.web3.Keypair.generate();

  it("Инициализирует дерево Меркла", async () => {
    const tx = await program.methods
      .initialize()
      .accounts({
        merkleAccount: merkleAccount.publicKey,
        user: provider.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([merkleAccount])
      .rpc();

    console.log("Транзакция инициализации:", tx);

    const account = await program.account.merkleAccount.fetch(merkleAccount.publicKey);
    expect(account.leafCount).to.equal(0);
    expect(account.leaves).to.have.lengthOf(0);
    expect(account.root).to.deep.equal(new Array(32).fill(0));
  });

  it("Добавляет лист в дерево", async () => {
    const testLeaf = new Array(32).fill(1);

    const tx = await program.methods
      .insertLeaf(testLeaf)
      .accounts({
        merkleAccount: merkleAccount.publicKey,
        authority: provider.publicKey,
      })
      .rpc();

    console.log("Транзакция добавления листа:", tx);

    const account = await program.account.merkleAccount.fetch(merkleAccount.publicKey);
    expect(account.leafCount).to.equal(1);
    expect(account.leaves).to.have.lengthOf(1);
    expect(account.leaves[0]).to.deep.equal(testLeaf);
  });

  it("Проверяет доказательство для листа", async () => {
    const leaf2 = new Array(32).fill(2);
    await program.methods
      .insertLeaf(leaf2)
      .accounts({
        merkleAccount: merkleAccount.publicKey,
        authority: provider.publicKey,
      })
      .rpc();

    const account = await program.account.merkleAccount.fetch(merkleAccount.publicKey);
    
    const proof = [leaf2];
    const path = [false];

    const tx = await program.methods
      .verifyProof(account.leaves[0], proof, path)
      .accounts({
        merkleAccount: merkleAccount.publicKey,
      })
      .rpc();

    console.log("Транзакция проверки доказательства:", tx);
  });

  it("Отслеживает события при добавлении листа", async () => {
    const listener = program.addEventListener("LeafInserted", (event, slot) => {
      console.log("Получено событие LeafInserted:", event);
      expect(event.leaf).to.deep.equal(new Array(32).fill(3));
    });

    const newLeaf = new Array(32).fill(3);
    await program.methods
      .insertLeaf(newLeaf)
      .accounts({
        merkleAccount: merkleAccount.publicKey,
        authority: provider.publicKey,
      })
      .rpc();

    await new Promise(resolve => setTimeout(resolve, 1000));
    await program.removeEventListener(listener);
  });

  it("Проверяет ограничение на максимальное количество листьев", async () => {
    const leaf = new Array(32).fill(4);
    let errorOccurred = false;

    try {
      for (let i = 0; i < 300; i++) {
        await program.methods
          .insertLeaf(leaf)
          .accounts({
            merkleAccount: merkleAccount.publicKey,
            authority: provider.publicKey,
          })
          .rpc();
      }
    } catch (error) {
      errorOccurred = true;
      expect(error.toString()).to.include("TreeFull");
    }

    expect(errorOccurred).to.be.true;
  });
});
