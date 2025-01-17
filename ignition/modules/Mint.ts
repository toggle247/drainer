import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

const MintModule = buildModule("Mint", (module) => {
  const name = module.getParameter("name", "Test Mint");
  const symbol = module.getParameter("symbol", "TMT");
  const initialSupply = module.getParameter(
    "initialSupply",
    100000n * BigInt(Math.pow(10, 18))
  );

  const mint = module.contract("Mint", [name, symbol, initialSupply]);

  return { mint };
});

export default MintModule;
