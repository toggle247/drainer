import hre from "hardhat";
import type { Address } from "viem";

const main = async (mint: Address) => {
  const client = await hre.viem.getPublicClient();
  const [from, to] = await hre.viem.getWalletClients();
  console.log(from.account.address);
  console.log(to.account.address);

  console.log(await client.getBalance({ address: from.account.address }));

  const contract = await hre.viem.getContractAt("Mint", mint);

  const hash = await contract.write.transfer([to.account.address, 1000n]);
  const tx = await client.waitForTransactionReceipt({ hash });
  console.log(tx.logs);
  console.log(await contract.read.balanceOf([from.account.address]));
};

main("0x5FbDB2315678afecb367f032d93F642f64180aa3");
