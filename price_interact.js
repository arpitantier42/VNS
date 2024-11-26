import { CodePromise, Abi, ContractPromise } from '@polkadot/api-contract';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { BN, BN_ONE, BN_ZERO } from '@polkadot/util';
import colors from "colors";
import { json } from "./abi.js";

async function main() {

    const wsProvider = new WsProvider('ws://54.219.1.159:9944');
    const api = await ApiPromise.create({ provider: wsProvider });

    const gasLimit = api.registry.createType("WeightV2", {
        refTime: new BN("1000000000000"),
        proofSize: new BN("1000000000000"),
    });
    
    const storageDepositLimit = null;

    const contractAddress = '0xfF8D6265650Dc95167555f9CaBb09e4bc2436962';
    const contract = new ContractPromise(api, json, contractAddress);
    console.log('Available contract methods:'.cyan, Object.keys(contract.tx));

    const keyring = new Keyring({ type: 'ethereum' });
    const userKeyring = keyring.addFromUri('0xd615610ab9435f7be2c82e87d6f62c06c75aeb63a2ba6a5cb55260a38ae4cfa4');

    async function calculate_price(name, duration) {
        const { result, gasConsumed, output } = await contract.query["calculatePrice"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
            name,
            duration
        );
        console.log(gasConsumed.toHuman(), "gas used");
        if (result.isOk) {
            const calculatedPrice = output.toHuman()
            console.log("calculatedPrice is : ".yellow, calculatedPrice.Ok);
            return calculatedPrice;
        } else {
            console.error('Failed to calculate price:', output);
            return null;
        }
    }

    async function read_owner() {
        const { result, gasUsed, output } = await contract.query["readOwner"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const ownerAddress = output.toHuman()
            console.log("owner address is : ".yellow, ownerAddress.Ok);
            return ownerAddress;
        } else {
            console.error('Failed to read owner address:', output);
            return null;
        }
    }

    async function get_price_per_year() {
        const { result, gasUsed, output } = await contract.query["getPricePerYear"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const price = output.toHuman()
            console.log("price per year : ".yellow, price.Ok);
            return price;
        } else {
            console.error('Failed to read price:', output);
            return null;
        }
    }

    async function get_price_per_letter() {
        const { result, gasUsed, output } = await contract.query["getPricePerLetter"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const price = output.toHuman()
            console.log("price per letter : ".yellow, price.Ok);
            return price;
        } else {
            console.error('Failed to read letter:', output);
            return null;
        }
    }

    async function read_premium_names() {
        const { result, gasUsed, output } = await contract.query["getPremiumNames"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const ownerAddress = output.toHuman()
            console.log("premium domains are  : ".yellow, ownerAddress.Ok);
            return ownerAddress;
        } else {
            console.error('Failed to read premium domains:', output);
            return null;
        }
    }

    async function set_price_per_letter(price) {
        await contract.tx
            .setPricePerLetter({ storageDepositLimit, gasLimit }, price)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function set_price_per_year(price) {
        await contract.tx
            .setPricePerYear({ storageDepositLimit, gasLimit }, price)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function add_premium_name(name) {
        await contract.tx
            .addPremiumName({ storageDepositLimit, gasLimit }, name)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function remove_premium_name(name) {
        await contract.tx
            .removePremiumName({ storageDepositLimit, gasLimit }, name)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    await calculate_price("akt.vne", 300000);
    // await read_owner();
    // await set_price_per_letter(20000000000000);
    // await set_price_per_year(20000000000000);
    // await add_premium_name("antiers.vne");
    // await read_premium_names();
    // await remove_premium_name("antiers.vne");
    await get_price_per_year();
    await get_price_per_letter();

}

main()

