import argparse
from Crypto.Cipher import AES
from Crypto.Util.Padding import pad, unpad
import base64
import sys

# pip install pycryptodome
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
    iv = args.iv
    data = args.data
    decrypt = args.decrypt
    cipher = AES.new(
        key=key.encode('utf-8'),
        mode=AES.MODE_CBC,
        iv=iv.encode('utf-8')
    )
    if decrypt:
        sys.stdout.write(aes_decrypt())
    else:
        sys.stdout.write(aes_encrypt())


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="json_format")
    parser.add_argument("--key", type=str, default="fd6b639dbcff0c2a", help="key")
    parser.add_argument("--iv", type=str, default="77b07a672d57d64c", help="iv")
    parser.add_argument("--data", type=str, default="", help="data")
    parser.add_argument("--decrypt", type=int, default=0, help="decrypt")
    args = parser.parse_args()
    aes_encrypt_decrypt(args)