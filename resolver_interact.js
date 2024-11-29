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

    const contractAddress = '0xf1F0611F204a89c5eBa04232736d137E18b0AE74';
    const contract = new ContractPromise(api, json, contractAddress);

    console.log('Available contract methods:'.cyan, Object.keys(contract.tx));

    const keyring = new Keyring({ type: 'ethereum' });
    const userKeyring = keyring.addFromUri('0xd615610ab9435f7be2c82e87d6f62c06c75aeb63a2ba6a5cb55260a38ae4cfa4');

    const name = 'prem.vne';
    const duration = 343;
    const price = 10;


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

    async function read_manager() {     
        const { result, gasUsed, output } = await contract.query["readManager"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const Manager = output.toHuman()
            console.log("Manager address is : ".yellow, Manager.Ok);
            return Manager;
        } else {
            console.error('Failed to read Manager address:', output);
            return null;
        }
    }

    async function read_domain_manager(domain_name) {
        const { result, gasUsed, output } = await contract.query["readDomainManager"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, domain_name
        );

        if (result.isOk) {
            const Manager = output.toHuman()
            console.log(" Domain Manager address is : ".yellow, Manager.Ok);
            return Manager;
        } else {
            console.error('Failed to read Domain Manager address:', output);
            return null;
        }
    }

    async function read_grace_period() {
        const { result, gasUsed, output } = await contract.query["readGracePeriod"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null },
        );

        if (result.isOk) {
            const gracePeriod = output.toHuman()
            console.log("grace period is : ".yellow, gracePeriod.Ok);
            return gracePeriod;
        } else {
            console.error('Failed to read grace period :', output);
            return null;
        }
    }
    async function read_domain_record(domain_name) {
        const { result, gasUsed, output } = await contract.query["readRecord"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, domain_name
        );

        if (result.isOk) {
            const readRecord = output.toHuman()
            console.log("domain record is : ".yellow, readRecord.Ok);
            return readRecord;
        } else {
            console.error('Failed to read domain record :', output);
            return null;
        }
    }
    async function read_content_hash(domain_name) {
        const { result, gasUsed, output } = await contract.query["readContentHash"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, domain_name
        );

        if (result.isOk) {
            const ContentHash = output.toHuman()
            console.log("content hash is : ".yellow, ContentHash.Ok);
            return ContentHash;
        } else {
            console.error('Failed to read content hash :', output);
            return null;
        }
    }

    async function read_content_text(domain_name
    ) {
        const { result, gasUsed, output } = await contract.query["readDomainContentText"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, domain_name
        );

        if (result.isOk) {
            const ContentText = output.toHuman()
            console.log("content text is : ".yellow, ContentText.Ok);
            return ContentText;
        } else {
            console.error('Failed to read content text :', output);
            return null;
        }
    }

    async function read_subdomain_content_text(sub_domain_name
    ) {
        const { result, gasUsed, output } = await contract.query["readSubdomainContentText"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, sub_domain_name
        );

        if (result.isOk) {
            const ContentText = output.toHuman()
            console.log("content text is : ".yellow, ContentText.Ok);
            return ContentText;
        } else {
            console.error('Failed to read content text :', output);
            return null;
        }
    }


    async function read_domain_owner(domain_name) {
        const { result, gasUsed, output } = await contract.query["readDomainOwner"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, domain_name
        );

        if (result.isOk) {
            const DomainOwner = output.toHuman()
            console.log("DomainOwner is : ".yellow, DomainOwner.Ok);
            return DomainOwner;
        } else {
            console.error('Failed to read DomainOwner :', output);
            return null;
        }
    }

    async function read_domain_expiry_time(domain_name) {
        const { result, gasUsed, output } = await contract.query["readDomainExpiryTime"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, domain_name
        );

        if (result.isOk) {
            const DomainExpiryTime = output.toHuman()
            console.log("DomainExpiryTime is : ".yellow, DomainExpiryTime.Ok);
            return DomainExpiryTime;
        } else {
            console.error('Failed to read DomainExpiryTime :', output);
            return null;
        }
    }

    async function check_domain_availablility(domain_name) {
        const { result, gasUsed, output } = await contract.query["checkDomainAvailablility"](
            userKeyring.address,
            { gasLimit: gasLimit, storageDepositLimit: null }, domain_name
        );

        if (result.isOk) {
            const DomainAvailablility = output.toHuman()
            console.log("DomainAvailablility is : ".yellow, DomainAvailablility.Ok);
            return DomainAvailablility;
        } else {
            console.error('Failed to check DomainAvailablility :', output);
            return null;
        }
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

    async function set_domain_content_text(domain_name, content_key, content_text) {
        await contract.tx
            .setDomainContentText({ storageDepositLimit, gasLimit }, domain_name, content_key, content_text)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }
    async function set_domain_content_text(domain_name, content_key, content_text) {
        await contract.tx
            .setDomainContentText({ storageDepositLimit, gasLimit }, domain_name, content_key, content_text)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function set_domain_manager(domain_name) {
        await contract.tx
            .setDomainManager({ storageDepositLimit, gasLimit }, domain_name)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function unregister_domain(domain_name) {
        await contract.tx
            .unregisterDomain({ storageDepositLimit, gasLimit }, domain_name)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function change_manager(manager) {
        await contract.tx
            .changeManager({ storageDepositLimit, gasLimit }, manager)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function register_subdomain(parent_domain, subdomain) {
        await contract.tx
            .registerSubdomain({ storageDepositLimit, gasLimit }, parent_domain,subdomain)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }

    async function set_grace_period(grace_period) {
        await contract.tx
            .setGracePeriod({ storageDepositLimit, gasLimit }, grace_period)
            .signAndSend(userKeyring, result => {
                if (result.status.isInBlock) {
                    console.log(`initialised in block : ${result.status.asInBlock}`.cyan);
                } else if (result.status.isFinalized) {
                    console.log(`finalized in block : ${result.status.asFinalized}`.cyan);
                }
            });
    }


    // await unregister_domain("akh.vne");
    // await set_content_hash("hello_Boi12343458989.vne","https://github.com/arpitantier42/secure_transaction_system");
    // await set_domain_content_text("arpitsss.vne","social","https://github.com/arpitantier42/secure");
    // await change_manager("0x1bacaecc83ed515b77a8d39f24e46e05c8bbc920");
    // await register_subdomain("akz.vne", "arpit.akz.vne");
    // await set_grace_period(100);

    // await read_owner();
    // await read_manager();
    // await read_grace_period();
    // await read_domain_record("google.vne");
    await read_content_hash("google.vne");

    // await read_content_text("hello_Boi12343458989.vne");
    // await read_domain_owner("arpitssk.vne")
    // await read_domain_expiry_time("akz.vne");
    // await check_domain_availablility("hello_Boi12343458989.vne");
    
}

main()

