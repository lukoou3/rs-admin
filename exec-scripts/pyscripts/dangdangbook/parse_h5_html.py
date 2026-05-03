from bs4 import BeautifulSoup
from bs4.element import Tag
import re
import os
import itertools
import json
import requests
import Levenshtein
import hashlib
import argparse

headers = {
    'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
    'Connection': 'keep-alive',
    'Pragma': 'no-cache',
    'Cache-Control': 'no-cache',
    'Accept-Encoding': 'gzip,deflate,sdch',
    'Accept-Language': 'zh-CN,zh;q=0.8',
    'User-Agent': 'Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/45.0.2454.101 Safari/537.36'
}

left_re = re.compile(r"left:\s*(\d+)px")
bottom_re = re.compile(r"bottom:\s*(-?\d+)px")
top_re = re.compile(r"top:\s*(-?\d+)px")
width_re = re.compile(r"width:\s*(\d+)px")
height_re = re.compile(r"height:\s*(\d+)px")

class Line:
    def __init__(self, content: str, pagenum: int, left: int, left_max: int, bottom: int, paragraph_start: bool = False,
                 img_title: bool = False, menu=None):
        self.content = content
        self.pagenum = pagenum
        self.left = left
        self.left_max = left_max
        self.bottom = bottom
        self.paragraph_start = paragraph_start
        self.img_title = img_title
        self.menu = menu


class Image:
    def __init__(self, src: str, pagenum: int, left: int, top: int, bottom: int, width: int, height: int, menu=None):
        self.src = src
        self.pagenum = pagenum
        self.left = left
        self.top = top
        self.bottom = bottom
        self.width = width
        self.height = height
        self.menu = menu

class Paragraph:
    def __init__(self, content: str, left: int, bottom: int, img_title: bool = False):
        self.content = content
        self.left = left
        self.bottom = bottom
        self.img_title = img_title

def get_left_by_style(style: str) -> int:
    return int(left_re.search(style).group(1))


def get_bottom_by_style(style: str) -> int:
    return int(bottom_re.search(style).group(1))


def get_top_by_style(style: str) -> int:
    return int(top_re.search(style).group(1))


def get_width_by_style(style: str) -> int:
    return int(width_re.search(style).group(1))


def get_height_by_style(style: str) -> int:
    return int(height_re.search(style).group(1))


def get_images(soup: BeautifulSoup, pagenum):
    img_divs = soup.select("div.image")
    images = []

    for p_img_div in img_divs:
        parent_style = p_img_div.get("style")
        for img_div in [e for e in p_img_div.contents if isinstance(e, Tag) and e.name == "div"]:
            div = img_div
            style = div.get("style")
            try:
                width = get_width_by_style(style)
                height = get_height_by_style(style)
                left = get_left_by_style(style)
                top = get_top_by_style(style)
                bottom = get_height_by_style(parent_style) - top - height
                src = img_div.select_one("img").get("src")
                # 注释先不处理
                if "note_epub.png" in src:
                    continue
                image = Image(src, pagenum, left=left, top=top, bottom=bottom, width=width, height=height)
                images.append(image)
            except Exception as e:
                print(e)
                print(style)
                raise e

    return images


class Span:
    def __init__(self, text: str, left: int, bottom: int, cl: str):
        self.text = text
        self.left = left
        self.bottom = bottom
        self.cl = cl


def get_lines(soup: BeautifulSoup, pagenum, pre_left):
    spans = []
    for span in soup.select("div.text span"):
        cls = span.get_attribute_list("class", [])
        cl = ' '.join(cls)
        style = span.get("style")
        left = get_left_by_style(style)
        bottom = get_bottom_by_style(style)
        text = span.get_text(strip=False)
        spans.append(Span(text, left, bottom, cl))

    #line_padding_min = 33
    line_padding_min = args.line_padding_min
    # itertools.groupby 需要先排好序
    spans.sort(key=lambda x: -x.bottom)
    grouped_spans = [[bottom, sorted(list(spans), key=lambda x: x.left)]
                     for bottom, spans in itertools.groupby(spans, key=lambda x: x.bottom)]
    grouped_spans.sort(key=lambda x: -x[0])
    for i, line_spans in enumerate(grouped_spans):
        if i > 0 and grouped_spans[i-1][0] - line_spans[0] < line_padding_min \
                and len(grouped_spans[i-1][1]) <= 10:
            grouped_spans[i-1][0] = line_spans[0]
    grouped_spans = [[bottom, sorted([spans for i, line_spans in line_spans_list for spans in line_spans], key=lambda x: x.left)]
                     for bottom, line_spans_list in itertools.groupby(grouped_spans, key=lambda x: x[0])]

    lines = []
    pre_bottom = None
    #paragraph_padding_max = 100 # 100, 66
    paragraph_padding_max = args.paragraph_padding_max # 100, 66
    # pre_left = None
    for bottom, spans in grouped_spans:
        content = ''.join([x.text for x in spans])
        if content.startswith("#"):
            content = '`#`' + content[1:]
        left = spans[0].left  # min([x.left for x in spans])
        left_max = spans[-1].left
        img_title = True if left >= 220 else False  # fs-b92d6262-1f5, fs-b92d6262-1f6
        paragraph_start = False

        if (pre_bottom is not None and pre_bottom - bottom > paragraph_padding_max) or \
                (pre_left is None and left >= 120) or (pre_left is not None and left - pre_left >= 80):
            paragraph_start = True
        if content.startswith("● "):
            content = '*' + content[1:]
            paragraph_start = True
        if content.startswith("❑"):
            content = '* ' + content[1:]
            paragraph_start = True
        pre_bottom = bottom
        pre_left = left

        # if "[root@laohan_httpd_server" in content:
        #     print(content)

        line = Line(content, pagenum, left, left_max, bottom, paragraph_start, img_title)
        lines.append(line)

    return lines


def get_paragraphs(lines):
    def get_paragraph(lines):
        content = ''.join([x.content for x in lines])
        left = lines[0].left
        bottom = lines[-1].bottom
        img_title = lines[0].img_title
        return Paragraph(content, left, bottom, img_title)

    paragraphs = []
    p_lines = []

    for line in lines:
        if line.paragraph_start and p_lines:
            paragraphs.append(get_paragraph(p_lines))
            p_lines.clear()
        p_lines.append(line)

    if p_lines:
        paragraphs.append(get_paragraph(p_lines))

    return paragraphs

def make_md5(raw):
    """计算出一个字符串的MD5值"""
    md5 = hashlib.md5()
    md5.update(raw.encode())
    return md5.hexdigest()

def download_img(url, img_path):
    """下载图片并返回文件名"""
    suffix = ".png"
    if url.rfind(".") > 0:
        suffix_ = url[url.rfind("."):]
        if suffix_ in [".jpg", ".png", ".gif"]:
            suffix = suffix_
    name = make_md5(url) + suffix

    path = os.path.join(img_path, name)
    if not os.path.exists(path):
        try:
            response = requests.get(url, timeout=5, headers= headers)
            with open(path, "wb") as fp:
                fp.write(response.content)
        except Exception as e:
            print((url + "下载失败", e))
            return url

    return "assets/" + name

def exec_convert(book_id):
    import pymongo
    client = pymongo.MongoClient(host='localhost', port=27017)
    db = client.dangdang
    contents = db.dd_book_h5_contents.find_one({"_id": book_id})['contents']
    menu_list = [{'chapterID': content['chapterID'],'chapterNum': content['chapterNum'],'pageIndex': content['htmlPage'],'label': content['label'],'level': content['level']} for content in contents]
    menu_list.sort(key=lambda x: (x['chapterID'], -x['chapterNum']))
    menu_map = {menu['chapterID']: [menu] for menu in menu_list}
    page_menus = {}
    for menu in menu_list:
        page = (menu['chapterID'], menu['pageIndex'])
        menus = page_menus.get(page, [])
        menus.append(menu)
        page_menus[page] = menus

    chapters = list(db.dd_book_h5_chapters.find({"book_id": book_id}))
    chapters.sort(key=lambda x: x['page']['pageNum'])

    image_lines = []
    pre_left = None
    has_menus = set()
    for item in chapters:
        pagenum = item['page']['pageNum']
        chapterID = int(item['_id'].split('_')[1])
        pageIndex = int(item['_id'].split('_')[2])
        menus = page_menus.get((chapterID, pageIndex), menu_map.get(chapterID, []))

        soup = BeautifulSoup(json.loads(item['chapterInfo'])['snippet'], "lxml")
        images = get_images(soup, pagenum)
        lines = get_lines(soup, pagenum, pre_left)

        if len(lines) > 0 and menus:
            for menu in menus:
                if (menu['chapterID'], menu['pageIndex'], menu['label'])  in has_menus:
                    continue
                if menu['pageIndex'] == 0:
                    lines[0].menu = menu
                else:
                    line = max(lines, key=lambda line: Levenshtein.jaro(line.content, menu['label']))
                    line.menu = menu
                has_menus.add((menu['chapterID'], menu['pageIndex'], menu['label']))
        elif len(images) > 0 and menus:
            for menu in menus:
                if (menu['chapterID'], menu['pageIndex'], menu['label'])  in has_menus:
                    continue
                if menu['pageIndex'] == 0:
                    images[0].menu = menu
                    has_menus.add((menu['chapterID'], menu['pageIndex'], menu['label']))

        this_image_lines = sorted(images + lines, key=lambda x: (x.pagenum, -x.bottom))
        if this_image_lines:
            pre_left = this_image_lines[-1].left
            if this_image_lines[-1].bottom > 200:
                pre_left = None
        else:
            pre_left = None

        image_lines.extend(this_image_lines)

    img_path = r"D:\MarkdownFiles\{project_name}\{dir_name}\assets".format(project_name=project_name, dir_name=dir_name)
    if not os.path.exists(img_path):
        os.makedirs(img_path)

    #original_line_mode = 1 # 0, 1
    original_line_mode = args.original_line_mode # 0, 1
    num = 1 if not original_line_mode else 11 #11
    line_cnt = 0
    path = os.path.join(base_path, project_name, dir_name, "{book_name}-{num}.md".format(book_name=book_name,num=str(num).rjust(2, "0")))
    print("gene:" + path, flush=True)
    file = open(path, "w", encoding="utf-8")

    #page_line_cnt = 2000
    page_line_cnt = args.page_line_cnt
    for i, data in enumerate(image_lines):
        line_cnt += 1
        if isinstance(data, Line):
            # if "[root@laohan_httpd_server" in data.content:
            #     print(data.content)
            if i == 0 or not isinstance(image_lines[i - 1], Line) or image_lines[i - 1].menu or (0 and image_lines[
                i - 1].left_max < 850 and image_lines[i - 1].left < 120):
                data.paragraph_start = True
            if data.menu:
                level = data.menu['level']
                label = data.menu['label']
                if level in [0] and line_cnt >= page_line_cnt:
                    file.close()
                    line_cnt = 0
                    num += 1
                    path = os.path.join(base_path, project_name, dir_name, "{book_name}-{num}.md".format(book_name=book_name,num=str(num).rjust(2, "0")))
                    print("gene:" + path, flush=True)
                    file = open(path, "w", encoding="utf-8")
                print("\n", file=file)
                print("#" * (level + 1) + " " + label, file=file)
            elif data.paragraph_start:
                print("\n", file=file)
                if data.left >= 200 and "表" in data.content:
                    print("""\n<p align="center">""" + data.content + """</p>\n""", end="", file=file)
                else:
                    print(data.content, end="", file=file)
            else:
                if original_line_mode:
                    print("\n", end="", file=file)
                print(data.content, end="", file=file)
        else:
            if data.menu:
                level = data.menu['level']
                label = data.menu['label']
                if level in [0] and line_cnt >= page_line_cnt:
                    file.close()
                    line_cnt = 0
                    num += 1
                    path = os.path.join(base_path, project_name, dir_name, "{book_name}-{num}.md".format(book_name=book_name,num=str(num).rjust(2, "0")))
                    print("gene:" + path, flush=True)
                    file = open(path, "w", encoding="utf-8")
                print("\n", file=file)
                print("#" * (level + 1) + " " + label, file=file)
            img_url = download_img(data.src, img_path)
            print(file=file)
            print("![]({0})".format(img_url), file=file)
    file.close()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(usage="parse_h5_html.", description="parse_h5_html.")
    parser.add_argument("--book_id", type=str, required=True, help="book_id")
    parser.add_argument("--base_path", type=str, default=r"D:\MarkdownFiles", help="base_path")
    parser.add_argument("--project_name", type=str, required=True, help="project_name")
    parser.add_argument("--dir_name", type=str, required=True, help="书名即目录名")
    parser.add_argument("--paragraph_padding_max", type=int, default=100, help="段落最大间隔")
    parser.add_argument("--line_padding_min", type=int, default=33, help="行最小间隔")
    parser.add_argument("--original_line_mode", type=int, default=0, help="是否输出原始行")
    parser.add_argument("--page_line_cnt", type=int, default=2000, help="多少行输出一个文件")
    args = parser.parse_args()
    print(json.dumps(args.__dict__, ensure_ascii=False, indent=4))
    book_id = args.book_id
    project_name = args.project_name
    dir_name = args.dir_name
    #book_id = "1901238548"
    #project_name = "pythonEpubBook"
    #dir_name = "Python3反爬虫原理与绕过实战"
    base_path = args.base_path
    book_name = "book"
    exec_convert(book_id)