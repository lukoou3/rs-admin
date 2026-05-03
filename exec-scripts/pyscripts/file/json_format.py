import sys
import json
import argparse

def json_format(args):
    pretty = args.pretty
    indent = args.indent
    file = args.file
    array_ele_compact = args.array_ele_compact

    if file:
        with open(file, encoding='utf-8') as fp:
            text = fp.read()
    else:
        text = sys.stdin.read()

    obj = json.loads(text)

    if array_ele_compact:
        for ele in obj:
            print(json.dumps(ele, ensure_ascii=False))
    else:
        if pretty:
            print(json.dumps(obj, ensure_ascii=False, indent=indent))
        else:
            print(json.dumps(obj, ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="json_format")
    parser.add_argument("--pretty", type=int, default=1, help="format pretty")
    parser.add_argument("--indent", type=int, default=4, help="indent")
    parser.add_argument("--file", type=str, default="", help="file")
    parser.add_argument("--array_ele_compact", type=int, default=0, help="array format compact")
    args = parser.parse_args()
    json_format(args)
