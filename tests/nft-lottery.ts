import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { NftLottery } from "../target/types/nft_lottery"
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js"
import {
  createMint,
  getAssociatedTokenAddressSync,
  mintTo,
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token"
import { Orao } from "@orao-network/solana-vrf"

describe("NFT Lottery Test Cases", async () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.nftLottery as Program<NftLottery>
  const connection = provider.connection
  const dev = provider.wallet.payer

  // Orao Setup
  const orao = new Orao(provider)
  let vrfTreasury: PublicKey

  // Variables
  let creator: Keypair
  let buyer1: Keypair
  let buyer2: Keypair
  let buyer3: Keypair
  let nftMint: PublicKey
  let creatorNftAccount: PublicKey
  let lotteryPda: PublicKey
  let lotteryNftPda: PublicKey

  // Parameters
  const force = Buffer.alloc(32)
  crypto.getRandomValues(force)

  before(async () => {
    // Generate Keypair
    creator = dev
    console.log(`${creator.publicKey}`)
    buyer1 = Keypair.generate()
    buyer2 = Keypair.generate()
    buyer3 = Keypair.generate()

    // Transfer from dev to buyer1
    let transferTx = await anchor.web3.sendAndConfirmTransaction(
      connection,
      new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: dev.publicKey,
          toPubkey: buyer1.publicKey,
          lamports: 0.1 * LAMPORTS_PER_SOL,
        })
      ),
      [dev]
    )
    console.log("Transferred SOL to buyer1: ", transferTx)

    // Orao setup
    const networkState = await orao.getNetworkState()
    vrfTreasury = networkState.config.treasury

    // NftMint
    nftMint = await createMint(connection, dev, creator.publicKey, null, 0)

    // AssociatedTokenAddress for creator
    creatorNftAccount = await createAssociatedTokenAccount(connection, dev, nftMint, creator.publicKey)

    // Mint 1 NFT token to creator
    await mintTo(connection, dev, nftMint, creatorNftAccount, creator, 1)

    // Derive PDAs
    ;[lotteryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("lottery"), creator.publicKey.toBuffer(), nftMint.toBuffer()],
      program.programId
    )
    lotteryNftPda = await getAssociatedTokenAddressSync(nftMint, lotteryPda, true)
  })

  it("Initialize Lottery", async () => {
    const ticket_price = new anchor.BN(0.05 * LAMPORTS_PER_SOL)
    const current_time = Math.floor(Date.now() / 1000)
    const start_time = new anchor.BN(current_time)
    const end_time = new anchor.BN(current_time + 3)
    const initTx = await program.methods
      .createLottery(ticket_price, start_time, end_time, Array.from(force))
      .accounts({
        creator: creator.publicKey,
        nftMint: nftMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([creator])
      .rpc({ commitment: "confirmed" })

    console.log("Transaction Confirmed: ", initTx)
  })

  it("Buying lottery ticket", async () => {
    const lotteryAccount = await program.account.lottery.fetch(lotteryPda)

    const buyTx = await program.methods
      .buyTicket()
      .accounts({
        buyer: buyer1.publicKey,
        lottery: lotteryPda,
      })
      .signers([buyer1])
      .rpc({ commitment: "confirmed" })

    console.log("Tickets purchased: ", buyTx)
  })

  it("Reveal Winner", async () => {
    await sleep(3000)
    const requestRandomnessTx = await program.methods
      .requestRandomness()
      .accounts({
        creator: creator.publicKey,
        lottery: lotteryPda,
        vrfTreasury: vrfTreasury,
      })
      .signers([creator])
      .rpc()

    console.log("Randomness requested: ", requestRandomnessTx)

    await orao.waitFulfilled(force)

    // Get randomness Account
    const randomness = await orao.getRandomness(force)
    const randomValue = randomness.getFulfilledRandomness()

    // Get lottery ticket
    const lotteryAccount = await program.account.lottery.fetch(lotteryPda)
    const lotteryTicket = lotteryAccount.ticketsSold.toNumber()

    // Calculate Winning ticket number
    const randomBytes = Buffer.from(randomValue.slice(0, 8))
    const randomU64 = randomBytes.readBigInt64LE(0)
    const remainder = Number(randomU64 % BigInt(lotteryTicket))

    console.log(` Winning Ticket : ${remainder}`)

    // Derive the User lottery ticket
    const [winningTicketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("ticket"), lotteryPda.toBuffer(), new anchor.BN(remainder).toArrayLike(Buffer, "le", 8)],
      program.programId
    )

    const pickWinnerTx = await program.methods
      .pickWinner()
      .accounts({
        creator: creator.publicKey,
        lottery: lotteryPda,
        winningTicket: winningTicketPda,
      })
      .signers([creator])
      .rpc({ commitment: "confirmed" })

    console.log("Winner Picked Success : ", pickWinnerTx)
  })
  it("Transfer NFT to Winner ", async () => {
    const winnerToken = await createAssociatedTokenAccount(connection, buyer1, nftMint, buyer1.publicKey)
    const rewardWinnerTx = await program.methods
      .rewardWinner()
      .accounts({
        winner: buyer1.publicKey,
        lottery: lotteryPda,
        nftLotteryVault: lotteryNftPda,
        nftMint: nftMint,
        winnerNft: winnerToken,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([buyer1])
      .rpc({ commitment: "confirmed" })

    console.log("The winner has been rewarded: ", rewardWinnerTx)
  })
})

// Sleep utility
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}
