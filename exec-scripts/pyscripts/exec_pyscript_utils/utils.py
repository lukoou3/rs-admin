import time
import hashlib
import logging

def get_simple_logger(name="default", level="INFO"):
    """
    简单获取logger对象，用于在控制台打印日志
    :param name:
    :param level: DEBUG, INFO, WARNING/WARN, ERROR
    :return: logger
    """
    logger = logging.getLogger(name)
    logger.setLevel(level)
    handler = logging.StreamHandler(stream=None)
    fmt = "%(asctime)s - %(name)s - %(module)s - thread[%(threadName)s - %(thread)d] - %(levelname)s : %(message)s"
    handler.setFormatter(logging.Formatter(fmt=fmt, datefmt="%Y-%m-%d %H:%M:%S"))
    logger.addHandler(handler)
    return logger

logger = get_simple_logger("default")

def retry(funcName, func, n, wait_sleep=0):
    """
    执行某个函数, 最多重试n次。
    :param funcName: 执行的逻辑名称, 用于显示日志排查错误
    :param func: 逻辑
    :param n: 重试次数
    :param wait_sleep: 重试时是否延时, 默认=0不延时立即重试
    :return: func返回值
    """
    def retry_times(times=1):
        try:
            return func()
        except Exception as e:
            if times < n:
                logger.warning("retry func({}) failed for {} times error msg: ${}".format(funcName, times, e))
                if wait_sleep > 0:
                    time.sleep(wait_sleep)
                return retry_times(times + 1)
            else:
                logger.error("retry func({}) {} times finally failed ".format(funcName, times), exc_info=True)
                raise e
    return retry_times()

def make_md5(raw) -> str:
    """计算出一个字符串的MD5值"""
    md5 = hashlib.md5()
    md5.update(raw.encode())
    return md5.hexdigest()

