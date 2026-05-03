import argparse, random, json, sys
import time


def exec():
    parser = argparse.ArgumentParser(usage="test usage tip.", description="test help info.")
    parser.add_argument("--id", type=int, default=1, help="id")
    #parser.add_argument("--name", type=str, required=True, help='name')
    parser.add_argument("--name", type=str, default="雄", help='name')
    args = parser.parse_args()
    data = {
        "id": args.id,
        "name": args.name
    }
    #time.sleep(2)
    for i in range(20):
        time.sleep(1)
        print("sysout:" + json.dumps(data, ensure_ascii=False), flush= True)
        sys.stderr.write("syserr:" + json.dumps(data, ensure_ascii=False) + "\n")
        sys.stderr.flush()

if __name__ == '__main__':
    exec()
