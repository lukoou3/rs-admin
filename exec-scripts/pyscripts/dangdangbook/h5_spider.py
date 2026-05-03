import pymongo
import requests
import json
import sys
import time

from exec_pyscript_utils.utils import retry

class BookSpider:
    """
    把当当云阅读h5端的电子书保存到mongo
    构造函数需要传入id和name，例如：BookSpider(id="1901283697", name="AI源码解读:推荐系统案例(Python版)")
    需要登录，需要修改的信息：token，headers['Cookie']
    """

    def __init__(self, id: str, name: str, token: str, cookie: str):
        self.id = str(id)
        self.name = name
        self.client = None
        self.db = None
        self.token = token
        self.contentsPage = "https://e.dangdang.com/media/api2.go?action=getOnlineContents&mediaId={}".format(self.id)
        self.baseUrl = "&appId=1&consumeType=1&platform=3&sign=html5&deviceType=html5&deviceVersion=5.0.0&channelId=70000&deviceSerialNo=html5&clientVersionNo=6.8.3&platformSource=DDDS-P&fromPlatform=107&permanentId=20230304152209810431912882711789496"
        self.contentsUrl = self.contentsPage + self.baseUrl
        self.chapterPage = "https://e.dangdang.com/media/api.go?action=getOnlineChapterInfo&chapterBlockSize=50&epubID={}".format(self.id)
        self.chapterUrl = self.chapterPage + self.baseUrl
        self.style = 3
        self.wordSize = 3
        self.headers = {
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
            'Connection': 'keep-alive',
            'Pragma': 'no-cache',
            'Cache-Control': 'no-cache',
            'Accept-Encoding': 'gzip,deflate,sdch',
            'Accept-Language': 'zh-CN,zh;q=0.8',
            'Cookie': cookie,
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36'
        }

    def getBookBaseInfo(self):
        item = {"_id": self.id, "name": self.name}
        tb = self.db.dd_book_h5
        tb.update_one({"_id": item["_id"]}, {"$set": item}, upsert=True)

    def getContentsInfo(self):
        tb = self.db.dd_book_h5_contents
        if tb.find_one({"_id": self.id}):
            return

        contents = []
        pageCount = 50
        pageNo = 1
        while True:
            data = {
                'token': self.token,
                'style': self.style,
                'pageCount': 50,
                'pageNo': pageNo,
            }
            text = retry("getContentsInfo", lambda : requests.get(self.contentsUrl, data, headers=self.headers, timeout=30).text, 5)
            contentsInfo = json.loads(text)
            assert contentsInfo['status']['code'] == 0
            contents.extend(contentsInfo['data']['contents'])
            total = contentsInfo['data']['total']
            if pageCount * pageNo >= total:
                assert len(contents) == total
                break
            pageNo = pageNo + 1

        item = {'_id': self.id, 'contents': contents}
        tb.update_one({"_id": item["_id"]}, {"$set": item}, upsert=True)

    def getChapterInfo(self):
        def requestChapterInfo(data):
            text = retry("getChapterInfo",
                         lambda: requests.get(self.chapterUrl, data, headers=self.headers, timeout=30).text, 5,
                         wait_sleep=3)
            chapterInfo = json.loads(text)
            assert chapterInfo['status']['code'] == 0
            return chapterInfo
        def getOneChapterInfo(chapterId, pageIndex):
            _id = self.id + "_" + str(chapterId) + "_" + str(pageIndex)
            tb = self.db.dd_book_h5_chapters
            chapterInfo = tb.find_one({"_id": _id})
            if chapterInfo:
                return chapterInfo

            time.sleep(1.5)

            data = {
                'token': self.token,
                'style': self.style,
                'wordSize': self.wordSize,
                'autoBuy': 0,
                'chapterId': chapterId,
                'pageIndex': pageIndex,
            }
            print({'chapterId': chapterId, 'pageIndex': pageIndex})

            chapterInfo = retry("getChapterInfo", lambda: requestChapterInfo(data), 5, wait_sleep=2)
            assert chapterInfo['status']['code'] == 0
            chapterInfo = chapterInfo['data']
            chapterInfo['_id'] = _id
            chapterInfo['book_id'] = self.id
            item = chapterInfo

            tb.update_one({"_id": item["_id"]}, {"$set": item}, upsert=True)

            return chapterInfo

        content = self.db.dd_book_h5_contents.find_one({"_id": self.id})['contents'][0]
        firstChapterID = content['chapterID']
        print("firstChapterID:", firstChapterID)
        chapterId = firstChapterID
        chapterId = 0 # 手动设置目录之前的封面
        pageIndex = 0
        count = 0
        while True:
            count = count + 1
            chapterInfo = getOneChapterInfo(chapterId, pageIndex)
            pageCount = chapterInfo['page']['pageCount']
            nextChapterId = chapterInfo['chapter'].get('nextChapterId')
            if pageIndex == pageCount - 1:
                if nextChapterId is None:
                    break
                chapterId = nextChapterId
                pageIndex = 0
            else:
                pageIndex = pageIndex + 1
            sys.stdout.flush()
        print("end")


    def init_db(self):
        if self.client is None:
            self.client = pymongo.MongoClient(host='localhost', port=27017)
            self.db = self.client.dangdang

    def close_db(self):
        if self.client is not None:
            self.client.close()
            self.client = None

    def spider(self):
        self.init_db()

        self.getBookBaseInfo()
        self.getContentsInfo()
        self.getChapterInfo()

        self.close_db()


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(usage="h5_spider.", description="h5_spider.")
    parser.add_argument("--id", type=str, default="1901238548", help="id")
    parser.add_argument("--name", type=str, default="Python3反爬虫原理与绕过实战", help='name')
    parser.add_argument("--token", type=str, default="m_9f2522294a7a7bad93665d77c67b6cb079a054256a1c0ff4863d99268d6d313c", help='token')
    parser.add_argument("--cookie", type=str, default="__permanent_id=20210311233437445167977491851529287; permanent_key=202108260738196966314791881fe77a; MDD_channelId=70000; MDD_fromPlatform=307; dest_area=country_id%3D9000%26province_id%3D111%26city_id%20%3D0%26district_id%3D0%26town_id%3D0; MDD_permanent_id=20230304152209810431912882711789496; MDD_province_str=%E5%8C%97%E4%BA%AC; MDD_province_id=111; MDD_city_str=%E5%8C%97%E4%BA%AC%E5%B8%82; MDD_city_id=1; MDD_area_str=%E4%B8%9C%E5%9F%8E%E5%8C%BA; MDD_area_id=1110101; MDD_username=18638489474; MDD_token=m_9f2522294a7a7bad93665d77c67b6cb079a054256a1c0ff4863d99268d6d313c; MDD_ucid=m_9f2522294a7a7bad93665d77c67b6cb079a054256a1c0ff4863d99268d6d313c; MDD_cvtype=SyzRTrJ5ztM=; MDD_dangdang.com=email%253DMTgyMzg4MzMxMDNAMTYzLmNvbQ%25253D%25253D%2526customerid%253DVziUkomVlfu9b2xaNo8Ysg%25253D%25253D; channelId=70000; __visit_id=20231028065938265329505705037539519; __out_refer=; __rpm=%7CbookshelfNew...1698447703384; __trace_id=20231028070144385979565391584335993", help='cookie')
    args = parser.parse_args()
    print(json.dumps(args.__dict__, ensure_ascii=False, indent = 4))
    id = args.id
    name = args.name
    token = args.token
    cookie = args.cookie
    bookSpider = BookSpider(id=id, name=name, token=token, cookie=cookie)
    bookSpider.spider()
