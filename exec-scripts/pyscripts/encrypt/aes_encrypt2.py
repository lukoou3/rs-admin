import argparse
from Crypto.Cipher import AES
from Crypto.Util.Padding import pad, unpad
import base64
import sys

def aes_encrypt_decrypt(args):
    def aes_encrypt():
        raw = pad(data.encode('utf-8'), 16)
        bs = cipher.encrypt(raw)
        return base64.b64encode(bs).decode('utf-8')
    
    def aes_decrypt():
        bs = base64.b64decode(data)
        decrypted_text = cipher.decrypt(bs)
        return unpad(decrypted_text, 16).decode('utf-8')
    
    key = args.key
    data = args.data
    decrypt = args.decrypt
    
    # 使用 ECB 模式，无需 IV
    cipher = AES.new(
        key=key.encode('utf-8'),
        mode=AES.MODE_ECB
    )
    
    if decrypt:
        sys.stdout.write(aes_decrypt())
    else:
        sys.stdout.write(aes_encrypt())

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="AES encryption compatible with MySQL AES_ENCRYPT")
    parser.add_argument("--key", type=str, default="fd6b639dbcff0c2a", help="key")
    parser.add_argument("--data", type=str, default="", help="data")
    parser.add_argument("--decrypt", type=int, default=0, help="decrypt (1 for decrypt, 0 for encrypt)")
    args = parser.parse_args()
    aes_encrypt_decrypt(args)