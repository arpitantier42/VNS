import { CodePromise, Abi, ContractPromise } from '@polkadot/api-contract';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { BN, BN_ONE, BN_ZERO } from "@polkadot/util";
import { json } from "./abi.js";
import dotenv from "dotenv";

dotenv.config()

const wsProvider = new WsProvider('ws://54.219.1.159:9944');
const api = await ApiPromise.create({ provider: wsProvider });
const code = new CodePromise(api, json, json.source.wasm);

const gasLimit = api.registry.createType("WeightV2", {
    refTime: new BN("1000000000000"),
    proofSize: new BN("100000000000"),
});

const storageDepositLimit = null;
const keyring = new Keyring({ type: "ethereum" });
const userKeyring = keyring.addFromUri('0xd615610ab9435f7be2c82e87d6f62c06c75aeb63a2ba6a5cb55260a38ae4cfa4');

// const tx = code.tx['new']({ value: 0, gasLimit: gasLimit, storageDepositLimit }, userKeyring.address, 120000, 60000, 120000, "0x628BF79707D5c0bAF7d63227D842CB70612c7830", "0xfF8D6265650Dc95167555f9CaBb09e4bc2436962", "0x628BF79707D5c0bAF7d63227D842CB70612c7830"); //registrar cotractn
// const tx = code.tx['new']({ value: 0, gasLimit: gasLimit, storageDepositLimit }, userKeyring.address, userKeyring.address, 120000); // resolver contract
// const tx = code.tx['new']({ value: 0, gasLimit: gasLimit, storageDepositLimit }, "0x628BF79707D5c0bAF7d63227D842CB70612c7830"); // nft contract

const unsub = await tx.signAndSend(userKeyring, { signer: userKeyring }, ({ contract, status, events }) => {
    console.log('status', status.toHuman())

    if (contract) {
        const addr = events.filter(e => e.event.method == 'Instantiated')[0].event.data.toHuman().contract;
        console.log('Contract address: ', addr)
        unsub()
    }
})



