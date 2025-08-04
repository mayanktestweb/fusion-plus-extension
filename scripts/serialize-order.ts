import * as borsh from "borsh";

// Borsh schema for MakerOrder
const makerOrderSchema = {
  struct: {
    root_hash: 'string',
    token: 'string',
    total_amount: 'u128',
    parts: 'u16',
    filled_amount: 'u128',
    withdrawn_amount: 'u128',
    maker: 'string',
    expiration: 'u64'
  }
};

// Create fake maker order
const makerOrder = {
  root_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", // SHA256 of empty string
  token: "mayank-token-1.testnet",
  total_amount: "1000000000000000000000000", // 1 token (assuming 24 decimals)
  parts: 1,
  filled_amount: "0",
  withdrawn_amount: "0",
  maker: "mayank-hello-world.testnet",
  expiration: (Date.now() + 86400000) * 1000000 // 24 hours from now in nanoseconds
};

function serializeOrder() {
  try {
    console.log('Original maker order:');
    console.log(JSON.stringify(makerOrder, null, 2));
    console.log('\n');
    
    // Serialize the maker order
    const serializedOrder = borsh.serialize(makerOrderSchema, makerOrder);
    const hexMsg = Buffer.from(serializedOrder).toString('hex');
    
    console.log('Serialized order (bytes):', serializedOrder);
    console.log('Serialized order (hex):', hexMsg);
    console.log('Hex length:', hexMsg.length);
    
    return hexMsg;
  } catch (error) {
    console.error('Error serializing order:', error);
    return null;
  }
}

// Run the serialization
const hexString = serializeOrder();

if (hexString) {
  console.log('\n✅ Successfully generated hex string!');
  console.log('Use this hex string as the msg parameter in ft_transfer_call');
} else {
  console.log('\n❌ Failed to generate hex string');
}
