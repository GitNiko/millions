# 通达信数据解析

## 日线数据格式
```sh
00 ~ 03 字节 年月日，整型；
04 ~ 07 字节 开盘价*100，整型；
08 ~ 11 字节 最高价*100, 整型；
12 ~ 15 字节 最低价*100, 整型；
16 ~ 19 字节 收盘价*100, 整型；
20 ~ 23 字节 成交额（元），float型；
24 ~ 27 字节 成交量（股），整型；
28 ~ 31 字节 （保留）
```

## 5,1分钟数据格式
```sh
00 ~ 01 字节 日期，整型；计算方法为：year = floor(num/2048) + 2004;  month = floor(mod(num,2048)/100);   day = mod(mod(num,2048), 100);
02 ~ 03 字节 0点至目前的分钟数，整型
04 ~ 07 字节 开盘价*100，整型
08 ~ 11 字节 最高价*100，整型
12 ~ 15 字节 最低价*100，整型
16 ~ 19 字节 收盘价*100，整型
20 ~ 23 字节 成交额*100，float型
24 ~ 27 字节 成交量（股），整型
28 ~ 31 字节 （保留）
```


## 财务数据
https://github.com/rainx/pytdx/pull/150/files

## todo
- 支持除权数据
- 

## env
### widnows
```ps
$env:MILLIONS_TDX="D:\workspace\stock\tdx\millions\example"
cargo test
```