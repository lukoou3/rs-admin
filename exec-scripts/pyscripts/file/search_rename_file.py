import os
import sys
import re
import json
import argparse

def search_rename_file(root_path, file_name_match="", file_name_match_re=False,
                       replace_old="", replace_new="", replace_match_re=False, replace=False):
    path = os.path.abspath(root_path)
    if not os.path.exists(path):
        sys.stderr.write("{}路径不存在\n".format(path))
    if file_name_match_re:
        file_name_re = re.compile(file_name_match)
    if replace_match_re:
        replace_re = re.compile(replace_old)
    for parent, dirs, names in os.walk(path):
        for name in names:
            if file_name_match_re:
                if not file_name_re.search(name):
                    continue
            else:
                if file_name_match not in name:
                    continue
            if replace_match_re:
                new_name = replace_re.sub(replace_new, name)
            else:
                new_name = name.replace(replace_old, replace_new)
            src_file = os.path.join(parent, name)
            dest_file = os.path.join(parent, new_name)
            if src_file == dest_file:
                continue
            print("{} => {}".format(src_file, dest_file))
            if replace:
                print("do {} => {}".format(src_file, dest_file))
                os.replace(src_file, dest_file)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="search_rename_file")
    parser.add_argument("--root_path", type=str, required=True, help="root_path")
    parser.add_argument("--file_name_match", type=str, default="", help="file_name_match")
    parser.add_argument("--file_name_match_re", type=int, default=0, help="file_name_match_re")
    parser.add_argument("--replace_old", type=str, default="", help="replace_old")
    parser.add_argument("--replace_new", type=str, default="", help="replace_new")
    parser.add_argument("--replace_match_re", type=int, default=0, help="replace_match_re")
    parser.add_argument("--replace", type=int, default=0, help="replace")
    args = parser.parse_args()
    print(json.dumps(args.__dict__, ensure_ascii=False, indent=4))
    #search_rename_file(r"F:\BaiduNetdiskDownload\clickhouse\ClickHouse大数据分析技术与实战", file_name_match="zip", file_name_match_re=1, replace_old="【瑞客论坛 www.ruike1.com】", replace_new="", replace=1)
    search_rename_file(**args.__dict__)