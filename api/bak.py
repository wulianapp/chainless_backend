import csv
from web3 import Web3
from web3.exceptions import TimeExhausted
from itertools import cycle

# 初始化Web3连接
web3 = Web3(Web3.HTTPProvider('https://saturn-rpc.swanchain.io'))  # 替换为你的以太坊节点地址

# 替换为你的发送方地址和私钥
sender_address = '<YOUR_WALLET_ADDRESS>'
private_key = '<YOUR_PRIVATE_KEY>'
transfer_amount = web3.to_wei(0.00000000000000001, 'ether')
print(web3.eth.chain_id)


def calculate_fees(gas_used, gas_price):
    gas_fee = gas_used * gas_price
    transfer_fee = transfer_amount + gas_fee
    base_fee = 0
    return gas_fee, transfer_fee, base_fee


def transfer_eth(address):
    try:
        # 获取实时的gas价格
        gas_price = web3.eth.gas_price
        # print(f"Current gas price: {gas_price} wei")
        # 估算gas limit
        gas_limit = web3.eth.estimate_gas({
            'to': address,
            'value': transfer_amount,
        })
        # print(f"Estimated gas limit: {gas_limit}")
        # 获取发送方的nonce
        # nonce = web3.eth.get_transaction_count(sender_address)
        sender_address_checksum = web3.to_checksum_address(sender_address)
        nonce = web3.eth.get_transaction_count(sender_address_checksum)
        # print(nonce)
        bl = web3.eth.get_balance(sender_address_checksum)
        print(f"current balance: {web3.from_wei(bl,'ether')} ETH, nonce:{nonce}")
        # 构建交易
        txn = {
            'to': address,
            'value': transfer_amount,
            'gas': gas_limit,
            'gasPrice': gas_price,
            'nonce': nonce,
            'chainId': web3.eth.chain_id
        }
        # 使用发送方私钥对交易进行签名
        signed_txn = web3.eth.account.sign_transaction(txn, private_key)
        # 发送已签名的交易
        txn_hash = web3.eth.send_raw_transaction(signed_txn.rawTransaction)
        txn_receipt = web3.eth.wait_for_transaction_receipt(txn_hash)
        gas_used = txn_receipt['gasUsed']

        gas_fee, transfer_fee, base_fee = calculate_fees(gas_used, gas_price)
        print(
            f'Transferred {web3.from_wei(transfer_amount, "ether")} ETH to {address}. Txn Hash: {txn_hash.hex()}, '
            f'Transfer fee: {web3.from_wei(transfer_fee, "ether")} ETH')
        # print("\n")
        # print(f"Gas fee: {gas_fee} wei")
        # print(f"Transfer fee: {web3.from_wei(transfer_fee, 'ether')} ETH")
        # print(f"Base fee: {base_fee} wei")
        # print(f"Total cost:{web3.from_wei(transfer_amount + gas_fee + transfer_fee + base_fee, 'ether')} ETH")
    except TimeExhausted as e:
        print(f"RPC call timed out for address {address}: {e}")
    except Exception as e:
        print(f"Error transferring ETH to {address}: {e}")


def main():
    try:
        # 使用无限循环的迭代器来重复读取CSV文件
        # with open('wallets.csv', 'r', encoding='utf-8') as file:
        with open('etherem_ethereum-address_000000000009.csv', 'r', encoding='utf-8') as file:

            reader = csv.reader(file)
            next(reader)  # 跳过第一行
            address_cycle = cycle(reader)
            for row in address_cycle:

                # address, balance = row
                address = str(row[0]).split(";")[0]
                balance = str(row[0]).split(";")[1]
                address = address.strip()
                address = web3.to_checksum_address(address)
                print("\n")
                # print(address)

                if float(balance) > 0:
                    # 调用转账函数
                    transfer_eth(address)
                else:
                    print(f"Balance for address {address} is 0, skipping transfer.")
    except Exception as e:
        print(f"Error reading CSV file: {e}")


if __name__ == "__main__":
    main()
