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

    const contractAddress = '0x2370bcbC0c5b90AfB12e36Cf1936d45a22c7EDD8';
    const contract = new ContractPromise(api, json, contractAddress);
    console.log('Available contract methods:'.cyan, Object.keys(contract.tx));

    const keyring = new Keyring({ type: 'ethereum' });
    const userKeyring = keyring.addFromUri('0xd615610ab9435f7be2c82e87d6f62c06c75aeb63a2ba6a5cb55260a38ae4cfa4');

    async function read_owner() {
        const { result, gasUsed, output } = await contract.query["readAdmin"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const ownerAddress = output.toHuman()
            console.log("admin address is : ".yellow, ownerAddress.Ok);
            return ownerAddress;
        } else {
            console.error('Failed to read admin address:', output);
            return null;
        }
    }

    async function read_resolver() {
        const { result, gasUsed, output } = await contract.query["readResolver"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const resolverAddress = output.toHuman()
            console.log("resolver address is : ".yellow, resolverAddress.Ok);
            return resolverAddress;
        } else {
            console.error('Failed to read resolver address:', output);
            return null;
        }
    }

    async function read_current_timestamp() {
        const { result, gasUsed, output } = await contract.query["currentTimestamp"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const currentTimestamp = output.toHuman()
            console.log("current timestamp is : ".yellow, currentTimestamp.Ok);
            return currentTimestamp;
        } else {
            console.error('Failed to read current timestamp :', output);
            return null;
        }
    }

    async function read_max_commit_age() {
        const { result, gasUsed, output } = await contract.query["readMaxCommitAge"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const maxCommitAge = output.toHuman()
            console.log("maxCommitAge is : ".yellow, maxCommitAge.Ok);
            return maxCommitAge;
        } else {
            console.error('Failed to read maxCommitAge :', output);
            return null;
        }
    }

    async function read_min_commit_age() {
        const { result, gasUsed, output } = await contract.query["readMinCommitAge"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const MinCommitAge = output.toHuman()
            console.log("MinCommitAge is : ".yellow, MinCommitAge.Ok);
            return MinCommitAge;
        } else {
            console.error('Failed to read MinCommitAge :', output);
            return null;
        }
    }

    async function read_min_registration_duration() {
        const { result, gasUsed, output } = await contract.query["readMinRegistrationDuration"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const MinRegistrationDuration = output.toHuman()
            console.log("MinRegistrationDuration is : ".yellow, MinRegistrationDuration.Ok);
            return MinRegistrationDuration;
        } else {
            console.error('Failed to read MinRegistrationDuration :', output);
            return null;
        }
    }

    async function read_domain_price(name, duration) {
        const { result, gasUsed, output } = await contract.query["readDomainPrice"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, name, duration
        );

        if (result.isOk) {
            const DomainPrice = output.toHuman()
            console.log("domain price is : ".yellow, DomainPrice.Ok);
            return DomainPrice;
        } else {
            console.error('Failed to read DomainPrice :', output);
            return null;
        }
    }

    async function make_commitment(domain_name, domain_owner, duration, secret, resolver) {
        const { result, gasUsed, output } = await contract.query["makeCommitment"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, domain_name, domain_owner, duration, secret, resolver
        );

        if (result.isOk) {
            const currentTimestamp = output.toHuman()
            console.log("commit hash is : ".yellow, currentTimestamp.Ok);
            return currentTimestamp;
        } else {
            console.error('Failed to read commit hash:', output);
            return null;
        }
    }

    async function commit(commit_hash) {
        await contract.tx
            .commit({ storageDepositLimit, gasLimit }, commit_hash)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function register_domain(domain_name, domain_owner, duration, commit_hash, resolver) {
        await contract.tx
            .register({ value: 7253678335870116692n, storageDepositLimit, gasLimit }, domain_name, domain_owner, duration, commit_hash, resolver)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function set_content_hash(domain_name, ipfsUri) {
        await contract.tx
            .setContentHash({ storageDepositLimit, gasLimit }, domain_name, ipfsUri)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }
    async function mint_nft(domain_name, domain_owner, token_uri) {
        await contract.tx
            .mintNft({ storageDepositLimit, gasLimit }, domain_name, domain_owner, token_uri)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    // await read_owner();
    // await read_resolver();
    await read_current_timestamp();
    // await read_min_commit_age();
    // await read_max_commit_age();
    // await read_min_registration_duration();
    // await read_domain_price("akb.vne", 400000);

    // await make_commitment("akz.vne", userKeyring.address, 400000, "0x01cda9526241efc47b98941546f244a0c9971873278214c59966241d2d667397","0xA62638e3931f800b924d5648A0562532f5b26CF3");
    // await commit("0x310e481780a48b48626774fbf067eaf48fc5a4f9d527b1551e6c9a6fc6aa2314");
    await register_domain("akz.vne", userKeyring.address, 400000, "0x01cda9526241efc47b98941546f244a0c9971873278214c59966241d2d667397", "0xA62638e3931f800b924d5648A0562532f5b26CF3"); 
    // await mint_nft("akz.vne",userKeyring.address,"mint" )
}

main()

//sub domain