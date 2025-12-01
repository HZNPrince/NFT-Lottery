import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { NftLottery } from "../target/types/nft_lottery"

describe("NFT Lottery Test Cases", async () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.nftLottery as Program<NftLottery>
  const connection = provider.connection
  const payer = provider.wallet.payer
})
