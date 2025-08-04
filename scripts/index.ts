import { keyStores, Near } from "near-api-js"

const near = new Near({
    nodeUrl: "https://rpc.mainnet.near.org",
    networkId: "default",
    keyStore: new keyStores.UnencryptedFileSystemKeyStore("./keys.json")
});


