import time
import pathlib
from dotenv import load_dotenv
import os


from stellar_sdk import Network, Keypair, TransactionBuilder
from stellar_sdk import xdr as stellar_xdr
from stellar_sdk.soroban import SorobanServer
from stellar_sdk.soroban.soroban_rpc import GetTransactionStatus

load_dotenv()

# TODO: You need to replace the following parameters according to the actual situation
secret = os.getenv("SECRET")
rpc_server_url = "https://rpc-futurenet.stellar.org:443"
network_passphrase = Network.FUTURENET_NETWORK_PASSPHRASE
contract_file_path = pathlib.Path().absolute().parent.resolve().joinpath("target/wasm32-unknown-unknown/release/disburse_contract.wasm").__str__()

kp = Keypair.from_secret(secret)
soroban_server = SorobanServer(rpc_server_url)

print("installing contract...")
source = soroban_server.load_account(kp.public_key)


# with open(contract_file_path, "rb") as f:
#     contract_bin = f.read()

tx = (
    TransactionBuilder(source, network_passphrase)
    .set_timeout(300)
    .append_install_contract_code_op(
        contract=contract_file_path,  # the path to the contract, or binary data
    )
    .build()
)

tx = soroban_server.prepare_transaction(tx)
tx.sign(kp)
send_transaction_data = soroban_server.send_transaction(tx)
print(f"sent transaction: {send_transaction_data}")

while True:
    print("waiting for transaction to be confirmed...")
    get_transaction_data = soroban_server.get_transaction(send_transaction_data.hash)
    if get_transaction_data.status != GetTransactionStatus.NOT_FOUND:
        break
    time.sleep(3)

print(f"transaction: {get_transaction_data}")

wasm_id = None
if get_transaction_data.status == GetTransactionStatus.SUCCESS:
    assert get_transaction_data.result_meta_xdr is not None
    transaction_meta = stellar_xdr.TransactionMeta.from_xdr(
        get_transaction_data.result_meta_xdr
    )
    result = transaction_meta.v3.tx_result.result.results[0].tr.invoke_host_function_result.success[0]  # type: ignore
    wasm_id = result.bytes.sc_bytes.hex()  # type: ignore
    print(f"wasm id: {wasm_id}")

assert wasm_id, "wasm id should not be empty"

print("creating contract...")

source = soroban_server.load_account(
    kp.public_key
)  # refresh source account, because the current SDK will increment the sequence number by one after building a transaction

tx = (
    TransactionBuilder(source, network_passphrase)
    .set_timeout(300)
    .append_create_contract_op(
        wasm_id=wasm_id,
    )
    .build()
)

tx = soroban_server.prepare_transaction(tx)
tx.sign(kp)

send_transaction_data = soroban_server.send_transaction(tx)
print(f"sent transaction: {send_transaction_data}")

while True:
    print("waiting for transaction to be confirmed...")
    get_transaction_data = soroban_server.get_transaction(send_transaction_data.hash)
    if get_transaction_data.status != GetTransactionStatus.NOT_FOUND:
        break
    time.sleep(3)

print(f"transaction: {get_transaction_data}")

if get_transaction_data.status == GetTransactionStatus.SUCCESS:
    assert get_transaction_data.result_meta_xdr is not None
    transaction_meta = stellar_xdr.TransactionMeta.from_xdr(
        get_transaction_data.result_meta_xdr
    )
    result = transaction_meta.v3.tx_result.result.results[0].tr.invoke_host_function_result.success[0]  # type: ignore
    contract_id = result.bytes.sc_bytes.hex()  # type: ignore
    print(f"contract id: {contract_id}")


